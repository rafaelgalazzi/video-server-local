# Current Task

## ID

LS-069

## Title

Audio-track selection

## Status

Not Started

## Goal

Expose accessible audio choices, persist safe per-item selections, and apply opaque selected tracks during Direct Play, remux, or transcode decisions.

## Completed

- Phase A through LS-031.
- LS-043 safe FFmpeg/ffprobe discovery and process boundary.
- LS-044 bounded normalized metadata probing, per-item failure isolation, schema-v4 persistence, and restart restoration.

## Verification

- Parser fixtures cover dual audio, missing tags/streams, text subtitles, bitmap subtitles, dispositions, and malformed output.
- A real generated MKV verifies dual-audio/text-subtitle probing, corrupt-file isolation, persistence, and restart restoration when local FFmpeg tools are installed.

## Remaining

- Add per-media audio preference persistence and safe reset behavior.
- Expose accessible audio labels and selection state in Vue.
- Resolve opaque track IDs to private ffprobe source indices for later playback decisions.

## Next Exact Step

Implement LS-069 persisted audio-track preference APIs and core validation against the current probed item.
