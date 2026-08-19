# Current Task

## ID

LS-044

## Title

ffprobe metadata service

## Status

Not Started

## Goal

Probe and persist normalized media/container/track metadata without allowing one corrupt or inaccessible item to abort a library scan.

## Completed

- Phase A through LS-031.
- LS-043 safe FFmpeg/ffprobe discovery and bounded structured process boundary.

## Verification

- LS-043 unit coverage includes structured hostile arguments, output limits, timeout, cancellation, invalid explicit paths, and installed-tool discovery.
- Real `ffprobe` and `ffmpeg` discovery executes on the development Windows host.

## Remaining

- Implement normalized ffprobe models and parsing.
- Integrate per-item probing with scan failure isolation.
- Add schema migration and restart-safe metadata persistence.
- Add dual-audio, text/bitmap subtitle, malformed, timeout, and inaccessible fixture coverage.

## Next Exact Step

Implement LS-044 normalized ffprobe models and parser using `media_tools::ProcessRunner`.
