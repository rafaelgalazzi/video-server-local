use std::{ffi::OsString, path::PathBuf, time::Duration};

use thiserror::Error;

use crate::{
    compatibility::{PlaybackDecision, PlaybackMethod},
    media_jobs::{
        JobFailure, MediaJobKey, MediaJobManager, MediaJobSubmission, MediaJobSubmitError,
    },
    media_tools::{ProcessRequest, ProcessRunner},
};

const REMUX_TIMEOUT: Duration = Duration::from_secs(2 * 60 * 60);
const PROCESS_OUTPUT_LIMIT: usize = 1024 * 1024;
const OUTPUT_OVERHEAD_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
pub(crate) struct RemuxSource {
    pub media_id: String,
    pub approved_root: PathBuf,
    pub media_path: PathBuf,
    pub source_size_bytes: u64,
    pub video_index: u32,
    pub audio_index: Option<u32>,
    pub subtitle_index: Option<u32>,
    pub subtitle_codec: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemuxSubmission {
    pub job: MediaJobSubmission,
    pub output_name: &'static str,
    pub content_type: &'static str,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RemuxError {
    #[error("the requested media does not exist")]
    UnknownMedia,
    #[error("the remux request is not compatible with the selected source")]
    UnsupportedInput,
    #[error("the remux target container is unsupported")]
    UnsupportedTarget,
    #[error("the selected track is unavailable")]
    InvalidTrack,
    #[error("the selected subtitle mode requires transcoding")]
    UnsupportedSubtitleDelivery,
    #[error("the media source is outside the approved library")]
    OutsideApprovedLibrary,
    #[error("the remux service is unavailable")]
    Unavailable,
    #[error(transparent)]
    Job(#[from] MediaJobSubmitError),
}

struct RemuxTarget {
    output_name: &'static str,
    content_type: &'static str,
    format: &'static str,
    faststart: bool,
}

impl RemuxTarget {
    fn supports_subtitle(&self, codec: Option<&str>) -> bool {
        match self.format {
            "mp4" => codec.is_some_and(|codec| {
                codec.eq_ignore_ascii_case("mov_text") || codec.eq_ignore_ascii_case("tx3g")
            }),
            "webm" => codec.is_some_and(|codec| codec.eq_ignore_ascii_case("webvtt")),
            _ => false,
        }
    }
}

pub(crate) async fn submit(
    jobs: &MediaJobManager,
    source: RemuxSource,
    decision: &PlaybackDecision,
    ffmpeg: PathBuf,
) -> Result<RemuxSubmission, RemuxError> {
    if decision.method != PlaybackMethod::Remux {
        return Err(RemuxError::UnsupportedInput);
    }
    let target = target(decision.target_container.as_deref())?;
    if source.subtitle_index.is_some()
        && !target.supports_subtitle(source.subtitle_codec.as_deref())
    {
        return Err(RemuxError::UnsupportedInput);
    }
    let root = tokio::fs::canonicalize(&source.approved_root)
        .await
        .map_err(|_| RemuxError::Unavailable)?;
    let input = tokio::fs::canonicalize(&source.media_path)
        .await
        .map_err(|_| RemuxError::Unavailable)?;
    if input == root || !input.starts_with(&root) {
        return Err(RemuxError::OutsideApprovedLibrary);
    }
    let reservation = source
        .source_size_bytes
        .checked_add(OUTPUT_OVERHEAD_BYTES)
        .ok_or(RemuxError::UnsupportedInput)?;
    let key = MediaJobKey::new(format!(
        "remux:{}:{}:{}:{}:{}",
        source.media_id,
        target.format,
        source.video_index,
        source
            .audio_index
            .map_or_else(|| "-".to_owned(), |v| v.to_string()),
        source
            .subtitle_index
            .map_or_else(|| "-".to_owned(), |v| v.to_string())
    ))?;
    let output_name = target.output_name;
    let content_type = target.content_type;
    let job = jobs.submit(key, reservation, move |context| async move {
        let output = context.directory().join(output_name);
        let request = build_request(&ffmpeg, &input, &output, &source, &target);
        let result = ProcessRunner::run(request, context.cancellation())
            .await
            .map_err(|_| JobFailure)?;
        if !result.success {
            return Err(JobFailure);
        }
        let metadata = tokio::fs::metadata(output).await.map_err(|_| JobFailure)?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(JobFailure);
        }
        context.progress().set_permille(999);
        Ok(())
    })?;
    Ok(RemuxSubmission {
        job,
        output_name,
        content_type,
    })
}

fn target(container: Option<&str>) -> Result<RemuxTarget, RemuxError> {
    match container {
        Some(value) if value.eq_ignore_ascii_case("mp4") => Ok(RemuxTarget {
            output_name: "output.mp4",
            content_type: "video/mp4",
            format: "mp4",
            faststart: true,
        }),
        Some(value) if value.eq_ignore_ascii_case("webm") => Ok(RemuxTarget {
            output_name: "output.webm",
            content_type: "video/webm",
            format: "webm",
            faststart: false,
        }),
        _ => Err(RemuxError::UnsupportedTarget),
    }
}

fn build_request(
    ffmpeg: &std::path::Path,
    input: &std::path::Path,
    output: &std::path::Path,
    source: &RemuxSource,
    target: &RemuxTarget,
) -> ProcessRequest {
    let mut arguments = vec![
        OsString::from("-v"),
        OsString::from("error"),
        OsString::from("-nostdin"),
        OsString::from("-y"),
        OsString::from("-i"),
        input.as_os_str().to_os_string(),
        OsString::from("-map"),
        OsString::from(format!("0:{}", source.video_index)),
    ];
    if let Some(index) = source.audio_index {
        arguments.extend([OsString::from("-map"), OsString::from(format!("0:{index}"))]);
    }
    if let Some(index) = source.subtitle_index {
        arguments.extend([OsString::from("-map"), OsString::from(format!("0:{index}"))]);
    }
    arguments.extend([OsString::from("-c"), OsString::from("copy")]);
    if target.faststart {
        arguments.extend([OsString::from("-movflags"), OsString::from("+faststart")]);
    }
    arguments.extend([
        OsString::from("-f"),
        OsString::from(target.format),
        output.as_os_str().to_os_string(),
    ]);
    ProcessRequest::new(ffmpeg)
        .args(arguments)
        .timeout(REMUX_TIMEOUT)
        .output_limit(PROCESS_OUTPUT_LIMIT)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs, process::Command, sync::Arc, time::Duration};

    use tempfile::tempdir;
    use tokio::sync::Notify;

    use super::*;
    use crate::{
        compatibility::{DecisionReason, SubtitleDelivery},
        media_jobs::{MediaJobConfig, MediaJobState},
    };

    fn decision(target: &str, audio: Option<&str>, subtitle: Option<&str>) -> PlaybackDecision {
        PlaybackDecision {
            method: PlaybackMethod::Remux,
            reason: DecisionReason::ContainerUnsupported,
            target_container: Some(target.to_owned()),
            selected_audio_track_id: audio.map(str::to_owned),
            selected_subtitle_track_id: subtitle.map(str::to_owned),
            subtitle_delivery: if subtitle.is_some() {
                SubtitleDelivery::Embedded
            } else {
                SubtitleDelivery::Off
            },
        }
    }

    async fn wait_terminal(
        jobs: &MediaJobManager,
        id: crate::media_jobs::MediaJobId,
    ) -> MediaJobState {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let state = jobs.snapshot(id).unwrap().state;
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
    async fn remuxes_selected_second_audio_and_text_subtitle_when_ffmpeg_is_available() {
        if Command::new("ffmpeg").arg("-version").output().is_err()
            || Command::new("ffprobe").arg("-version").output().is_err()
        {
            return;
        }
        let temp = tempdir().unwrap();
        let library = temp.path().join("library");
        fs::create_dir(&library).unwrap();
        let subtitles = library.join("captions.srt");
        fs::write(&subtitles, "1\n00:00:00,000 --> 00:00:00,150\nHello\n").unwrap();
        let input = library.join("dual.mkv");
        let status = Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=size=64x64:duration=0.2",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.2",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=880:duration=0.2",
                "-i",
            ])
            .arg(&subtitles)
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
                "vp9",
                "-c:a",
                "libopus",
                "-c:s",
                "webvtt",
                "-metadata:s:a:0",
                "language=eng",
                "-metadata:s:a:1",
                "language=por",
            ])
            .arg(&input)
            .status()
            .unwrap();
        assert!(status.success());
        let jobs = MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("work"),
            max_concurrent: 1,
            max_queued: 2,
            temporary_byte_quota: 10 * 1024 * 1024,
        })
        .await
        .unwrap();
        let source = RemuxSource {
            media_id: "opaque".into(),
            approved_root: library,
            media_path: input.clone(),
            source_size_bytes: fs::metadata(&input).unwrap().len(),
            video_index: 0,
            audio_index: Some(2),
            subtitle_index: Some(3),
            subtitle_codec: Some("webvtt".to_owned()),
        };
        let submission = submit(
            &jobs,
            source,
            &decision("webm", Some("audio-2"), Some("sub-1")),
            PathBuf::from("ffmpeg"),
        )
        .await
        .unwrap();
        assert_eq!(
            wait_terminal(&jobs, submission.job.id).await,
            MediaJobState::Completed
        );
        let output = jobs.test_output_path(submission.job.id, submission.output_name);
        let probe = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "stream=codec_type",
                "-of",
                "csv=p=0",
            ])
            .arg(output)
            .output()
            .unwrap();
        assert!(probe.status.success());
        let types = String::from_utf8(probe.stdout).unwrap();
        assert_eq!(
            types.lines().collect::<BTreeSet<_>>(),
            BTreeSet::from(["audio", "subtitle", "video"])
        );
        assert_eq!(types.lines().filter(|kind| *kind == "audio").count(), 1);
        let language = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "a:0",
                "-show_entries",
                "stream_tags=language",
                "-of",
                "default=nw=1:nk=1",
            ])
            .arg(jobs.test_output_path(submission.job.id, submission.output_name))
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(language.stdout).unwrap().trim(), "por");
    }

    #[tokio::test]
    async fn rejects_non_remux_unknown_target_and_outside_source() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir(&root).unwrap();
        let outside = temp.path().join("outside.mkv");
        fs::write(&outside, b"media").unwrap();
        let jobs = MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("work"),
            max_concurrent: 1,
            max_queued: 1,
            temporary_byte_quota: 1024 * 1024,
        })
        .await
        .unwrap();
        let source = || RemuxSource {
            media_id: "id".into(),
            approved_root: root.clone(),
            media_path: outside.clone(),
            source_size_bytes: 5,
            video_index: 0,
            audio_index: None,
            subtitle_index: None,
            subtitle_codec: None,
        };
        assert_eq!(
            submit(
                &jobs,
                source(),
                &decision("avi", None, None),
                PathBuf::from("ffmpeg")
            )
            .await
            .unwrap_err(),
            RemuxError::UnsupportedTarget
        );
        assert_eq!(
            submit(
                &jobs,
                source(),
                &decision("mp4", None, None),
                PathBuf::from("ffmpeg")
            )
            .await
            .unwrap_err(),
            RemuxError::OutsideApprovedLibrary
        );
        let mut direct = decision("mp4", None, None);
        direct.method = PlaybackMethod::DirectPlay;
        assert_eq!(
            submit(&jobs, source(), &direct, PathBuf::from("ffmpeg"))
                .await
                .unwrap_err(),
            RemuxError::UnsupportedInput
        );
    }

    #[tokio::test]
    async fn queued_remux_can_be_cancelled_and_respects_temporary_quota() {
        let temp = tempdir().unwrap();
        let library = temp.path().join("root");
        fs::create_dir(&library).unwrap();
        let input = library.join("input.mkv");
        fs::write(&input, b"not reached").unwrap();
        let jobs = MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("work"),
            max_concurrent: 1,
            max_queued: 2,
            temporary_byte_quota: 2 * 1024 * 1024,
        })
        .await
        .unwrap();
        let gate = Arc::new(Notify::new());
        let worker_gate = Arc::clone(&gate);
        jobs.submit(
            MediaJobKey::new("occupy-worker").unwrap(),
            1,
            move |_| async move {
                worker_gate.notified().await;
                Ok(())
            },
        )
        .unwrap();
        let make_source = || RemuxSource {
            media_id: "id".into(),
            approved_root: library.clone(),
            media_path: input.clone(),
            source_size_bytes: 1,
            video_index: 0,
            audio_index: None,
            subtitle_index: None,
            subtitle_codec: None,
        };
        let submission = submit(
            &jobs,
            make_source(),
            &decision("mp4", None, None),
            PathBuf::from("ffmpeg"),
        )
        .await
        .unwrap();
        assert!(jobs.cancel(submission.job.id));
        gate.notify_one();
        assert_eq!(
            wait_terminal(&jobs, submission.job.id).await,
            MediaJobState::Cancelled
        );

        let small_jobs = MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("small"),
            max_concurrent: 1,
            max_queued: 1,
            temporary_byte_quota: 1024 * 1024,
        })
        .await
        .unwrap();
        assert_eq!(
            submit(
                &small_jobs,
                make_source(),
                &decision("mp4", None, None),
                PathBuf::from("ffmpeg")
            )
            .await
            .unwrap_err(),
            RemuxError::Job(MediaJobSubmitError::InvalidReservation)
        );
    }
}
