# Transcode Fallback

## Purpose

Produce conservative browser-compatible media when selected source codecs or subtitle delivery prevent Direct Play and remux.

## Features

- Software-only H.264/AAC MP4 and VP9/Opus WebM profiles.
- Exact selected video/audio mapping.
- Text subtitle conversion for embedded delivery.
- Text and bitmap subtitle burn-in through bounded FFmpeg filter arguments.
- Approved-root containment, bounded jobs, cancellation, quotas, and path-free output access.

## Public Interfaces

- `LocalStreamCore::submit_transcode`: resolves private source mappings and submits an explicit transcode decision.
- `TranscodeSubmission`: opaque job and safe output metadata.

## Limitations

Hardware acceleration is disabled pending independent platform validation. Profiles prioritize compatibility and determinism over speed or adaptive bitrate. Parsed FFmpeg time/frame progress is not yet exposed.
