use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::media::{AudioTrack, MediaItem, SubtitleKind, SubtitleMode, SubtitleTrack};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    pub containers: BTreeSet<String>,
    pub video_codecs: BTreeSet<String>,
    pub audio_codecs: BTreeSet<String>,
    pub embedded_text_subtitle_codecs: BTreeSet<String>,
    pub external_webvtt: bool,
    pub embedded_audio_selection: bool,
    pub bitmap_subtitles: bool,
    pub remux_targets: Vec<RemuxTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemuxTarget {
    pub container: String,
    pub video_codecs: BTreeSet<String>,
    pub audio_codecs: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMethod {
    DirectPlay,
    Remux,
    Transcode,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReason {
    DirectPlayCompatible,
    MetadataUnavailable,
    VideoStreamMissing,
    SelectedAudioTrackUnavailable,
    SelectedSubtitleTrackUnavailable,
    BitmapSubtitleRequiresBurnIn,
    SubtitleFormatUnsupported,
    VideoCodecUnsupported,
    AudioCodecUnsupported,
    AudioSelectionRequiresRemux,
    ContainerUnsupported,
    NoCompatibleRemuxTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleDelivery {
    Off,
    ExternalWebVtt,
    Embedded,
    BurnIn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackDecision {
    pub method: PlaybackMethod,
    pub reason: DecisionReason,
    pub target_container: Option<String>,
    pub selected_audio_track_id: Option<String>,
    pub selected_subtitle_track_id: Option<String>,
    pub subtitle_delivery: SubtitleDelivery,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompatibilityError {
    #[error("the requested media does not exist in the current library")]
    UnknownMedia,
    #[error("the compatibility service is unavailable")]
    Unavailable,
}

pub fn decide_playback(item: &MediaItem, client: &ClientCapabilities) -> PlaybackDecision {
    let Some(metadata) = &item.metadata else {
        return unavailable(DecisionReason::MetadataUnavailable);
    };
    let Some(video) = &metadata.video else {
        return unavailable(DecisionReason::VideoStreamMissing);
    };
    let selected_audio = match effective_audio(item) {
        Ok(track) => track,
        Err(reason) => return unavailable(reason),
    };
    let selected_subtitle = match effective_subtitle(item) {
        Ok(track) => track,
        Err(reason) => return unavailable(reason),
    };
    let audio_id = selected_audio.map(|track| track.id.clone());
    let subtitle_id = selected_subtitle.map(|track| track.id.clone());
    let video_supported = contains(&client.video_codecs, &video.codec);
    let audio_supported = selected_audio
        .map(|track| contains(&client.audio_codecs, &track.codec))
        .unwrap_or(true);
    let container_supported = contains(&client.containers, &metadata.container);
    let explicit_non_default_audio = selected_audio.is_some_and(|track| {
        item.selected_audio_track_id.as_deref() == Some(track.id.as_str()) && !track.is_default
    });
    let audio_selection_supported = !explicit_non_default_audio || client.embedded_audio_selection;
    let (subtitle_delivery, subtitle_direct) = subtitle_plan(selected_subtitle, client);

    if video_supported
        && audio_supported
        && container_supported
        && audio_selection_supported
        && subtitle_direct
    {
        return PlaybackDecision {
            method: PlaybackMethod::DirectPlay,
            reason: DecisionReason::DirectPlayCompatible,
            target_container: None,
            selected_audio_track_id: audio_id,
            selected_subtitle_track_id: subtitle_id,
            subtitle_delivery,
        };
    }

    if selected_subtitle
        .is_some_and(|track| track.kind == SubtitleKind::Bitmap && !client.bitmap_subtitles)
    {
        return transcode(
            DecisionReason::BitmapSubtitleRequiresBurnIn,
            client,
            audio_id,
            subtitle_id,
            SubtitleDelivery::BurnIn,
        );
    }
    if selected_subtitle.is_some_and(|track| track.kind == SubtitleKind::Unknown) {
        return PlaybackDecision {
            method: PlaybackMethod::Unavailable,
            reason: DecisionReason::SubtitleFormatUnsupported,
            target_container: None,
            selected_audio_track_id: audio_id,
            selected_subtitle_track_id: subtitle_id,
            subtitle_delivery: SubtitleDelivery::Off,
        };
    }
    if !subtitle_direct && selected_subtitle.is_some() {
        return transcode(
            DecisionReason::SubtitleFormatUnsupported,
            client,
            audio_id,
            subtitle_id,
            SubtitleDelivery::BurnIn,
        );
    }
    if !video_supported {
        return transcode(
            DecisionReason::VideoCodecUnsupported,
            client,
            audio_id,
            subtitle_id,
            subtitle_delivery,
        );
    }
    if !audio_supported {
        return transcode(
            DecisionReason::AudioCodecUnsupported,
            client,
            audio_id,
            subtitle_id,
            subtitle_delivery,
        );
    }

    let remux_reason = if !audio_selection_supported {
        DecisionReason::AudioSelectionRequiresRemux
    } else {
        DecisionReason::ContainerUnsupported
    };
    if let Some(target) = client.remux_targets.iter().find(|target| {
        contains(&client.containers, &target.container)
            && contains(&target.video_codecs, &video.codec)
            && selected_audio
                .map(|track| contains(&target.audio_codecs, &track.codec))
                .unwrap_or(true)
    }) {
        return PlaybackDecision {
            method: PlaybackMethod::Remux,
            reason: remux_reason,
            target_container: Some(target.container.clone()),
            selected_audio_track_id: audio_id,
            selected_subtitle_track_id: subtitle_id,
            subtitle_delivery,
        };
    }

    transcode(
        DecisionReason::NoCompatibleRemuxTarget,
        client,
        audio_id,
        subtitle_id,
        subtitle_delivery,
    )
}

fn effective_audio(item: &MediaItem) -> Result<Option<&AudioTrack>, DecisionReason> {
    let tracks = item
        .metadata
        .as_ref()
        .map(|metadata| metadata.audio_tracks.as_slice())
        .unwrap_or_default();
    if let Some(selected) = item.selected_audio_track_id.as_deref() {
        return tracks
            .iter()
            .find(|track| track.id == selected)
            .map(Some)
            .ok_or(DecisionReason::SelectedAudioTrackUnavailable);
    }
    Ok(tracks
        .iter()
        .find(|track| track.is_default)
        .or_else(|| tracks.first()))
}

fn effective_subtitle(item: &MediaItem) -> Result<Option<&SubtitleTrack>, DecisionReason> {
    let tracks = item
        .metadata
        .as_ref()
        .map(|metadata| metadata.subtitle_tracks.as_slice())
        .unwrap_or_default();
    match item.subtitle_mode {
        SubtitleMode::Off => Ok(None),
        SubtitleMode::Automatic => Ok(tracks
            .iter()
            .find(|track| track.is_forced)
            .or_else(|| tracks.iter().find(|track| track.is_default))),
        SubtitleMode::Track => {
            let selected = item
                .selected_subtitle_track_id
                .as_deref()
                .ok_or(DecisionReason::SelectedSubtitleTrackUnavailable)?;
            tracks
                .iter()
                .find(|track| track.id == selected)
                .map(Some)
                .ok_or(DecisionReason::SelectedSubtitleTrackUnavailable)
        }
    }
}

fn subtitle_plan(
    track: Option<&SubtitleTrack>,
    client: &ClientCapabilities,
) -> (SubtitleDelivery, bool) {
    let Some(track) = track else {
        return (SubtitleDelivery::Off, true);
    };
    match track.kind {
        SubtitleKind::Text if client.external_webvtt => (SubtitleDelivery::ExternalWebVtt, true),
        SubtitleKind::Text if contains(&client.embedded_text_subtitle_codecs, &track.codec) => {
            (SubtitleDelivery::Embedded, true)
        }
        SubtitleKind::Bitmap if client.bitmap_subtitles => (SubtitleDelivery::Embedded, true),
        SubtitleKind::Text | SubtitleKind::Unknown | SubtitleKind::Bitmap => {
            (SubtitleDelivery::BurnIn, false)
        }
    }
}

fn contains(values: &BTreeSet<String>, value: &str) -> bool {
    values
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(value))
}

fn unavailable(reason: DecisionReason) -> PlaybackDecision {
    PlaybackDecision {
        method: PlaybackMethod::Unavailable,
        reason,
        target_container: None,
        selected_audio_track_id: None,
        selected_subtitle_track_id: None,
        subtitle_delivery: SubtitleDelivery::Off,
    }
}

fn transcode(
    reason: DecisionReason,
    client: &ClientCapabilities,
    audio_id: Option<String>,
    subtitle_id: Option<String>,
    subtitle_delivery: SubtitleDelivery,
) -> PlaybackDecision {
    PlaybackDecision {
        method: PlaybackMethod::Transcode,
        reason,
        target_container: client
            .remux_targets
            .iter()
            .find(|target| contains(&client.containers, &target.container))
            .map(|target| target.container.clone()),
        selected_audio_track_id: audio_id,
        selected_subtitle_track_id: subtitle_id,
        subtitle_delivery,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::media::{
        AudioTrack, MediaItem, MediaMetadata, ProbeStatus, SubtitleKind, SubtitleMode,
        SubtitleTrack, VideoTrack,
    };

    use super::{
        decide_playback, ClientCapabilities, DecisionReason, PlaybackMethod, RemuxTarget,
        SubtitleDelivery,
    };

    fn set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn mp4_target() -> RemuxTarget {
        RemuxTarget {
            container: "mov".to_owned(),
            video_codecs: set(&["h264", "hevc"]),
            audio_codecs: set(&["aac"]),
        }
    }

    fn chromium_like() -> ClientCapabilities {
        ClientCapabilities {
            containers: set(&["mov", "webm"]),
            video_codecs: set(&["h264", "vp9", "av1"]),
            audio_codecs: set(&["aac", "opus", "vorbis"]),
            embedded_text_subtitle_codecs: BTreeSet::new(),
            external_webvtt: true,
            embedded_audio_selection: false,
            bitmap_subtitles: false,
            remux_targets: vec![mp4_target()],
        }
    }

    fn firefox_like() -> ClientCapabilities {
        ClientCapabilities {
            containers: set(&["webm", "mov"]),
            video_codecs: set(&["vp9", "av1", "h264"]),
            audio_codecs: set(&["opus", "vorbis", "aac"]),
            embedded_text_subtitle_codecs: BTreeSet::new(),
            external_webvtt: true,
            embedded_audio_selection: false,
            bitmap_subtitles: false,
            remux_targets: vec![mp4_target()],
        }
    }

    fn safari_like() -> ClientCapabilities {
        ClientCapabilities {
            containers: set(&["mov"]),
            video_codecs: set(&["h264", "hevc"]),
            audio_codecs: set(&["aac"]),
            embedded_text_subtitle_codecs: set(&["mov_text"]),
            external_webvtt: true,
            embedded_audio_selection: false,
            bitmap_subtitles: false,
            remux_targets: vec![mp4_target()],
        }
    }

    fn item(container: &str, video_codec: &str, audio_codecs: &[&str]) -> MediaItem {
        MediaItem {
            id: "media".to_owned(),
            title: "Movie".to_owned(),
            extension: "mkv".to_owned(),
            size_bytes: 42,
            probe_status: ProbeStatus::Available,
            selected_audio_track_id: None,
            subtitle_mode: SubtitleMode::Off,
            selected_subtitle_track_id: None,
            metadata: Some(MediaMetadata {
                container: container.to_owned(),
                duration_millis: Some(1_000),
                video: Some(VideoTrack {
                    id: "video".to_owned(),
                    codec: video_codec.to_owned(),
                    width: Some(1920),
                    height: Some(1080),
                }),
                audio_tracks: audio_codecs
                    .iter()
                    .enumerate()
                    .map(|(index, codec)| AudioTrack {
                        id: format!("audio-{index}"),
                        codec: (*codec).to_owned(),
                        channels: Some(2),
                        language: None,
                        title: None,
                        is_default: index == 0,
                    })
                    .collect(),
                subtitle_tracks: Vec::new(),
            }),
        }
    }

    fn add_subtitle(item: &mut MediaItem, kind: SubtitleKind, codec: &str) {
        item.metadata
            .as_mut()
            .expect("metadata")
            .subtitle_tracks
            .push(SubtitleTrack {
                id: "subtitle".to_owned(),
                codec: codec.to_owned(),
                language: Some("eng".to_owned()),
                title: None,
                is_default: true,
                is_forced: false,
                kind,
            });
        item.subtitle_mode = SubtitleMode::Track;
        item.selected_subtitle_track_id = Some("subtitle".to_owned());
    }

    #[test]
    fn decision_table_covers_representative_browser_profiles_and_mkv_tracks() {
        struct Case {
            name: &'static str,
            item: MediaItem,
            client: ClientCapabilities,
            method: PlaybackMethod,
            reason: DecisionReason,
            subtitles: SubtitleDelivery,
        }

        let direct_mp4 = item("mov", "h264", &["aac"]);
        let mkv_remux = item("matroska", "h264", &["aac"]);
        let mut alternate_audio = item("matroska", "h264", &["aac", "aac"]);
        alternate_audio.selected_audio_track_id = Some("audio-1".to_owned());
        let unsupported_video = item("matroska", "hevc", &["aac"]);
        let unsupported_audio = item("matroska", "h264", &["ac3"]);
        let webm_direct = item("webm", "vp9", &["opus"]);
        let mut text_subtitle = item("mov", "h264", &["aac"]);
        add_subtitle(&mut text_subtitle, SubtitleKind::Text, "subrip");
        let mut bitmap_subtitle = item("mov", "h264", &["aac"]);
        add_subtitle(
            &mut bitmap_subtitle,
            SubtitleKind::Bitmap,
            "hdmv_pgs_subtitle",
        );

        let cases = [
            Case {
                name: "Chromium-like MP4 Direct Play",
                item: direct_mp4,
                client: chromium_like(),
                method: PlaybackMethod::DirectPlay,
                reason: DecisionReason::DirectPlayCompatible,
                subtitles: SubtitleDelivery::Off,
            },
            Case {
                name: "Safari-like MKV container remux",
                item: mkv_remux,
                client: safari_like(),
                method: PlaybackMethod::Remux,
                reason: DecisionReason::ContainerUnsupported,
                subtitles: SubtitleDelivery::Off,
            },
            Case {
                name: "explicit second audio requires remux",
                item: alternate_audio,
                client: safari_like(),
                method: PlaybackMethod::Remux,
                reason: DecisionReason::AudioSelectionRequiresRemux,
                subtitles: SubtitleDelivery::Off,
            },
            Case {
                name: "unsupported HEVC requires transcode",
                item: unsupported_video,
                client: chromium_like(),
                method: PlaybackMethod::Transcode,
                reason: DecisionReason::VideoCodecUnsupported,
                subtitles: SubtitleDelivery::Off,
            },
            Case {
                name: "unsupported AC3 requires transcode",
                item: unsupported_audio,
                client: safari_like(),
                method: PlaybackMethod::Transcode,
                reason: DecisionReason::AudioCodecUnsupported,
                subtitles: SubtitleDelivery::Off,
            },
            Case {
                name: "Firefox-like WebM Direct Play",
                item: webm_direct,
                client: firefox_like(),
                method: PlaybackMethod::DirectPlay,
                reason: DecisionReason::DirectPlayCompatible,
                subtitles: SubtitleDelivery::Off,
            },
            Case {
                name: "text subtitle uses external WebVTT",
                item: text_subtitle,
                client: chromium_like(),
                method: PlaybackMethod::DirectPlay,
                reason: DecisionReason::DirectPlayCompatible,
                subtitles: SubtitleDelivery::ExternalWebVtt,
            },
            Case {
                name: "bitmap subtitle requires burn-in",
                item: bitmap_subtitle,
                client: chromium_like(),
                method: PlaybackMethod::Transcode,
                reason: DecisionReason::BitmapSubtitleRequiresBurnIn,
                subtitles: SubtitleDelivery::BurnIn,
            },
        ];

        for case in cases {
            let decision = decide_playback(&case.item, &case.client);
            assert_eq!(decision.method, case.method, "{}", case.name);
            assert_eq!(decision.reason, case.reason, "{}", case.name);
            assert_eq!(decision.subtitle_delivery, case.subtitles, "{}", case.name);
        }
    }

    #[test]
    fn rejects_missing_metadata_video_and_stale_track_choices() {
        let client = chromium_like();
        let mut missing_metadata = item("mov", "h264", &["aac"]);
        missing_metadata.metadata = None;
        assert_eq!(
            decide_playback(&missing_metadata, &client).reason,
            DecisionReason::MetadataUnavailable
        );

        let mut missing_video = item("mov", "h264", &["aac"]);
        missing_video.metadata.as_mut().unwrap().video = None;
        assert_eq!(
            decide_playback(&missing_video, &client).reason,
            DecisionReason::VideoStreamMissing
        );

        let mut stale_audio = item("mov", "h264", &["aac"]);
        stale_audio.selected_audio_track_id = Some("missing".to_owned());
        assert_eq!(
            decide_playback(&stale_audio, &client).reason,
            DecisionReason::SelectedAudioTrackUnavailable
        );

        let mut stale_subtitle = item("mov", "h264", &["aac"]);
        stale_subtitle.subtitle_mode = SubtitleMode::Track;
        stale_subtitle.selected_subtitle_track_id = Some("missing".to_owned());
        assert_eq!(
            decide_playback(&stale_subtitle, &client).reason,
            DecisionReason::SelectedSubtitleTrackUnavailable
        );
    }

    #[test]
    fn uses_embedded_text_when_available_and_burns_it_when_no_text_path_exists() {
        let mut media = item("mov", "h264", &["aac"]);
        add_subtitle(&mut media, SubtitleKind::Text, "mov_text");
        let mut embedded = safari_like();
        embedded.external_webvtt = false;
        let decision = decide_playback(&media, &embedded);
        assert_eq!(decision.method, PlaybackMethod::DirectPlay);
        assert_eq!(decision.subtitle_delivery, SubtitleDelivery::Embedded);

        embedded.embedded_text_subtitle_codecs.clear();
        let decision = decide_playback(&media, &embedded);
        assert_eq!(decision.method, PlaybackMethod::Transcode);
        assert_eq!(decision.reason, DecisionReason::SubtitleFormatUnsupported);
        assert_eq!(decision.subtitle_delivery, SubtitleDelivery::BurnIn);

        let mut unknown = item("mov", "h264", &["aac"]);
        add_subtitle(&mut unknown, SubtitleKind::Unknown, "mystery_codec");
        let decision = decide_playback(&unknown, &chromium_like());
        assert_eq!(decision.method, PlaybackMethod::Unavailable);
        assert_eq!(decision.reason, DecisionReason::SubtitleFormatUnsupported);
    }
}
