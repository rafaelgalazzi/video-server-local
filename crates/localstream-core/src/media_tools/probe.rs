use std::{path::Path, time::Duration};

use serde::Deserialize;
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::media::{
    AudioTrack, MediaMetadata, SubtitleKind, SubtitleTrack, TrackMapping, VideoTrack,
};

use super::{ProcessError, ProcessRequest, ProcessRunner};

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const PROBE_OUTPUT_LIMIT: usize = 2 * 1024 * 1024;
const MAX_TRACKS: usize = 256;
const MAX_LABEL_CHARS: usize = 256;

#[derive(Debug, Clone)]
pub(crate) struct ProbeResult {
    pub metadata: MediaMetadata,
    pub mappings: Vec<TrackMapping>,
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("ffprobe could not inspect the media")]
    Process(#[from] ProcessError),
    #[error("ffprobe rejected the media")]
    Rejected,
    #[error("ffprobe returned malformed metadata")]
    Malformed,
    #[error("ffprobe returned too many streams")]
    TooManyStreams,
}

pub(crate) async fn probe_media(
    ffprobe: &Path,
    media_id: &str,
    media_path: &Path,
    cancellation: CancellationToken,
) -> Result<ProbeResult, ProbeError> {
    let request = ProcessRequest::new(ffprobe)
        .args([
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
        ])
        .arg(media_path)
        .timeout(PROBE_TIMEOUT)
        .output_limit(PROBE_OUTPUT_LIMIT);
    let output = ProcessRunner::run(request, cancellation).await?;
    if !output.success {
        return Err(ProbeError::Rejected);
    }
    parse_probe_output(media_id, &output.stdout)
}

#[derive(Deserialize)]
struct RawProbe {
    #[serde(default)]
    streams: Vec<RawStream>,
    format: Option<RawFormat>,
}

#[derive(Deserialize)]
struct RawFormat {
    format_name: Option<String>,
    duration: Option<String>,
}

#[derive(Deserialize)]
struct RawStream {
    index: u32,
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    channels: Option<u16>,
    #[serde(default)]
    tags: RawTags,
    #[serde(default)]
    disposition: RawDisposition,
}

#[derive(Default, Deserialize)]
struct RawTags {
    language: Option<String>,
    title: Option<String>,
}

#[derive(Default, Deserialize)]
struct RawDisposition {
    #[serde(default)]
    default: i32,
    #[serde(default)]
    forced: i32,
}

fn parse_probe_output(media_id: &str, json: &[u8]) -> Result<ProbeResult, ProbeError> {
    let raw: RawProbe = serde_json::from_slice(json).map_err(|_| ProbeError::Malformed)?;
    if raw.streams.len() > MAX_TRACKS {
        return Err(ProbeError::TooManyStreams);
    }
    let format = raw.format.ok_or(ProbeError::Malformed)?;
    let container =
        normalize_container(format.format_name.as_deref()).ok_or(ProbeError::Malformed)?;
    let duration_millis = format.duration.as_deref().and_then(parse_duration);
    let mut video = None;
    let mut audio_tracks = Vec::new();
    let mut subtitle_tracks = Vec::new();
    let mut mappings = Vec::new();

    for stream in raw.streams {
        let Some(codec) = clean_label(stream.codec_name) else {
            continue;
        };
        let Some(kind) = stream.codec_type.as_deref() else {
            continue;
        };
        match kind {
            "video" if video.is_none() => {
                let id = track_id(media_id, kind, stream.index, &codec, None, None);
                video = Some(VideoTrack {
                    id: id.clone(),
                    codec,
                    width: stream.width,
                    height: stream.height,
                });
                mappings.push(TrackMapping {
                    id,
                    source_index: stream.index,
                    kind: "video",
                });
            }
            "audio" => {
                let language = clean_label(stream.tags.language);
                let title = clean_label(stream.tags.title);
                let fingerprint = format!("{}", stream.channels.unwrap_or_default());
                let id = track_id(
                    media_id,
                    kind,
                    stream.index,
                    &codec,
                    language.as_deref(),
                    Some(&format!(
                        "{}:{fingerprint}",
                        title.as_deref().unwrap_or_default()
                    )),
                );
                audio_tracks.push(AudioTrack {
                    id: id.clone(),
                    codec,
                    channels: stream.channels,
                    language,
                    title,
                    is_default: stream.disposition.default == 1,
                });
                mappings.push(TrackMapping {
                    id,
                    source_index: stream.index,
                    kind: "audio",
                });
            }
            "subtitle" => {
                let language = clean_label(stream.tags.language);
                let title = clean_label(stream.tags.title);
                let id = track_id(
                    media_id,
                    kind,
                    stream.index,
                    &codec,
                    language.as_deref(),
                    title.as_deref(),
                );
                let subtitle_kind = classify_subtitle(&codec);
                subtitle_tracks.push(SubtitleTrack {
                    id: id.clone(),
                    codec,
                    language,
                    title,
                    is_default: stream.disposition.default == 1,
                    is_forced: stream.disposition.forced == 1,
                    kind: subtitle_kind,
                });
                mappings.push(TrackMapping {
                    id,
                    source_index: stream.index,
                    kind: "subtitle",
                });
            }
            _ => {}
        }
    }

    Ok(ProbeResult {
        metadata: MediaMetadata {
            container,
            duration_millis,
            video,
            audio_tracks,
            subtitle_tracks,
        },
        mappings,
    })
}

fn track_id(
    media_id: &str,
    kind: &str,
    index: u32,
    codec: &str,
    language: Option<&str>,
    detail: Option<&str>,
) -> String {
    Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!(
            "{media_id}:{kind}:{index}:{codec}:{}:{}",
            language.unwrap_or_default(),
            detail.unwrap_or_default()
        )
        .as_bytes(),
    )
    .to_string()
}

fn normalize_container(value: Option<&str>) -> Option<String> {
    let first = value?.split(',').next()?.trim().to_ascii_lowercase();
    (!first.is_empty()
        && first.len() <= 64
        && first
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
    .then_some(first)
}

fn parse_duration(value: &str) -> Option<u64> {
    let seconds = value.parse::<f64>().ok()?;
    (seconds.is_finite() && seconds >= 0.0 && seconds <= (u64::MAX / 1000) as f64)
        .then(|| (seconds * 1000.0).round() as u64)
}

fn clean_label(value: Option<String>) -> Option<String> {
    let value = value?.trim().to_owned();
    (!value.is_empty()
        && value.chars().count() <= MAX_LABEL_CHARS
        && !value.chars().any(char::is_control))
    .then_some(value)
}

fn classify_subtitle(codec: &str) -> SubtitleKind {
    match codec {
        "ass" | "mov_text" | "srt" | "ssa" | "subrip" | "text" | "webvtt" => SubtitleKind::Text,
        "dvb_subtitle" | "dvd_subtitle" | "hdmv_pgs_subtitle" | "xsub" => SubtitleKind::Bitmap,
        _ => SubtitleKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tokio_util::sync::CancellationToken;

    use super::{parse_probe_output, ProbeError};
    use crate::media::SubtitleKind;

    #[test]
    fn normalizes_dual_audio_and_text_bitmap_subtitles() {
        let json = br#"{"format":{"format_name":"matroska,webm","duration":"62.125"},"streams":[
          {"index":0,"codec_type":"video","codec_name":"h264","width":1920,"height":1080},
          {"index":1,"codec_type":"audio","codec_name":"aac","channels":2,"tags":{"language":"eng","title":"Stereo"},"disposition":{"default":1}},
          {"index":2,"codec_type":"audio","codec_name":"ac3","channels":6,"tags":{"language":"por"}},
          {"index":3,"codec_type":"subtitle","codec_name":"subrip","tags":{"language":"eng"},"disposition":{"forced":1}},
          {"index":4,"codec_type":"subtitle","codec_name":"hdmv_pgs_subtitle","tags":{"language":"por"}}
        ]}"#;
        let result = parse_probe_output("media-id", json).expect("fixture should parse");
        assert_eq!(result.metadata.container, "matroska");
        assert_eq!(result.metadata.duration_millis, Some(62_125));
        assert_eq!(result.metadata.audio_tracks.len(), 2);
        assert!(result.metadata.audio_tracks[0].is_default);
        assert_eq!(result.metadata.subtitle_tracks[0].kind, SubtitleKind::Text);
        assert!(result.metadata.subtitle_tracks[0].is_forced);
        assert_eq!(
            result.metadata.subtitle_tracks[1].kind,
            SubtitleKind::Bitmap
        );
        assert_eq!(result.mappings[4].source_index, 4);
        assert!(result
            .mappings
            .iter()
            .all(|mapping| !mapping.id.contains("media-id")));
    }

    #[test]
    fn tolerates_missing_streams_and_tags() {
        let result = parse_probe_output(
            "id",
            br#"{"format":{"format_name":"mov,mp4,m4a,3gp,3g2,mj2"},"streams":[]}"#,
        )
        .expect("empty stream set is valid");
        assert_eq!(result.metadata.container, "mov");
        assert!(result.metadata.video.is_none());
        assert!(result.metadata.audio_tracks.is_empty());
    }

    #[test]
    fn rejects_malformed_or_missing_format_output() {
        assert!(matches!(
            parse_probe_output("id", b"not json"),
            Err(ProbeError::Malformed)
        ));
        assert!(matches!(
            parse_probe_output("id", br#"{"streams":[]}"#),
            Err(ProbeError::Malformed)
        ));
    }

    #[tokio::test]
    async fn inaccessible_media_returns_a_safe_probe_error() {
        if std::process::Command::new("ffprobe")
            .arg("-version")
            .output()
            .is_err()
        {
            return;
        }
        let error = super::probe_media(
            Path::new("ffprobe"),
            "opaque-media-id",
            Path::new("missing;not-a-command.mkv"),
            CancellationToken::new(),
        )
        .await
        .expect_err("missing media must fail safely");
        assert!(matches!(error, ProbeError::Rejected));
    }
}
