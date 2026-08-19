# Current Task

## ID

LS-047

## Title

Remux fallback

## Status

Not Started

## Goal

Stream-copy compatible selected tracks from unsupported containers into a browser-compatible container through the bounded process and job boundaries.

## Completed

- Phase A through LS-031.
- LS-043 bounded FFmpeg process boundary.
- LS-044 normalized metadata probing.
- LS-069 and LS-070 persisted track preferences and subtitle delivery semantics.
- LS-045 explicit client-capability decision engine with Direct Play → remux → transcode precedence and stable reason codes.
- LS-046 bounded media job concurrency, queues, deduplication, cancellation, temporary reservations, cleanup, and safe progress snapshots.

## Verification

- Job tests cover saturation, active-key deduplication, cancellation, manager shutdown, quota admission/output enforcement, stale restart cleanup, bounded progress, and cancellation propagation through the child-process boundary.

## Remaining

- Build structured FFmpeg stream-copy arguments from compatibility decisions and private source-track indices.
- Produce a bounded browser-compatible output without exposing paths.
- Cover dual-audio/subtitle mapping, cancellation, unsupported input, quota, and output playability.

## Next Exact Step

Implement LS-047 remux fallback through the LS-043 process boundary and LS-046 job manager.
