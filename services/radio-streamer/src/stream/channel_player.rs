use actix_rt::time::Instant;
use actix_web::web::Bytes;
use async_broadcast;
use futures::channel::oneshot;
use futures::lock::Mutex;
use futures::StreamExt;
use slog::{debug, error, warn, Logger};
use std::sync::Arc;
use std::time::Duration;

use crate::codec::AudioCodecService;
use crate::constants::{
    ALLOWED_DELAY_FOR_PRE_SPAWNED_RECEIVER, PREFETCH_TIME, RAW_AUDIO_STEREO_BYTE_RATE,
};
use crate::helpers::io::{
    pipe_channel_with_cancel, sleep_until_deadline, throttled_channel, PipeChannelError,
};
use crate::metrics::Metrics;
use crate::mor_backend_client::{MorBackendClient, MorBackendClientError};

pub struct ChannelPlayer {
    pub audio_receiver: async_broadcast::Receiver<Bytes>,
    pub title_receiver: async_broadcast::Receiver<String>,
    restart_sender: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl ChannelPlayer {
    fn create(
        channel_id: usize,
        client_id: Option<String>,
        backend_client: Arc<MorBackendClient>,
        audio_codec_service: Arc<AudioCodecService>,
        metrics: Arc<Metrics>,
        logger: Arc<Logger>,
    ) -> Self {
        let (audio_sender, audio_receiver) = async_broadcast::broadcast(1);
        let (title_sender, title_receiver) = async_broadcast::broadcast(1);

        let restart_sender = Arc::new(Mutex::new(None));
        let pre_spawned_receiver: Arc<Mutex<_>> = Arc::new(Mutex::new(None));

        let (mut thr_sender, thr_receiver) = throttled_channel(
            RAW_AUDIO_STEREO_BYTE_RATE,
            RAW_AUDIO_STEREO_BYTE_RATE * PREFETCH_TIME.as_secs() as usize,
        );

        actix_rt::spawn({
            let logger = logger.clone();
            let audio_sender = audio_sender.clone();

            let mut thr_receiver = thr_receiver;

            async move {
                while let Some(Ok(bytes)) = thr_receiver.next().await {
                    if let Err(error) = audio_sender.broadcast(bytes).await {
                        error!(logger, "Unable to broadcast audio data"; "error" => ?error);
                        break;
                    }
                }
            }
        });

        actix_rt::spawn({
            let logger = logger.clone();
            let restart_sender = restart_sender.clone();

            async move {
                metrics.inc_streaming_in_progress();

                loop {
                    let now_playing = match backend_client
                        .get_now_playing(&channel_id, client_id.clone(), &PREFETCH_TIME)
                        .await
                    {
                        Ok(now_playing) => {
                            debug!(logger, "Now playing: {:?}", &now_playing);
                            now_playing
                        }
                        Err(MorBackendClientError::ChannelNotFound) => {
                            // Channel was deleted when streaming. Nothing special.
                            break;
                        }
                        Err(error) => {
                            error!(logger, "Unable to get now playing"; "error" => ?error);
                            break;
                        }
                    };

                    let (restart_sender_internal, mut restart_receiver_internal) =
                        oneshot::channel();

                    drop(restart_sender.lock().await.replace(restart_sender_internal));

                    let current_track = now_playing.current_track;
                    let next_track = now_playing.next_track;

                    let current_track_left = current_track.duration - current_track.offset;
                    let should_finish_at = Instant::now() + current_track_left;

                    let mut current_track_receiver = match pre_spawned_receiver.lock().await.take()
                    {
                        Some(receiver)
                            if current_track.offset < ALLOWED_DELAY_FOR_PRE_SPAWNED_RECEIVER =>
                        {
                            receiver
                        }

                        _ => {
                            match audio_codec_service
                                .spawn_audio_decoder(&current_track.url, &current_track.offset)
                            {
                                Ok(receiver) => receiver,
                                Err(error) => {
                                    error!(logger, "Unable to spawn current track decoder"; "error" => ?error);
                                    break;
                                }
                            }
                        }
                    };

                    match audio_codec_service
                        .spawn_audio_decoder(&next_track.url, &Duration::default())
                    {
                        Ok(receiver) => {
                            pre_spawned_receiver.lock().await.replace(receiver);
                        }
                        Err(error) => {
                            error!(logger, "Unable to spawn next track decoder"; "error" => ?error);
                            break;
                        }
                    }

                    let title = format!("StreamTitle='{}';", &current_track.title);

                    if let Err(error) = title_sender.broadcast(title).await {
                        error!(logger, "Unable to send track title"; "error" => ?error);
                    }

                    // TODO Copy bytes from decoder to ChannelPlayer with throttle

                    let result = pipe_channel_with_cancel(
                        &mut current_track_receiver,
                        &mut thr_sender,
                        &mut restart_receiver_internal,
                    )
                    .await;

                    match result {
                        Ok(_) => {
                            let sleep_fut = sleep_until_deadline(
                                should_finish_at,
                                &mut restart_receiver_internal,
                            );

                            if let Err(error) = sleep_fut.await {
                                warn!(logger, "Sleep cancelled");
                            }
                        }
                        Err(PipeChannelError::CancelError(_)) => {
                            debug!(logger, "Received restart signal...");
                            drop(pre_spawned_receiver.lock().await.take());
                        }
                        Err(PipeChannelError::SendError(error)) => {
                            error!(logger, "Unable to pipe bytes"; "error" => ?error);
                            break;
                        }
                    }
                }

                metrics.dec_streaming_in_progress();
            }
        });

        ChannelPlayer {
            audio_receiver,
            title_receiver,
            restart_sender,
        }
    }

    pub async fn restart(&self) {
        if let Some(sender) = self.restart_sender.lock().await.take() {
            sender.send(());
        }
    }
}
