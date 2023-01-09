use crate::audio_formats::AudioFormat;
use crate::helpers::io::{read_exact_from_stdout, read_from_stdout, write_to_stdin};
use crate::helpers::system::which;
use crate::metrics::Metrics;
use crate::stream::constants::{
    AUDIO_BITRATE, AUDIO_BYTES_PER_SECOND, AUDIO_CHANNELS_NUMBER, AUDIO_PACKET_SIZE,
    AUDIO_SAMPLING_FREQUENCY,
};
use crate::stream::types::{Buffer, Format};
use actix_web::web::Bytes;
use async_process::{Command, Stdio};
use futures::channel::{mpsc, oneshot};
use futures::io::BufReader;
use futures::{SinkExt, StreamExt};
use futures_lite::{AsyncBufReadExt, FutureExt};
use lazy_static::lazy_static;
use regex::Regex;
use scopeguard::defer;
use slog::{debug, error, info, o, trace, warn, Logger};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const STDIO_BUFFER_SIZE: usize = 64576;

lazy_static! {
    static ref FFMPEG_COMMAND: &'static str =
        Box::leak(Box::new(which("ffmpeg").expect("Unable to locate ffmpeg")));
    static ref FFPROBE_COMMAND: &'static str = Box::leak(Box::new(
        which("ffprobe").expect("Unable to locate ffprobe")
    ));
    // muxer <- type:audio pkt_pts:288639 pkt_pts_time:6.01331 pkt_dts:288639 pkt_dts_time:6.01331 size:17836
    static ref FFMPEG_OUTPUT_PTS_REGEX: &'static Regex =
        Box::leak(Box::new(Regex::new(r"muxer <- type:audio pkt_pts:([0-9]+) pkt_pts_time:([0-9]+\.[0-9]+) pkt_dts:([0-9]+) pkt_dts_time:([0-9]+\.[0-9]+) size:([0-9]+)").unwrap()));
}

#[derive(Debug)]
pub(crate) enum DecoderError {
    ProcessError,
    StdoutUnavailable,
    StderrUnavailable,
}

pub(crate) enum DecoderOutput {
    Buffer(Buffer),
    EOF,
}

pub(crate) fn build_ffmpeg_decoder(
    source_url: &str,
    offset: &Duration,
    logger: &Logger,
    metrics: &Metrics,
) -> Result<mpsc::Receiver<DecoderOutput>, DecoderError> {
    let (mut tx, rx) = mpsc::channel::<DecoderOutput>(0);
    let logger = logger.new(o!("kind" => "ffmpeg_decoder"));

    let mut start_time = Some(Instant::now());

    let mut process = match Command::new(*FFMPEG_COMMAND)
        .args(&[
            "-debug_ts",
            "-v",
            "info",
            "-nostats",
            "-hide_banner",
            "-ss",
            &format!("{:.4}", offset.as_secs_f32()),
            "-i",
            &source_url,
            "-vn",
            "-codec:a",
            "pcm_s16le",
            "-ar",
            &AUDIO_SAMPLING_FREQUENCY.to_string(),
            "-ac",
            &AUDIO_CHANNELS_NUMBER.to_string(),
            "-f",
            "s16le", // BYTES_PER_SAMPLE = 2
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(process) => process,
        Err(error) => {
            error!(logger, "Unable to start the decoder process"; "error" => ?error);
            return Err(DecoderError::ProcessError);
        }
    };
    let format = Arc::new(Format {
        codec: "pcm_s16le".to_string(),
        container: "s16le".to_string(),
        bitrate: AUDIO_BITRATE,
        channels: AUDIO_CHANNELS_NUMBER,
        sample_rate: AUDIO_SAMPLING_FREQUENCY,
    });

    info!(logger, "Started audio decoder"; "url" => source_url, "offset" => ?offset);

    let status = process.status();

    let stdout = match process.stdout.take() {
        Some(stdout) => stdout,
        None => {
            error!(logger, "Unable to start decoder: stdout is not available");
            return Err(DecoderError::StdoutUnavailable);
        }
    };

    let stderr = match process.stderr.take() {
        Some(stderr) => stderr,
        None => {
            error!(logger, "Unable to start decoder: stderr is not available");
            return Err(DecoderError::StderrUnavailable);
        }
    };

    let (next_packet_sender, next_packet_receiver) = mpsc::channel::<(Duration, usize)>(10);

    actix_rt::spawn({
        let logger = logger.clone();

        let mut next_packet_sender = next_packet_sender;

        async move {
            let mut err_lines = BufReader::new(stderr).split(b'\n');

            while let Some(Ok(line)) = err_lines.next().await {
                let line = String::from_utf8_lossy(&line);

                trace!(logger, "ffmpeg stderr: {}", line);

                if let Some(captures) = FFMPEG_OUTPUT_PTS_REGEX.captures(&line) {
                    let last_encoded_dts = Duration::from_secs_f64(captures[2].parse().unwrap());
                    let last_packet_size = captures[5].parse().unwrap();

                    let _ = next_packet_sender
                        .send((last_encoded_dts, last_packet_size))
                        .await;
                }
            }

            drop(err_lines);
        }
    });

    actix_rt::spawn({
        let metrics = metrics.clone();
        let logger = logger.clone();

        let mut bytes_sent = 0usize;

        let mut next_packet_receiver = next_packet_receiver;

        async move {
            metrics.inc_active_decoders();

            defer!(metrics.dec_active_decoders());

            let mut stdout = stdout;

            let mut channel_closed = false;

            loop {
                let (next_dts, next_size) = match next_packet_receiver.next().await {
                    Some(d) => d,
                    None => break,
                };

                trace!(
                    logger,
                    "planned packet size={} dts={:?}",
                    next_size,
                    next_dts
                );

                match read_exact_from_stdout(&mut stdout, &next_size).await {
                    Some(Ok(bytes)) => {
                        if let Some(time) = start_time.take() {
                            metrics.update_audio_decoder_track_open_duration(time.elapsed());
                        }

                        let bytes_len = bytes.len();

                        trace!(logger, "read packet size={}", bytes_len);

                        let timed_bytes = Buffer::new(bytes, next_dts, &format);

                        if let Err(_) = tx.send(DecoderOutput::Buffer(timed_bytes)).await {
                            channel_closed = true;
                            break;
                        };

                        bytes_sent += bytes_len;
                    }
                    _ => break,
                }
            }

            let _ = tx.send(DecoderOutput::EOF).await;

            drop(stdout);

            if let Ok(exit_status) = status.await {
                match exit_status.code() {
                    Some(code) if code == 1 && channel_closed => {
                        debug!(
                            logger,
                            "Decoder exited because output channel has been closed"
                        );
                    }
                    Some(code) if code != 0 => {
                        warn!(logger, "Decoder exited with non-zero exit code"; "exit_code" => code);
                    }
                    _ => (),
                }
            }
        }
    });

    Ok(rx)
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum EncoderError {
    #[error("Error while processing data")]
    ProcessError,
    #[error("Unable to access stdout")]
    StdoutUnavailable,
    #[error("Unable to access stdin")]
    StdinUnavailable,
    #[error("Unable to access stderr")]
    StderrUnavailable,
}

pub(crate) enum EncoderOutput {
    Buffer(Buffer),
    EOF,
}

pub(crate) fn build_ffmpeg_encoder(
    audio_format: &AudioFormat,
    logger: &Logger,
    metrics: &Metrics,
) -> Result<(mpsc::Sender<Buffer>, mpsc::Receiver<EncoderOutput>), EncoderError> {
    let logger = logger.new(o!("kind" => "ffmpeg_encoder"));

    let mut process = match Command::new(*FFMPEG_COMMAND)
        .args(&[
            "-debug_ts",
            "-v",
            "info",
            "-nostats",
            "-hide_banner",
            "-acodec",
            "pcm_s16le",
            "-ar",
            &AUDIO_SAMPLING_FREQUENCY.to_string(),
            "-ac",
            &AUDIO_CHANNELS_NUMBER.to_string(),
            "-f",
            "s16le",
            "-i",
            "-",
            // TODO Replace with apply of pre-computed audio peak level.
            // "-af",
            // "compand=0 0:1 1:-90/-900 -70/-70 -21/-21 0/-15:0.01:12:0:0",
            "-map_metadata",
            "-1",
            "-vn",
            "-ar",
            &AUDIO_SAMPLING_FREQUENCY.to_string(),
            "-ac",
            "2",
            "-b:a",
            &format!("{}k", audio_format.bitrate),
            "-codec:a",
            &audio_format.codec,
            "-f",
            &audio_format.format,
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(process) => process,
        Err(error) => {
            error!(logger, "Unable to start encoder process: error occurred"; "error" => ?error);
            return Err(EncoderError::ProcessError);
        }
    };

    let stdout = match process.stdout.take() {
        Some(stdout) => stdout,
        None => {
            error!(
                logger,
                "Unable to start encoder process: stdout is not available"
            );
            return Err(EncoderError::StdoutUnavailable);
        }
    };

    let stdin = match process.stdin.take() {
        Some(stdin) => stdin,
        None => {
            error!(
                logger,
                "Unable to start encoder process: stdin is not available"
            );
            return Err(EncoderError::StdinUnavailable);
        }
    };

    let stderr = match process.stderr.take() {
        Some(stderr) => stderr,
        None => {
            error!(logger, "Unable to start decoder: stderr is not available");
            return Err(EncoderError::StderrUnavailable);
        }
    };

    let (next_packet_sender, next_packet_receiver) = mpsc::channel::<(Duration, usize)>(10);

    let (term_signal, term_handler) = oneshot::channel::<()>();

    let (sink_sender, sink_receiver) = mpsc::channel::<Buffer>(0);
    let (src_sender, src_receiver) = mpsc::channel::<EncoderOutput>(0);

    let format = Arc::new(Format {
        codec: audio_format.codec.to_string(),
        container: audio_format.format.to_string(),
        sample_rate: AUDIO_SAMPLING_FREQUENCY,
        channels: AUDIO_CHANNELS_NUMBER,
        bitrate: audio_format.bitrate as usize,
    });

    actix_rt::spawn({
        let logger = logger.clone();

        let mut next_packet_sender = next_packet_sender;

        async move {
            let mut err_lines = BufReader::new(stderr).split(b'\n');

            while let Some(Ok(line)) = err_lines.next().await {
                let line = String::from_utf8_lossy(&line);

                trace!(logger, "ffmpeg stderr: {}", line);

                if let Some(captures) = FFMPEG_OUTPUT_PTS_REGEX.captures(&line) {
                    let last_encoded_dts = Duration::from_secs_f64(captures[2].parse().unwrap());
                    let last_packet_size = captures[5].parse().unwrap();

                    let _ = next_packet_sender
                        .send((last_encoded_dts, last_packet_size))
                        .await;
                }
            }

            drop(err_lines);
        }
    });

    actix_rt::spawn({
        let mut sink_receiver = sink_receiver;
        let mut stdin = stdin;

        let logger = logger.clone();

        let pipe = async move {
            while let Some(buffer) = sink_receiver.next().await {
                if let Err(error) = write_to_stdin(&mut stdin, buffer.into_bytes()).await {
                    error!(logger, "Unable to write data to encoder: error occurred"; "error" => ?error);
                    break;
                }
            }

            drop(stdin);
        };

        let abort = async move {
            let _ = term_handler.await;
        };

        abort.or(pipe)
    });

    actix_rt::spawn({
        let mut stdout = stdout;
        let mut src_sender = src_sender;

        let metrics = metrics.clone();
        let format_string = audio_format.to_string();

        let mut next_packet_receiver = next_packet_receiver;

        async move {
            metrics.inc_active_encoders(&format_string);

            defer!(metrics.dec_active_encoders(&format_string));

            loop {
                let (next_dts, next_size) = match next_packet_receiver.next().await {
                    Some(d) => d,
                    None => break,
                };

                trace!(
                    logger,
                    "planned packet size={} dts={:?}",
                    next_size,
                    next_dts
                );

                match read_exact_from_stdout(&mut stdout, &next_size).await {
                    Some(Ok(bytes)) => {
                        let bytes_len = bytes.len();

                        trace!(logger, "read packet size={}", bytes_len);

                        if let Err(_) = src_sender
                            .send(EncoderOutput::Buffer(Buffer::new(
                                bytes,
                                Duration::default(),
                                &format,
                            )))
                            .await
                        {
                            break;
                        };
                    }
                    _ => break,
                }
            }

            let _ = src_sender.send(EncoderOutput::EOF).await;

            drop(stdout);

            let _ = term_signal.send(());
        }
    });

    Ok((sink_sender, src_receiver))
}
