use crate::audio_formats::AudioFormat;
use crate::helpers::io::{read_to_stdin, send_from_stdout};
use actix_web::web::Bytes;
use async_process::{Command, Stdio};
use futures::channel::{mpsc, oneshot};
use futures::io;
use futures_lite::FutureExt;
use slog::{debug, error, Logger};

#[derive(Debug)]
pub enum AudioCodecError {
    ProcessError,
    StdoutUnavailable,
    StdinUnavailable,
}

pub struct AudioCodecService {
    path_to_ffmpeg: String,
    logger: Logger,
}

impl AudioCodecService {
    pub fn new(path_to_ffmpeg: &str, logger: &Logger) -> Self {
        AudioCodecService {
            path_to_ffmpeg: path_to_ffmpeg.to_string(),
            logger: logger.clone(),
        }
    }

    // TODO Whitelist supported formats.
    pub fn spawn_audio_decoder(
        &self,
        url: &str,
        offset: &u32,
    ) -> Result<mpsc::Receiver<Result<Bytes, io::Error>>, AudioCodecError> {
        let (sender, receiver) = mpsc::channel(4);

        debug!(self.logger, "Spawning audio decoder...");

        let offset = format!("{:.4}", offset / 1000);

        let child = match Command::new(&self.path_to_ffmpeg)
            .args(&[
                "-re",
                "-fflags",
                "fastseek",
                "-ss",
                &offset,
                "-i",
                &url,
                "-vn",
                "-filter",
                "afade=t=in:st=0:d=1",
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
                error!(self.logger, "Unable to start process"; "error" => ?error);
                return Err(AudioCodecError::ProcessError);
            }
        };

        debug!(self.logger, "Audio decoder spawned");

        let stdout = match child.stdout {
            Some(stdout) => stdout,
            None => {
                error!(self.logger, "Stdout is not available");
                return Err(AudioCodecError::StdoutUnavailable);
            }
        };

        actix_rt::spawn({
            let mut stdout = stdout;
            let mut sender = sender;

            let logger = self.logger.clone();

            async move {
                send_from_stdout(&mut stdout, &mut sender, logger).await;
            }
        });

        Ok(receiver)
    }

    pub fn spawn_audio_encoder(
        &self,
        format: &AudioFormat,
    ) -> Result<
        (
            mpsc::Sender<Result<Bytes, io::Error>>,
            mpsc::Receiver<Result<Bytes, io::Error>>,
        ),
        AudioCodecError,
    > {
        let (input_sender, input_receiver) = mpsc::channel(4);
        let (output_sender, output_receiver) = mpsc::channel(4);

        debug!(self.logger, "Spawning audio encoder process...");

        let process = match Command::new(&self.path_to_ffmpeg)
            .args(&[
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
                &format!("{}k", format.bitrate()),
                "-codec:a",
                &format.codec(),
                "-f",
                &format.format(),
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(process) => process,
            Err(error) => {
                error!(self.logger, "Unable to start process"; "error" => ?error);
                return Err(AudioCodecError::ProcessError);
            }
        };

        debug!(self.logger, "Audio encoder spawned");

        let stdout = match process.stdout {
            Some(stdout) => stdout,
            None => {
                error!(self.logger, "Stdout is not available");
                return Err(AudioCodecError::StdoutUnavailable);
            }
        };

        let stdin = match process.stdin {
            Some(stdin) => stdin,
            None => {
                error!(self.logger, "Stdin is not available");
                return Err(AudioCodecError::StdinUnavailable);
            }
        };

        let (term_signal, term_handler) = oneshot::channel::<()>();

        actix_rt::spawn({
            let mut input_receiver = input_receiver;
            let mut stdin = stdin;

            let logger = self.logger.clone();

            let pipe = async move {
                read_to_stdin(&mut input_receiver, &mut stdin, logger).await;
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

            async move {
                send_from_stdout(&mut stdout, &mut output_sender, logger).await;
                let _ = term_signal.send(());
            }
        });

        Ok((input_sender, output_receiver))
    }
}