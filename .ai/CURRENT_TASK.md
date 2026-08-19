# Current Task

## ID

LS-048

## Title

Transcode fallback

## Status

Not Started

## Goal

Implement bounded video/audio transcoding and subtitle conversion or burn-in profiles for measured browser compatibility gaps.

## Completed

- Phase A through LS-031.
- LS-043 bounded FFmpeg process boundary.
- LS-044 normalized metadata probing.
- LS-069 and LS-070 persisted track preferences and subtitle delivery semantics.
- LS-045 explicit client-capability decision engine with Direct Play → remux → transcode precedence and stable reason codes.
- LS-046 bounded media job concurrency, queues, deduplication, cancellation, temporary reservations, cleanup, and safe progress snapshots.
- LS-047 containment-checked MP4/WebM remux jobs with exact selected-track stream-copy mapping and opaque output access.

## Verification

- Real FFmpeg/ffprobe coverage verifies playable WebM output, exact second-audio selection, embedded text subtitle mapping, cancellation, unsupported requests, containment, and quota rejection.

## Remaining

- Define conservative software video/audio encoding profiles for MP4 and WebM.
- Map selected audio and subtitle conversion/burn-in modes through structured FFmpeg arguments.
- Cover representative output codecs, cancellation/concurrency, quotas, track correctness, and Direct Play precedence.

## Next Exact Step

Implement LS-048 bounded transcode fallback through the existing compatibility, process, and job boundaries.
