use std::{ffi::OsString, path::PathBuf, time::Duration};

use thiserror::Error;

use crate::{
    compatibility::{PlaybackDecision, PlaybackMethod, SubtitleDelivery},
    media::SubtitleKind,
    media_jobs::{
        JobFailure, MediaJobKey, MediaJobManager, MediaJobSubmission, MediaJobSubmitError,
    },
    media_tools::{ProcessRequest, ProcessRunner},
};

const TIMEOUT: Duration = Duration::from_secs(4 * 60 * 60);
const OUTPUT_LIMIT: usize = 1024 * 1024;
const MIN_RESERVATION: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TranscodeSubtitle {
    pub source_index: u32,
    pub subtitle_ordinal: u32,
    pub kind: SubtitleKind,
}

#[derive(Debug)]
pub(crate) struct TranscodeSource {
    pub media_id: String,
    pub approved_root: PathBuf,
    pub media_path: PathBuf,
    pub source_size_bytes: u64,
    pub video_index: u32,
    pub audio_index: Option<u32>,
    pub subtitle: Option<TranscodeSubtitle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscodeSubmission {
    pub job: MediaJobSubmission,
    pub output_name: &'static str,
    pub content_type: &'static str,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TranscodeError {
    #[error("the requested media does not exist")]
    UnknownMedia,
    #[error("the transcode request is unsupported")]
    UnsupportedInput,
    #[error("the transcode target is unsupported")]
    UnsupportedTarget,
    #[error("the selected track is unavailable")]
    InvalidTrack,
    #[error("the media source is outside the approved library")]
    OutsideApprovedLibrary,
    #[error("the transcode service is unavailable")]
    Unavailable,
    #[error(transparent)]
    Job(#[from] MediaJobSubmitError),
}

struct Profile {
    name: &'static str,
    output_name: &'static str,
    content_type: &'static str,
    format: &'static str,
    video_codec: &'static str,
    audio_codec: &'static str,
    subtitle_codec: &'static str,
}

pub(crate) async fn submit(
    jobs: &MediaJobManager,
    source: TranscodeSource,
    decision: &PlaybackDecision,
    ffmpeg: PathBuf,
) -> Result<TranscodeSubmission, TranscodeError> {
    if decision.method != PlaybackMethod::Transcode {
        return Err(TranscodeError::UnsupportedInput);
    }
    let profile = profile(decision.target_container.as_deref())?;
    validate_subtitle(&source, decision.subtitle_delivery)?;
    let root = tokio::fs::canonicalize(&source.approved_root)
        .await
        .map_err(|_| TranscodeError::Unavailable)?;
    let input = tokio::fs::canonicalize(&source.media_path)
        .await
        .map_err(|_| TranscodeError::Unavailable)?;
    if input == root || !input.starts_with(&root) {
        return Err(TranscodeError::OutsideApprovedLibrary);
    }
    let reservation = source
        .source_size_bytes
        .checked_mul(2)
        .ok_or(TranscodeError::UnsupportedInput)?
        .max(MIN_RESERVATION);
    let subtitle_key = source
        .subtitle
        .map_or_else(|| "-".to_owned(), |s| format!("{}", s.source_index));
    let key = MediaJobKey::new(format!(
        "transcode:{}:{}:{}:{}:{}:{:?}",
        source.media_id,
        profile.name,
        source.video_index,
        source
            .audio_index
            .map_or_else(|| "-".into(), |v| v.to_string()),
        subtitle_key,
        decision.subtitle_delivery
    ))?;
    let output_name = profile.output_name;
    let content_type = profile.content_type;
    let delivery = decision.subtitle_delivery;
    let job = jobs.submit(key, reservation, move |context| async move {
        let output = context.directory().join(output_name);
        let request = build_request(&ffmpeg, &input, &output, &source, &profile, delivery);
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
    Ok(TranscodeSubmission {
        job,
        output_name,
        content_type,
    })
}

fn validate_subtitle(
    source: &TranscodeSource,
    delivery: SubtitleDelivery,
) -> Result<(), TranscodeError> {
    match (source.subtitle, delivery) {
        (None, SubtitleDelivery::Off) | (None, SubtitleDelivery::ExternalWebVtt) => Ok(()),
        (Some(_), SubtitleDelivery::Embedded | SubtitleDelivery::BurnIn) => Ok(()),
        _ => Err(TranscodeError::UnsupportedInput),
    }
}

fn profile(container: Option<&str>) -> Result<Profile, TranscodeError> {
    match container {
        Some(value) if value.eq_ignore_ascii_case("mp4") => Ok(Profile {
            name: "h264-aac",
            output_name: "output.mp4",
            content_type: "video/mp4",
            format: "mp4",
            video_codec: "libx264",
            audio_codec: "aac",
            subtitle_codec: "mov_text",
        }),
        Some(value) if value.eq_ignore_ascii_case("webm") => Ok(Profile {
            name: "vp9-opus",
            output_name: "output.webm",
            content_type: "video/webm",
            format: "webm",
            video_codec: "libvpx-vp9",
            audio_codec: "libopus",
            subtitle_codec: "webvtt",
        }),
        _ => Err(TranscodeError::UnsupportedTarget),
    }
}

fn build_request(
    ffmpeg: &std::path::Path,
    input: &std::path::Path,
    output: &std::path::Path,
    source: &TranscodeSource,
    profile: &Profile,
    delivery: SubtitleDelivery,
) -> ProcessRequest {
    let mut args = vec![
        OsString::from("-v"),
        OsString::from("error"),
        OsString::from("-nostdin"),
        OsString::from("-y"),
        OsString::from("-i"),
        input.as_os_str().to_owned(),
    ];
    let burn_in = delivery == SubtitleDelivery::BurnIn;
    if burn_in {
        let subtitle = source.subtitle.expect("validated burn-in subtitle");
        let filter = match subtitle.kind {
            SubtitleKind::Bitmap => format!(
                "[0:{}][0:{}]overlay",
                source.video_index, subtitle.source_index
            ),
            SubtitleKind::Text => format!(
                "subtitles=filename='{}':si={}",
                escape_filter_path(input),
                subtitle.subtitle_ordinal
            ),
            SubtitleKind::Unknown => String::new(),
        };
        args.extend([OsString::from("-filter_complex"), OsString::from(filter)]);
    } else {
        args.extend([
            OsString::from("-map"),
            OsString::from(format!("0:{}", source.video_index)),
        ]);
    }
    if let Some(index) = source.audio_index {
        args.extend([OsString::from("-map"), OsString::from(format!("0:{index}"))]);
    }
    if delivery == SubtitleDelivery::Embedded {
        let subtitle = source.subtitle.expect("validated embedded subtitle");
        args.extend([
            OsString::from("-map"),
            OsString::from(format!("0:{}", subtitle.source_index)),
        ]);
    }
    args.extend([
        OsString::from("-c:v"),
        OsString::from(profile.video_codec),
        OsString::from("-preset"),
        OsString::from("veryfast"),
        OsString::from("-crf"),
        OsString::from("28"),
    ]);
    if source.audio_index.is_some() {
        args.extend([
            OsString::from("-c:a"),
            OsString::from(profile.audio_codec),
            OsString::from("-b:a"),
            OsString::from("128k"),
        ]);
    }
    if delivery == SubtitleDelivery::Embedded {
        args.extend([
            OsString::from("-c:s"),
            OsString::from(profile.subtitle_codec),
        ]);
    }
    if profile.format == "mp4" {
        args.extend([
            OsString::from("-pix_fmt"),
            OsString::from("yuv420p"),
            OsString::from("-movflags"),
            OsString::from("+faststart"),
        ]);
    }
    args.extend([
        OsString::from("-f"),
        OsString::from(profile.format),
        output.as_os_str().to_owned(),
    ]);
    ProcessRequest::new(ffmpeg)
        .args(args)
        .timeout(TIMEOUT)
        .output_limit(OUTPUT_LIMIT)
}

fn escape_filter_path(path: &std::path::Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(|character| {
            if matches!(character, '\\' | ':' | '\'' | ',' | ';' | '[' | ']') {
                vec!['\\', character]
            } else {
                vec![character]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{fs, process::Command};

    use tempfile::tempdir;

    use super::*;
    use crate::{
        compatibility::DecisionReason,
        media_jobs::{MediaJobConfig, MediaJobState},
    };

    fn decision(delivery: SubtitleDelivery) -> PlaybackDecision {
        PlaybackDecision {
            method: PlaybackMethod::Transcode,
            reason: DecisionReason::VideoCodecUnsupported,
            target_container: Some("mp4".to_owned()),
            selected_audio_track_id: Some("audio-2".to_owned()),
            selected_subtitle_track_id: (delivery != SubtitleDelivery::Off)
                .then(|| "sub-1".to_owned()),
            subtitle_delivery: delivery,
        }
    }

    async fn wait(jobs: &MediaJobManager, id: crate::media_jobs::MediaJobId) -> MediaJobState {
        tokio::time::timeout(Duration::from_secs(20), async {
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

    fn tools_available() -> bool {
        Command::new("ffmpeg").arg("-version").output().is_ok()
            && Command::new("ffprobe").arg("-version").output().is_ok()
    }

    fn fixture(root: &std::path::Path) -> PathBuf {
        let subtitle = root.join("captions.srt");
        fs::write(
            &subtitle,
            "1\n00:00:00,000 --> 00:00:00,500\nVisible text\n",
        )
        .unwrap();
        let input = root.join("source.mkv");
        let status = Command::new("ffmpeg")
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
                "-i",
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
                "mpeg4",
                "-c:a",
                "aac",
                "-c:s",
                "srt",
                "-metadata:s:a:0",
                "language=eng",
                "-metadata:s:a:1",
                "language=por",
            ])
            .arg(&input)
            .status()
            .unwrap();
        assert!(status.success());
        input
    }

    fn source(root: &std::path::Path, input: &std::path::Path) -> TranscodeSource {
        TranscodeSource {
            media_id: "opaque".to_owned(),
            approved_root: root.to_owned(),
            media_path: input.to_owned(),
            source_size_bytes: fs::metadata(input).unwrap().len(),
            video_index: 0,
            audio_index: Some(2),
            subtitle: Some(TranscodeSubtitle {
                source_index: 3,
                subtitle_ordinal: 0,
                kind: SubtitleKind::Text,
            }),
        }
    }

    #[tokio::test]
    async fn creates_playable_mp4_with_selected_audio_and_converted_subtitle() {
        if !tools_available() {
            return;
        }
        let temp = tempdir().unwrap();
        let root = temp.path().join("library");
        fs::create_dir(&root).unwrap();
        let input = fixture(&root);
        let jobs = MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("work"),
            max_concurrent: 1,
            max_queued: 2,
            temporary_byte_quota: 64 * 1024 * 1024,
        })
        .await
        .unwrap();
        let submission = submit(
            &jobs,
            source(&root, &input),
            &decision(SubtitleDelivery::Embedded),
            PathBuf::from("ffmpeg"),
        )
        .await
        .unwrap();
        assert_eq!(
            wait(&jobs, submission.job.id).await,
            MediaJobState::Completed
        );
        let output = jobs.test_output_path(submission.job.id, submission.output_name);
        let probe = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "stream=codec_name,codec_type:stream_tags=language",
                "-of",
                "compact=p=0:nk=1",
            ])
            .arg(output)
            .output()
            .unwrap();
        let streams = String::from_utf8(probe.stdout).unwrap();
        assert!(streams.contains("h264|video"));
        assert!(streams.contains("aac|audio|por"));
        assert!(streams.contains("mov_text|subtitle"));
        assert!(!streams.contains("eng"));
    }

    #[tokio::test]
    async fn burns_text_subtitle_without_retaining_subtitle_stream() {
        if !tools_available() {
            return;
        }
        let temp = tempdir().unwrap();
        let root = temp.path().join("library");
        fs::create_dir(&root).unwrap();
        let input = fixture(&root);
        let jobs = MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("work"),
            max_concurrent: 1,
            max_queued: 1,
            temporary_byte_quota: 64 * 1024 * 1024,
        })
        .await
        .unwrap();
        let submission = submit(
            &jobs,
            source(&root, &input),
            &decision(SubtitleDelivery::BurnIn),
            PathBuf::from("ffmpeg"),
        )
        .await
        .unwrap();
        assert_eq!(
            wait(&jobs, submission.job.id).await,
            MediaJobState::Completed
        );
        let probe = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "s",
                "-show_entries",
                "stream=index",
                "-of",
                "csv=p=0",
            ])
            .arg(jobs.test_output_path(submission.job.id, submission.output_name))
            .output()
            .unwrap();
        assert!(probe.stdout.is_empty());
    }

    #[tokio::test]
    async fn rejects_direct_play_unknown_target_invalid_modes_and_quota() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("library");
        fs::create_dir(&root).unwrap();
        let input = root.join("source.mkv");
        fs::write(&input, b"source").unwrap();
        let jobs = MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("work"),
            max_concurrent: 1,
            max_queued: 1,
            temporary_byte_quota: MIN_RESERVATION - 1,
        })
        .await
        .unwrap();
        let mut request = decision(SubtitleDelivery::Off);
        request.method = PlaybackMethod::DirectPlay;
        let mut no_subtitle = source(&root, &input);
        no_subtitle.subtitle = None;
        assert_eq!(
            submit(&jobs, no_subtitle, &request, PathBuf::from("ffmpeg"))
                .await
                .unwrap_err(),
            TranscodeError::UnsupportedInput
        );
        let mut unknown = decision(SubtitleDelivery::Off);
        unknown.target_container = Some("avi".into());
        let mut no_subtitle = source(&root, &input);
        no_subtitle.subtitle = None;
        assert_eq!(
            submit(&jobs, no_subtitle, &unknown, PathBuf::from("ffmpeg"))
                .await
                .unwrap_err(),
            TranscodeError::UnsupportedTarget
        );
        let mut no_subtitle = source(&root, &input);
        no_subtitle.subtitle = None;
        assert_eq!(
            submit(
                &jobs,
                no_subtitle,
                &decision(SubtitleDelivery::Off),
                PathBuf::from("ffmpeg")
            )
            .await
            .unwrap_err(),
            TranscodeError::Job(MediaJobSubmitError::InvalidReservation)
        );
    }
}
