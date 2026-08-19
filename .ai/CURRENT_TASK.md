# Current Task

## ID

LS-046

## Title

Bounded media job manager

## Status

Not Started

## Goal

Own transform concurrency, bounded queues, deduplication, cancellation, child cleanup, temporary quotas, stale cleanup, and safe progress models.

## Completed

- Phase A through LS-031.
- LS-043 bounded FFmpeg process boundary.
- LS-044 normalized metadata probing.
- LS-069 and LS-070 persisted track preferences and subtitle delivery semantics.
- LS-045 explicit client-capability decision engine with Direct Play → remux → transcode precedence and stable reason codes.

## Verification

- Decision-table tests cover conservative Chromium-like, Firefox-like, and Safari-like capability inputs.
- Cases cover MP4/WebM Direct Play, MKV remux, selected second audio, unsupported video/audio, external/embedded text subtitles, bitmap burn-in, unknown formats, missing metadata/video, and stale selections.

## Remaining

- Define bounded job identifiers, states, progress, queue admission, and deduplication keys.
- Enforce transform concurrency, temporary-byte quotas, cancellation, process cleanup, and stale restart cleanup.
- Add saturation, cancellation, crash/quota, cleanup, and no-orphan-process tests.

## Next Exact Step

Implement LS-046's reusable media job manager around the LS-043 process boundary.
