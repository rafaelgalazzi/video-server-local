# Progressive HLS Domain

## Purpose

Produce low-startup browser-compatible HLS fallback without weakening Direct Play precedence or exposing filesystem paths.

## Features

- Four-second event-playlist segments become readable while FFmpeg continues.
- H.264 video is stream-copied while selected audio is converted to AAC stereo for the preferred low-CPU path.
- Unsupported video uses an explicitly reported software H.264 slow path.
- Opaque bounded media jobs provide concurrency, cancellation, quota reservation, and startup cleanup.
- Only fixed playlist and six-digit segment names are readable.

## Important Files

- `mod.rs`: HLS profiles, structured FFmpeg request, progressive output access, and tests.

## Public Interfaces

The core and playback service submit sessions and expose opaque job IDs, snapshots, fixed assets, cancellation, and release. Paths remain private.

## Dependencies

The shared media job manager and structured FFmpeg process boundary.

## Limitations

HLS uses MPEG-TS with AAC audio. Only H.264 takes the video-copy path initially; other video codecs use software x264. The native player seek range grows as new segments become available.
