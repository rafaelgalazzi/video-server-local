use serde::Serialize;
use thiserror::Error;

use crate::{
    compatibility::{ClientCapabilities, DecisionReason, PlaybackMethod},
    media_jobs::{
        MediaJobConfig, MediaJobId, MediaJobManager, MediaJobOutput, MediaJobOutputError,
        MediaJobSnapshot, MediaJobStartError,
    },
    LocalStreamCore,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackPreparation {
    pub method: PlaybackMethod,
    pub reason: DecisionReason,
    pub job_id: Option<MediaJobId>,
    pub output_name: Option<&'static str>,
    pub content_type: Option<&'static str>,
}

#[derive(Debug, Error)]
pub enum PlaybackPrepareError {
    #[error("the requested media is unavailable")]
    Unavailable,
    #[error("the media cannot be played by this client")]
    Incompatible,
}

#[derive(Clone)]
pub struct LocalPlaybackService {
    jobs: MediaJobManager,
}

impl LocalPlaybackService {
    pub async fn start(config: MediaJobConfig) -> Result<Self, MediaJobStartError> {
        Ok(Self {
            jobs: MediaJobManager::start(config).await?,
        })
    }

    pub async fn prepare(
        &self,
        core: &LocalStreamCore,
        media_id: &str,
        client: &ClientCapabilities,
    ) -> Result<PlaybackPreparation, PlaybackPrepareError> {
        let decision = core
            .playback_decision(media_id, client)
            .map_err(|_| PlaybackPrepareError::Unavailable)?;
        match decision.method {
            PlaybackMethod::DirectPlay => Ok(PlaybackPreparation {
                method: decision.method,
                reason: decision.reason,
                job_id: None,
                output_name: None,
                content_type: None,
            }),
            PlaybackMethod::Remux => {
                let submission = core
                    .submit_remux(&self.jobs, media_id, &decision)
                    .await
                    .map_err(|_| PlaybackPrepareError::Unavailable)?;
                Ok(PlaybackPreparation {
                    method: decision.method,
                    reason: decision.reason,
                    job_id: Some(submission.job.id),
                    output_name: Some(submission.output_name),
                    content_type: Some(submission.content_type),
                })
            }
            PlaybackMethod::Transcode => {
                let submission = core
                    .submit_transcode(&self.jobs, media_id, &decision)
                    .await
                    .map_err(|_| PlaybackPrepareError::Unavailable)?;
                Ok(PlaybackPreparation {
                    method: decision.method,
                    reason: decision.reason,
                    job_id: Some(submission.job.id),
                    output_name: Some(submission.output_name),
                    content_type: Some(submission.content_type),
                })
            }
            PlaybackMethod::Unavailable => Err(PlaybackPrepareError::Incompatible),
        }
    }

    pub fn snapshot(&self, id: MediaJobId) -> Option<MediaJobSnapshot> {
        self.jobs.snapshot(id)
    }
    pub fn cancel(&self, id: MediaJobId) -> bool {
        self.jobs.cancel(id)
    }
    pub async fn open_output(
        &self,
        id: MediaJobId,
        name: &str,
    ) -> Result<MediaJobOutput, MediaJobOutputError> {
        self.jobs.open_output(id, name).await
    }
    pub async fn release(&self, id: MediaJobId) -> bool {
        self.jobs.release(id).await
    }
    pub async fn cancel_and_release(&self, id: MediaJobId) -> bool {
        self.jobs.cancel(id);
        for _ in 0..100 {
            let Some(snapshot) = self.jobs.snapshot(id) else {
                return false;
            };
            if matches!(
                snapshot.state,
                crate::media_jobs::MediaJobState::Completed
                    | crate::media_jobs::MediaJobState::Failed
                    | crate::media_jobs::MediaJobState::Cancelled
            ) {
                return self.jobs.release(id).await;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        false
    }

    #[cfg(test)]
    pub(crate) fn test_jobs(&self) -> &MediaJobManager {
        &self.jobs
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs, process::Command, time::Duration};

    use tempfile::tempdir;

    use super::*;
    use crate::{compatibility::RemuxTarget, media::SubtitleMode, media_jobs::MediaJobState};

    fn set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|v| (*v).to_owned()).collect()
    }

    fn capabilities(
        containers: &[&str],
        video: &[&str],
        target: RemuxTarget,
    ) -> ClientCapabilities {
        ClientCapabilities {
            containers: set(containers),
            video_codecs: set(video),
            audio_codecs: set(&["opus", "aac"]),
            embedded_text_subtitle_codecs: set(&["webvtt", "mov_text"]),
            external_webvtt: true,
            embedded_audio_selection: true,
            bitmap_subtitles: false,
            remux_targets: vec![target],
        }
    }

    async fn terminal(service: &LocalPlaybackService, id: MediaJobId) -> MediaJobState {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                let state = service.snapshot(id).unwrap().state;
                if matches!(
                    state,
                    MediaJobState::Completed | MediaJobState::Failed | MediaJobState::Cancelled
                ) {
                    return state;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn end_to_end_direct_play_remux_transcode_selection_change_and_recovery() {
        if Command::new("ffmpeg").arg("-version").output().is_err()
            || Command::new("ffprobe").arg("-version").output().is_err()
        {
            return;
        }
        let temp = tempdir().unwrap();
        let library = temp.path().join("Videos");
        fs::create_dir(&library).unwrap();
        let subtitle = library.join("captions.srt");
        fs::write(&subtitle, "1\n00:00:00,000 --> 00:00:00,500\nHello\n").unwrap();
        let movie = library.join("movie.mkv");
        assert!(Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=size=96x64:duration=0.8",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.8",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=880:duration=0.8",
                "-i"
            ])
            .arg(&subtitle)
            .args([
                "-map",
                "0:v",
                "-map",
                "1:a",
                "-map",
                "2:a",
                "-map",
                "3:s",
                "-c:v",
                "libvpx-vp9",
                "-c:a",
                "libopus",
                "-c:s",
                "webvtt",
                "-metadata:s:a:0",
                "language=eng",
                "-metadata:s:a:1",
                "language=por"
            ])
            .arg(&movie)
            .status()
            .unwrap()
            .success());
        let core = LocalStreamCore::open(temp.path().join("db.sqlite3")).unwrap();
        let scan = core
            .scan_and_persist_library_with_probe(
                &library,
                tokio_util::sync::CancellationToken::new(),
            )
            .await
            .unwrap();
        let item = scan
            .items
            .iter()
            .find(|item| item.title == "movie")
            .unwrap();
        let audio2 = item.metadata.as_ref().unwrap().audio_tracks[1].id.clone();
        let subtitle_id = item.metadata.as_ref().unwrap().subtitle_tracks[0]
            .id
            .clone();
        core.select_audio_track(&item.id, Some(&audio2)).unwrap();
        core.select_subtitle(&item.id, SubtitleMode::Track, Some(&subtitle_id))
            .unwrap();
        let service = LocalPlaybackService::start(MediaJobConfig {
            work_root: temp.path().join("work"),
            max_concurrent: 1,
            max_queued: 4,
            temporary_byte_quota: 128 * 1024 * 1024,
        })
        .await
        .unwrap();
        let webm_target = RemuxTarget {
            container: "webm".into(),
            video_codecs: set(&["vp9"]),
            audio_codecs: set(&["opus"]),
        };
        let direct = capabilities(&["matroska"], &["vp9"], webm_target.clone());
        assert_eq!(
            service
                .prepare(&core, &item.id, &direct)
                .await
                .unwrap()
                .method,
            PlaybackMethod::DirectPlay
        );
        let remux = capabilities(&["webm"], &["vp9"], webm_target);
        let prepared = service.prepare(&core, &item.id, &remux).await.unwrap();
        assert_eq!(prepared.method, PlaybackMethod::Remux);
        assert_eq!(
            terminal(&service, prepared.job_id.unwrap()).await,
            MediaJobState::Completed
        );
        let mp4_target = RemuxTarget {
            container: "mp4".into(),
            video_codecs: set(&["h264"]),
            audio_codecs: set(&["aac"]),
        };
        let transcode = capabilities(&["mp4"], &["h264"], mp4_target);
        let prepared = service.prepare(&core, &item.id, &transcode).await.unwrap();
        assert_eq!(prepared.method, PlaybackMethod::Transcode);
        assert_eq!(
            terminal(&service, prepared.job_id.unwrap()).await,
            MediaJobState::Completed
        );
        core.select_audio_track(&item.id, None).unwrap();
        let changed = service.prepare(&core, &item.id, &transcode).await.unwrap();
        assert_ne!(changed.job_id, prepared.job_id);
        assert!(service.cancel_and_release(changed.job_id.unwrap()).await);
        assert!(service.snapshot(changed.job_id.unwrap()).is_none());
        let recovered = service.prepare(&core, &item.id, &transcode).await.unwrap();
        assert_ne!(recovered.job_id, changed.job_id);
    }
}
