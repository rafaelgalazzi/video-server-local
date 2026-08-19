# ADR-0009 — Progressive HLS fallback

## Status

Accepted

## Context

LS-049 proved Direct Play and complete-file MP4/WebM fallback. Physical browser evidence then showed two material gaps: an H.264 MKV with browser-incompatible multichannel audio could play silently when delivered directly, while preparing a complete fallback file delayed playback and could consume excessive CPU. This satisfies the DD-001 decision trigger.

The application must remain local-first, Direct Play-first, bounded, authenticated on the LAN, and usable without an Internet CDN. Raw paths and unbounded FFmpeg/cache behavior remain prohibited.

## Decision

Add progressive HLS as a browser fallback, not as the universal delivery format.

1. Direct Play remains first when the complete selected container/video/audio combination is supported.
2. For compatible video in an incompatible container or with incompatible audio, generate short HLS segments by stream-copying the selected video and transcoding only the selected audio to AAC. This is the preferred low-CPU path.
3. Full software video transcoding is a distinct slow path used only when the selected video codec is unsupported or subtitle burn-in is explicitly required. The UI must identify this cost before or during preparation.
4. Initial segments become readable while FFmpeg continues; clients do not wait for the complete title.
5. Segment production, sessions, processes, temporary bytes, names, lifetime, and cleanup are bounded. Cancellation reaches the FFmpeg child.
6. Browser control endpoints are same-origin HTTPS, session-authenticated unsafe requests protected by exact Origin, Fetch Metadata, and an explicit CSRF token. Playlists and segments use opaque session IDs and never expose paths.
7. Browser HLS support is bundled locally. Primary playback must not require a CDN or Internet access.
8. Subtitle delivery prefers external WebVTT. Burn-in is never selected merely because an optional subtitle is available.

## Alternatives Considered

- Continue waiting for complete MP4/WebM fallback files.
- Always transcode video and audio.
- Serve MKV directly and accept missing audio on browsers.
- Copy the unauthenticated Node prototype, including wildcard binding and CDN-hosted HLS.js.
- Make HLS the only delivery mode.

## Consequences

Compatible H.264/H.265 sources can normally begin after a small number of segments with video-copy CPU cost and audio-only AAC encoding. Unsupported video and subtitle burn-in remain CPU-intensive. Seeking beyond generated media may be limited until later segments exist. The implementation adds session lifecycle, CSRF, manifest/segment authorization, cache quotas, offline frontend assets, and more integration tests.
