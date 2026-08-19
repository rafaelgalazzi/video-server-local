# Current Task

## ID

LS-045

## Title

Direct Play compatibility decisions

## Status

Not Started

## Goal

Select Direct Play first, then remux, then transcode from normalized media metadata, selected tracks, subtitle delivery requirements, and explicit browser capabilities.

## Completed

- Phase A through LS-031.
- LS-043 safe FFmpeg/ffprobe discovery and process boundary.
- LS-044 normalized metadata probing and schema-v4 persistence.
- LS-069 schema-v5 audio preferences and private source-index resolution.
- LS-070 schema-v6 Automatic/Off/track subtitle preferences, accessible controls, bounded WebVTT text extraction, and explicit bitmap-transform errors.

## Verification

- Core tests cover subtitle Off/Automatic/track modes, invalid IDs, restart retention, changed-track reset, and bitmap rejection.
- A generated MKV fixture verifies embedded text conversion to WebVTT through bounded structured FFmpeg execution.
- Vue tests cover forced-before-default behavior, labels, Off, bitmap notices, invalid IDs, and safe remote-mode behavior.

## Remaining

- Define representative browser capability profiles.
- Implement explainable Direct Play/remux/transcode decision rules.
- Cover dual-audio, text/bitmap subtitle, container, and codec combinations with decision-table tests.

## Next Exact Step

Implement LS-045 transport-neutral browser capability and playback-decision models in the Rust core.
