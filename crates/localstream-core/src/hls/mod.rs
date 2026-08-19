use std::{
    collections::HashMap,
    ffi::OsString,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::Serialize;
use thiserror::Error;

use crate::{
    media_jobs::{
        JobFailure, MediaJobId, MediaJobKey, MediaJobManager, MediaJobOutput, MediaJobOutputError,
        MediaJobSnapshot, MediaJobState, MediaJobSubmission, MediaJobSubmitError,
        SubmitDisposition,
    },
    media_tools::{ProcessRequest, ProcessRunner},
};

const HLS_TIMEOUT: Duration = Duration::from_secs(4 * 60 * 60);
const PROCESS_OUTPUT_LIMIT: usize = 1024 * 1024;
const OUTPUT_OVERHEAD_BYTES: u64 = 16 * 1024 * 1024;
const HLS_SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);
pub const PLAYLIST_NAME: &str = "index.m3u8";

#[derive(Debug)]
pub(crate) struct HlsSource {
    pub media_id: String,
    pub approved_root: PathBuf,
    pub media_path: PathBuf,
    pub source_size_bytes: u64,
    pub video_index: u32,
    pub video_codec: String,
    pub audio_index: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HlsVideoMode {
    Copy,
    Transcode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HlsSubmission {
    pub job_id: MediaJobId,
    pub playlist_name: &'static str,
    pub video_mode: HlsVideoMode,
}

#[derive(Clone)]
pub struct HlsSessionService {
    playback: crate::playback::LocalPlaybackService,
    owners: Arc<Mutex<HashMap<MediaJobId, String>>>,
}

impl HlsSessionService {
    pub fn new(playback: crate::playback::LocalPlaybackService) -> Self {
        Self {
            playback,
            owners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn prepare(
        &self,
        core: &crate::LocalStreamCore,
        owner_id: &str,
        media_id: &str,
    ) -> Result<HlsSubmission, HlsError> {
        let submission = self.playback.prepare_hls(core, media_id).await?;
        lock(&self.owners).insert(submission.job_id, owner_id.to_owned());
        let owners = Arc::clone(&self.owners);
        let id = submission.job_id;
        tokio::spawn(async move {
            tokio::time::sleep(HLS_SESSION_TTL).await;
            lock(&owners).remove(&id);
        });
        Ok(submission)
    }

    pub fn snapshot(&self, owner_id: &str, id: MediaJobId) -> Option<MediaJobSnapshot> {
        self.owns(owner_id, id)
            .then(|| self.playback.hls_snapshot(id))
            .flatten()
    }

    pub async fn open_asset(
        &self,
        owner_id: &str,
        id: MediaJobId,
        name: &str,
    ) -> Result<MediaJobOutput, MediaJobOutputError> {
        if !self.owns(owner_id, id) {
            return Err(MediaJobOutputError::UnknownJob);
        }
        self.playback.open_hls_asset(id, name).await
    }

    pub async fn cancel_and_release(&self, owner_id: &str, id: MediaJobId) -> bool {
        if !self.owns(owner_id, id) {
            return false;
        }
        self.playback.cancel_hls(id);
        for _ in 0..100 {
            let terminal = self.playback.hls_snapshot(id).map_or(true, |snapshot| {
                matches!(
                    snapshot.state,
                    MediaJobState::Completed | MediaJobState::Failed | MediaJobState::Cancelled
                )
            });
            if terminal {
                lock(&self.owners).remove(&id);
                return self.playback.release_hls(id).await;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        false
    }

    fn owns(&self, owner_id: &str, id: MediaJobId) -> bool {
        lock(&self.owners)
            .get(&id)
            .is_some_and(|owner| owner == owner_id)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HlsError {
    #[error("the requested media does not exist")]
    UnknownMedia,
    #[error("the media metadata is unavailable")]
    UnsupportedInput,
    #[error("the selected audio track is unavailable")]
    InvalidTrack,
    #[error("the media source is outside the approved library")]
    OutsideApprovedLibrary,
    #[error("the HLS service is unavailable")]
    Unavailable,
    #[error(transparent)]
    Job(#[from] MediaJobSubmitError),
}

pub(crate) async fn submit(
    jobs: &MediaJobManager,
    source: HlsSource,
    ffmpeg: PathBuf,
) -> Result<HlsSubmission, HlsError> {
    let root = tokio::fs::canonicalize(&source.approved_root)
        .await
        .map_err(|_| HlsError::Unavailable)?;
    let input = tokio::fs::canonicalize(&source.media_path)
        .await
        .map_err(|_| HlsError::Unavailable)?;
    if input == root || !input.starts_with(&root) {
        return Err(HlsError::OutsideApprovedLibrary);
    }
    let video_mode = if source.video_codec.eq_ignore_ascii_case("h264") {
        HlsVideoMode::Copy
    } else {
        HlsVideoMode::Transcode
    };
    let multiplier = if video_mode == HlsVideoMode::Copy {
        1
    } else {
        2
    };
    let reservation = source
        .source_size_bytes
        .checked_mul(multiplier)
        .and_then(|size| size.checked_add(OUTPUT_OVERHEAD_BYTES))
        .ok_or(HlsError::UnsupportedInput)?;
    let key = MediaJobKey::new(format!(
        "hls:{}:{}:{}:{}",
        source.media_id,
        source.video_index,
        source
            .audio_index
            .map_or_else(|| "-".to_owned(), |value| value.to_string()),
        match video_mode {
            HlsVideoMode::Copy => "copy",
            HlsVideoMode::Transcode => "h264",
        }
    ))?;
    let job = jobs.submit(key, reservation, move |context| async move {
        let playlist = context.directory().join(PLAYLIST_NAME);
        let segment_pattern = context.directory().join("segment-%06d.ts");
        let request = ProcessRequest::new(&ffmpeg)
            .args(build_arguments(
                &input,
                &playlist,
                &segment_pattern,
                source.video_index,
                source.audio_index,
                video_mode,
            ))
            .timeout(HLS_TIMEOUT)
            .output_limit(PROCESS_OUTPUT_LIMIT);
        let result = ProcessRunner::run(request, context.cancellation())
            .await
            .map_err(|_| JobFailure)?;
        if !result.success {
            return Err(JobFailure);
        }
        let metadata = tokio::fs::metadata(&playlist)
            .await
            .map_err(|_| JobFailure)?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(JobFailure);
        }
        context.progress().set_permille(999);
        Ok(())
    })?;
    if job.disposition == SubmitDisposition::Admitted {
        schedule_expiry(jobs.clone(), job.id);
    }
    Ok(submission(job, video_mode))
}

fn schedule_expiry(jobs: MediaJobManager, id: MediaJobId) {
    tokio::spawn(async move {
        tokio::time::sleep(HLS_SESSION_TTL).await;
        jobs.cancel(id);
        for _ in 0..100 {
            let terminal = jobs.snapshot(id).map_or(true, |snapshot| {
                matches!(
                    snapshot.state,
                    MediaJobState::Completed | MediaJobState::Failed | MediaJobState::Cancelled
                )
            });
            if terminal {
                let _ = jobs.release(id).await;
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
}

fn submission(job: MediaJobSubmission, video_mode: HlsVideoMode) -> HlsSubmission {
    HlsSubmission {
        job_id: job.id,
        playlist_name: PLAYLIST_NAME,
        video_mode,
    }
}

pub fn is_asset_name(name: &str) -> bool {
    name == PLAYLIST_NAME
        || name
            .strip_prefix("segment-")
            .and_then(|value| value.strip_suffix(".ts"))
            .is_some_and(|digits| {
                digits.len() == 6 && digits.bytes().all(|byte| byte.is_ascii_digit())
            })
}

pub async fn open_asset(
    jobs: &MediaJobManager,
    id: MediaJobId,
    name: &str,
) -> Result<MediaJobOutput, MediaJobOutputError> {
    if !is_asset_name(name) {
        return Err(MediaJobOutputError::InvalidName);
    }
    jobs.open_progressive_output(id, name).await
}

pub fn snapshot(jobs: &MediaJobManager, id: MediaJobId) -> Option<MediaJobSnapshot> {
    jobs.snapshot(id)
}

pub fn cancel(jobs: &MediaJobManager, id: MediaJobId) -> bool {
    jobs.cancel(id)
}

pub async fn release(jobs: &MediaJobManager, id: MediaJobId) -> bool {
    jobs.release(id).await
}

fn build_arguments(
    input: &std::path::Path,
    playlist: &std::path::Path,
    segment_pattern: &std::path::Path,
    video_index: u32,
    audio_index: Option<u32>,
    video_mode: HlsVideoMode,
) -> Vec<OsString> {
    let mut arguments = vec![
        OsString::from("-v"),
        OsString::from("error"),
        OsString::from("-nostdin"),
        OsString::from("-y"),
    ];
    arguments.extend([
        OsString::from("-i"),
        input.as_os_str().to_owned(),
        OsString::from("-map"),
        OsString::from(format!("0:{video_index}")),
    ]);
    if let Some(index) = audio_index {
        arguments.extend([OsString::from("-map"), OsString::from(format!("0:{index}"))]);
    }
    arguments.extend([OsString::from("-sn"), OsString::from("-dn")]);
    match video_mode {
        HlsVideoMode::Copy => {
            arguments.extend([OsString::from("-c:v"), OsString::from("copy")]);
        }
        HlsVideoMode::Transcode => arguments.extend([
            OsString::from("-c:v"),
            OsString::from("libx264"),
            OsString::from("-preset"),
            OsString::from("veryfast"),
            OsString::from("-crf"),
            OsString::from("23"),
            OsString::from("-pix_fmt"),
            OsString::from("yuv420p"),
            OsString::from("-force_key_frames"),
            OsString::from("expr:gte(t,n_forced*4)"),
        ]),
    }
    if audio_index.is_some() {
        arguments.extend([
            OsString::from("-c:a"),
            OsString::from("aac"),
            OsString::from("-b:a"),
            OsString::from("192k"),
            OsString::from("-ac"),
            OsString::from("2"),
        ]);
    }
    arguments.extend([
        OsString::from("-f"),
        OsString::from("hls"),
        OsString::from("-hls_time"),
        OsString::from("4"),
        OsString::from("-hls_list_size"),
        OsString::from("0"),
        OsString::from("-hls_playlist_type"),
        OsString::from("event"),
        OsString::from("-hls_flags"),
        OsString::from("independent_segments+temp_file"),
        OsString::from("-hls_segment_filename"),
        segment_pattern.as_os_str().to_owned(),
        playlist.as_os_str().to_owned(),
    ]);
    arguments
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_fixed_hls_asset_names() {
        assert!(is_asset_name("index.m3u8"));
        assert!(is_asset_name("segment-000001.ts"));
        for invalid in [
            "../index.m3u8",
            "segment-1.ts",
            "segment-000001.ts.exe",
            "segment-abcdef.ts",
            "other.m3u8",
        ] {
            assert!(!is_asset_name(invalid));
        }
    }

    #[test]
    fn h264_profile_copies_video_and_converts_only_audio() {
        let arguments = build_arguments(
            std::path::Path::new("movie.mkv"),
            std::path::Path::new("index.m3u8"),
            std::path::Path::new("segment-%06d.ts"),
            0,
            Some(2),
            HlsVideoMode::Copy,
        );
        let arguments: Vec<_> = arguments
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect();
        assert!(arguments.windows(2).any(|pair| pair == ["-c:v", "copy"]));
        assert!(arguments.windows(2).any(|pair| pair == ["-c:a", "aac"]));
        assert!(!arguments.iter().any(|value| value == "libx264"));
        assert!(arguments.iter().any(|value| value == "0:2"));
    }
}
