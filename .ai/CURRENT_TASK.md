# Current Task

## ID

LS-070

## Title

Embedded subtitle selection

## Status

Not Started

## Goal

Expose accessible Off/default/forced subtitle choices, persist safe per-item preferences, and route text/bitmap selections into browser-compatible delivery decisions.

## Completed

- Phase A through LS-031.
- LS-043 safe FFmpeg/ffprobe discovery and process boundary.
- LS-044 normalized metadata probing and schema-v4 persistence.
- LS-069 schema-v5 validated audio preferences, opaque private source-index resolution, rescan retention/reset, and accessible native playback controls.

## Verification

- Core tests cover valid/invalid/unknown selections, clearing, restart persistence, source-index resolution, unchanged rescan retention, and changed-track reset.
- Vue tests cover default selection, accessible language/title/codec/channel labels, persistence, and unavailable-mode errors.

## Remaining

- Add Off/default/forced subtitle preference semantics.
- Persist and validate opaque subtitle selections.
- Add accessible Vue subtitle controls.
- Define supported text extraction/conversion and explicit bitmap behavior for compatibility decisions.

## Next Exact Step

Implement LS-070 subtitle preference models and persistence, including the explicit Off state.
