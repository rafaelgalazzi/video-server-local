# Remux Fallback

## Purpose

Stream-copy compatible selected media tracks into a browser-compatible container when Direct Play is unavailable.

## Features

- Accepts only explicit `PlaybackMethod::Remux` decisions.
- Resolves opaque track choices to private source indices in the core facade.
- Maps exactly one video, the effective audio track, and an embedded subtitle only when requested.
- Uses structured FFmpeg arguments, the bounded process runner, and the bounded media job manager.
- Canonicalizes the source beneath its approved library and returns only an opaque job plus safe output metadata.

## Important Files

- `mod.rs`: remux validation, structured plan construction, job submission, and tests.

## Public Interfaces

- `LocalStreamCore::submit_remux`: validates and submits remux work without exposing paths or source indices.
- `RemuxSubmission`: opaque job submission and output filename/content type.

## Dependencies

Consumes compatibility decisions, private database mappings, `media_tools::ProcessRunner`, and `media_jobs::MediaJobManager`.

## Limitations and Planned Work

MP4 and WebM are the initial output containers. Incompatible subtitles and codecs are handled by `transcode/`; `playback/` coordinates and streams completed output.
