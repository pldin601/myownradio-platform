use crate::utils::{rescale_audio_frame_ts, Frame, Timestamp};
use crate::{INTERNAL_CHANNELS_NUMBER, INTERNAL_TIME_BASE};
use crate::{INTERNAL_SAMPLE_SIZE, INTERNAL_SAMPLING_FREQUENCY, RESAMPLER_TIME_BASE};
use ffmpeg_next::codec::Context;
use ffmpeg_next::format::context::Input;
use ffmpeg_next::format::sample::Type;
use ffmpeg_next::format::Sample;
use ffmpeg_next::frame::Audio;
use ffmpeg_next::{rescale, ChannelLayout, Packet, Rescale};
use futures::channel::mpsc::{channel, Receiver, SendError, Sender};
use futures::SinkExt;
use std::time::Duration;
use tracing::error;

struct AudioDecoder {
    input_index: usize,
    decoder: ffmpeg_next::decoder::Audio,
    resampler: ffmpeg_next::software::resampling::Context,
    async_runtime: actix_rt::Runtime,
    async_sender: Sender<Frame>,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioDecoderError {
    #[error("Unable to open input file: {0}")]
    OpenFileError(ffmpeg_next::Error),
    #[error("Audio stream not found")]
    AudioStreamNotFound,
    #[error("Audio decoding failed: {0}")]
    AudioDecoderError(ffmpeg_next::Error),
    #[error("Audio resampling failed: {0}")]
    ResamplingError(ffmpeg_next::Error),
    #[error("Unable to seek input to specified position")]
    SeekError(ffmpeg_next::Error),
    #[error("Unable to send processed frame to Sender")]
    SendError(SendError),
}

impl AudioDecoder {
    fn resample_and_process_frames(&mut self, decoded: &Audio) -> Result<(), AudioDecoderError> {
        let mut resampled = Audio::empty();
        resampled.clone_from(decoded);

        // @TODO Handle resampler delays somehow.
        // Using flush after run causes error: `ffmpeg::Error(1668179714: Output changed)`
        let _delay = self
            .resampler
            .run(decoded, &mut resampled)
            .map_err(|error| AudioDecoderError::ResamplingError(error))?;

        rescale_audio_frame_ts(
            &mut resampled,
            self.decoder.time_base(),
            RESAMPLER_TIME_BASE.into(),
        );

        self.async_runtime
            .block_on(self.async_sender.send(resampled.into()))
            .map_err(|error| AudioDecoderError::SendError(error))?;

        Ok(())
    }

    fn send_packet_to_decoder(&mut self, packet: &Packet) -> Result<(), AudioDecoderError> {
        self.decoder
            .send_packet(packet)
            .map_err(|error| AudioDecoderError::AudioDecoderError(error))
    }

    fn send_eof_to_decoder(&mut self) -> Result<(), AudioDecoderError> {
        self.decoder
            .send_eof()
            .map_err(|error| AudioDecoderError::AudioDecoderError(error))
    }

    fn receive_and_process_decoded_frames(&mut self) -> Result<(), AudioDecoderError> {
        let mut decoded = Audio::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let timestamp = decoded.timestamp();
            decoded.set_pts(timestamp);

            self.resample_and_process_frames(&decoded)?;
        }

        Ok(())
    }
}

fn make_audio_decoder(
    ictx: &mut Input,
    async_runtime: actix_rt::Runtime,
    async_sender: Sender<Frame>,
) -> Result<AudioDecoder, AudioDecoderError> {
    let input = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Audio)
        .ok_or_else(|| AudioDecoderError::AudioStreamNotFound)?;
    let input_index = input.index();
    let context = Context::from_parameters(input.parameters())
        .map_err(|error| AudioDecoderError::AudioDecoderError(error))?;

    let mut decoder = context
        .decoder()
        .audio()
        .map_err(|error| AudioDecoderError::AudioDecoderError(error))?;

    decoder
        .set_parameters(input.parameters())
        .map_err(|error| AudioDecoderError::AudioDecoderError(error))?;

    if decoder.channel_layout().is_empty() {
        decoder.set_channel_layout(ChannelLayout::default(decoder.channels() as i32));
    }

    let resampler = decoder
        .resampler(
            Sample::I16(Type::Packed),
            ChannelLayout::default(INTERNAL_CHANNELS_NUMBER as i32),
            INTERNAL_SAMPLING_FREQUENCY as u32,
        )
        .map_err(|error| AudioDecoderError::ResamplingError(error))?;

    Ok(AudioDecoder {
        input_index,
        decoder,
        resampler,
        async_runtime,
        async_sender,
    })
}

impl Into<Frame> for Audio {
    fn into(self) -> Frame {
        let pts = Timestamp::new(self.pts().unwrap_or_default(), RESAMPLER_TIME_BASE);
        let duration = Timestamp::new(self.samples() as i64, RESAMPLER_TIME_BASE);

        let data_len = self.samples() * INTERNAL_SAMPLE_SIZE;
        let data = &self.data(0)[..data_len];

        Frame::new(pts, duration, Vec::from(data))
    }
}

#[tracing::instrument]
pub fn decode_audio_file(
    source_url: &str,
    offset: &Duration,
) -> Result<Receiver<Frame>, AudioDecoderError> {
    let (frame_sender, frame_receiver) = channel(0);

    let mut ictx = ffmpeg_next::format::input(&source_url.to_string())
        .map_err(|error| AudioDecoderError::OpenFileError(error))?;

    if !offset.is_zero() {
        let position_millis = offset.as_millis() as i64;
        let position = position_millis.rescale(INTERNAL_TIME_BASE, rescale::TIME_BASE);

        ictx.seek(position, ..position)
            .map_err(|error| AudioDecoderError::SeekError(error))?;
    };

    std::thread::spawn(move || {
        let async_runtime = actix_rt::Runtime::new().expect("Unable to initialize async runtime");
        let mut audio_decoder = make_audio_decoder(&mut ictx, async_runtime, frame_sender)
            .expect("Unable to initialize audio decoder");

        for (stream, mut packet) in ictx.packets() {
            if stream.index() == audio_decoder.input_index {
                packet.rescale_ts(stream.time_base(), audio_decoder.decoder.time_base());
                if let Err(error) = audio_decoder.send_packet_to_decoder(&packet) {
                    error!(?error, "Unable to send packet to decoder");
                    return;
                }

                if let Err(error) = audio_decoder.receive_and_process_decoded_frames() {
                    error!(?error, "Unable to receive and process decoded frames");
                    return;
                };
            }
        }

        if let Err(error) = audio_decoder.send_eof_to_decoder() {
            error!(?error, "Unable to send EOL to decoder");
            return;
        };

        if let Err(error) = audio_decoder.receive_and_process_decoded_frames() {
            error!(?error, "Unable to receive and process final decoded frames");
            return;
        }
    });

    Ok(frame_receiver)
}

#[cfg(test)]
mod tests {
    use crate::decoder::make_audio_decoder;
    use ffmpeg_next::format::input;
    use ffmpeg_next::{rescale, Rescale};
    use futures::channel::mpsc::channel;
    use futures::StreamExt;
    use std::time::Duration;
    use tracing_test::traced_test;

    #[ctor::ctor]
    fn init() {
        ffmpeg_next::init().expect("Unable to initialize ffmpeg");
        ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Trace);
    }

    const TEST_FILES: [(&str, Duration, Duration); 13] = [
        (
            "tests/fixtures/test_file.wav",
            Duration::from_millis(2834),
            Duration::from_millis(0),
        ),
        (
            "tests/fixtures/test_file.wav",
            Duration::from_millis(2834),
            Duration::from_millis(1500),
        ),
        (
            "tests/fixtures/test_file.aac",
            Duration::from_millis(2877),
            Duration::from_millis(0),
        ),
        (
            "tests/fixtures/test_file.aac",
            Duration::from_millis(2877),
            Duration::from_millis(1500),
        ),
        (
            "tests/fixtures/test_file.flac",
            Duration::from_millis(2833),
            Duration::from_millis(0),
        ),
        (
            "tests/fixtures/test_file.flac",
            Duration::from_millis(2833),
            Duration::from_millis(1500),
        ),
        (
            "tests/fixtures/test_file.m4a",
            Duration::from_millis(2854),
            Duration::from_millis(0),
        ),
        (
            "tests/fixtures/test_file.m4a",
            Duration::from_millis(2854),
            Duration::from_millis(1500),
        ),
        (
            "tests/fixtures/test_file.mp3",
            Duration::from_millis(2858),
            Duration::from_millis(0),
        ),
        (
            "tests/fixtures/test_file.mp3",
            Duration::from_millis(2858),
            Duration::from_millis(1500),
        ),
        (
            "tests/fixtures/test_file.ogg",
            Duration::from_millis(2834),
            Duration::from_millis(0),
        ),
        (
            "tests/fixtures/test_file.ogg",
            Duration::from_millis(2834),
            Duration::from_millis(1500),
        ),
        (
            "tests/fixtures/sample-6s.mp3",
            Duration::from_millis(6415),
            Duration::from_millis(1500),
        ),
    ];

    #[actix_rt::test]
    async fn test_opening_source_files() {
        for (filename, ..) in TEST_FILES {
            assert!(input(&filename).is_ok());
        }
    }

    #[actix_rt::test]
    async fn test_seeking_source_files() {
        let position = 1500i64.rescale(crate::INTERNAL_TIME_BASE, rescale::TIME_BASE);
        for (filename, ..) in TEST_FILES {
            assert!(input(&filename)
                .expect("Unable to open input file")
                .seek(position, ..position)
                .is_ok());
        }
    }

    #[actix_rt::test]
    async fn test_iterating_over_source_packets() {
        for (filename, ..) in TEST_FILES {
            let mut ictx = input(&filename).expect("Unable to open input file");

            assert!(ictx.packets().last().is_some());
        }
    }

    #[actix_rt::test]
    async fn test_audio_decoder_decoding_packets() {
        for (filename, ..) in TEST_FILES {
            let (frame_sender, mut frame_receiver) = channel(1000);

            actix_rt::task::spawn_blocking(move || {
                let mut ictx = input(&filename).expect("Unable to open input file");
                let async_runtime =
                    actix_rt::Runtime::new().expect("Unable to initialize async runtime");

                let mut audio_decoder = make_audio_decoder(&mut ictx, async_runtime, frame_sender)
                    .expect("Unable to initialize audio decoder");

                for (stream, packet) in ictx.packets() {
                    if stream.index() == audio_decoder.input_index {
                        audio_decoder
                            .send_packet_to_decoder(&packet)
                            .expect("Unable to send packet to decoder");

                        audio_decoder
                            .receive_and_process_decoded_frames()
                            .expect("Unable to process decoded frames");
                    }
                }

                audio_decoder
                    .send_eof_to_decoder()
                    .expect("Unable to send EOF to decoder");

                audio_decoder
                    .receive_and_process_decoded_frames()
                    .expect("Unable to process decoded frames");
            })
            .await
            .expect("Blocking task failed");

            let mut had_frames = false;

            while frame_receiver.next().await.is_some() {
                had_frames = true;
            }

            assert!(had_frames);
        }
    }

    #[actix_rt::test]
    async fn test_decoding_test_files() {
        for (filename, expected_duration, offset) in TEST_FILES {
            eprintln!("file: {}", filename);

            let mut frames =
                super::decode_audio_file(filename, &offset).expect("Unable to decode file");

            let mut duration = Duration::default();

            while let Some(frame) = frames.next().await {
                duration = frame.duration().into();
                duration += frame.pts().into();
            }

            assert_eq!(expected_duration, duration);
        }
    }

    #[actix_rt::test]
    #[traced_test]
    async fn test_decoding_file_by_url() {
        let test_file_url = "https://download.samplelib.com/mp3/sample-6s.mp3";
        let mut frames = super::decode_audio_file(test_file_url, &Duration::from_secs(0))
            .expect("Unable to decode file");

        let mut duration = Duration::default();

        while let Some(frame) = frames.next().await {
            duration = frame.duration().into();
            duration += frame.pts().into();
        }

        assert_eq!(Duration::from_millis(6415), duration);
    }

    #[actix_rt::test]
    async fn test_seek_accuracy() {
        let test_file_path = "tests/fixtures/test_file.wav";
        let seek_position = Duration::from_millis(400);

        let frame = super::decode_audio_file(test_file_path, &seek_position)
            .expect("Unable to decode file")
            .next()
            .await
            .unwrap();

        assert_eq!(seek_position, frame.pts().into());
    }
}
