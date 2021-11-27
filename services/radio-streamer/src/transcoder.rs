use actix_web::web::Bytes;
use async_process::{Command, Stdio};
use futures::channel::{mpsc, oneshot};
use futures::{io, SinkExt, StreamExt};
use futures_lite::FutureExt;
use slog::{debug, error, Logger};

use crate::audio_formats::AudioFormat;
use crate::constants::RAW_AUDIO_STEREO_BYTE_RATE;
use crate::helpers::io::{read_from_stdout, write_to_stdin};
use crate::metrics::Metrics;
use std::sync::Arc;
use std::time::Duration;

const STDIO_BUFFER_SIZE: usize = 4096;

// Should be enough for 5 seconds of audio.
const DECODER_CHANNEL_BUFFER: usize = 5 * (RAW_AUDIO_STEREO_BYTE_RATE / STDIO_BUFFER_SIZE);

#[derive(Debug)]
pub enum TranscoderError {
    ProcessError,
    StdoutUnavailable,
    StdinUnavailable,
}

pub struct TranscoderService {
    path_to_ffmpeg: String,
    logger: Logger,
    metrics: Arc<Metrics>,
}

impl TranscoderService {
    pub fn new(path_to_ffmpeg: &str, logger: Logger, metrics: Arc<Metrics>) -> Self {
        let path_to_ffmpeg = path_to_ffmpeg.to_owned();

        TranscoderService {
            path_to_ffmpeg,
            logger,
            metrics,
        }
    }

    // TODO Whitelist supported formats.
    pub fn decoder(
        &self,
        url: &str,
        offset: &Duration,
    ) -> Result<mpsc::Receiver<Result<Bytes, io::Error>>, TranscoderError> {
        let (sender, receiver) = mpsc::channel(DECODER_CHANNEL_BUFFER);

        let mut process = match Command::new(&self.path_to_ffmpeg)
            .args(&[
                "-v",
                "quiet",
                "-hide_banner",
                "-ss",
                &format!("{:.4}", offset.as_secs()),
                "-i",
                &url,
                "-vn",
                // "-filter",
                // "afade=t=in:st=0:d=1",
                "-codec:a",
                "pcm_s16le",
                "-ar",
                "44100",
                "-ac",
                "2",
                "-f",
                "s16le",
                "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
        {
            Ok(process) => process,
            Err(error) => {
                error!(self.logger, "Unable to start decoder process: error occurred"; "error" => ?error);
                return Err(TranscoderError::ProcessError);
            }
        };

        let status = process.status();

        let stdout = match process.stdout {
            Some(stdout) => stdout,
            None => {
                error!(
                    self.logger,
                    "Unable to start decoder process: stdout is not available"
                );
                return Err(TranscoderError::StdoutUnavailable);
            }
        };

        actix_rt::spawn({
            let mut sender = sender;

            let stdout = stdout;
            let logger = self.logger.clone();
            let metrics = self.metrics.clone();

            async move {
                metrics.inc_spawned_decoder_processes();

                {
                    let mut stdout = stdout;
                    let mut buffer = vec![0u8; STDIO_BUFFER_SIZE];

                    while let Some(result) = read_from_stdout(&mut stdout, &mut buffer).await {
                        if let Err(error) = sender.send(result).await {
                            error!(logger, "Unable to send data from decoder: I/O error"; "error" => ?error);
                            break;
                        };
                    }
                }

                metrics.dec_spawned_decoder_processes();

                if let Ok(exit_status) = status.await {
                    debug!(logger, "Audio decoder closed"; "exit_code" => exit_status.code());
                }
            }
        });

        Ok(receiver)
    }

    pub fn encoder(
        &self,
        format: &AudioFormat,
    ) -> Result<
        (
            mpsc::Sender<Result<Bytes, io::Error>>,
            mpsc::Receiver<Result<Bytes, io::Error>>,
        ),
        TranscoderError,
    > {
        let (input_sender, input_receiver) = mpsc::channel(0);
        let (output_sender, output_receiver) = mpsc::channel(0);

        let process = match Command::new(&self.path_to_ffmpeg)
            .args(&[
                "-v",
                "quiet",
                "-hide_banner",
                "-acodec",
                "pcm_s16le",
                "-ar",
                "44100",
                "-ac",
                "2",
                "-f",
                "s16le",
                "-i",
                "-",
                // TODO Replace with apply of pre-computed audio peak level.
                "-af",
                "compand=0 0:1 1:-90/-900 -70/-70 -21/-21 0/-15:0.01:12:0:0",
                "-map_metadata",
                "-1",
                "-vn",
                "-ar",
                "44100",
                "-ac",
                "2",
                "-b:a",
                &format!("{}k", format.bitrate),
                "-codec:a",
                &format.codec,
                "-f",
                &format.format,
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(process) => process,
            Err(error) => {
                error!(self.logger, "Unable to start encoder process: error occurred"; "error" => ?error);
                return Err(TranscoderError::ProcessError);
            }
        };

        let stdout = match process.stdout {
            Some(stdout) => stdout,
            None => {
                error!(
                    self.logger,
                    "Unable to start encoder process: stdout is not available"
                );
                return Err(TranscoderError::StdoutUnavailable);
            }
        };

        let stdin = match process.stdin {
            Some(stdin) => stdin,
            None => {
                error!(
                    self.logger,
                    "Unable to start encoder process: stdin is not available"
                );
                return Err(TranscoderError::StdinUnavailable);
            }
        };

        let (term_signal, term_handler) = oneshot::channel::<()>();

        actix_rt::spawn({
            let mut input_receiver = input_receiver;
            let mut stdin = stdin;

            let logger = self.logger.clone();

            let pipe = async move {
                while let Some(result) = input_receiver.next().await {
                    match result {
                        Ok(bytes) => {
                            if let Err(error) = write_to_stdin(&mut stdin, bytes).await {
                                error!(logger, "Unable to write data to encoder: error occurred"; "error" => ?error);
                                break;
                            }
                        }
                        Err(error) => {
                            error!(logger, "Unable to write data to encoder: source not available"; "error" => ?error);
                            break;
                        }
                    };
                }
            };

            let abort = async move {
                let _ = term_handler.await;
            };

            abort.or(pipe)
        });

        actix_rt::spawn({
            let mut stdout = stdout;
            let mut output_sender = output_sender;

            let logger = self.logger.clone();
            let metrics = self.metrics.clone();

            async move {
                metrics.inc_spawned_encoder_processes();

                let mut buffer = vec![0u8; STDIO_BUFFER_SIZE];
                while let Some(result) = read_from_stdout(&mut stdout, &mut buffer).await {
                    if let Err(error) = output_sender.send(result).await {
                        error!(logger, "Unable to send data from encoder: I/O error"; "error" => ?error);
                        break;
                    };
                }

                metrics.dec_spawned_encoder_processes();

                let _ = term_signal.send(());
            }
        });

        Ok((input_sender, output_receiver))
    }
}