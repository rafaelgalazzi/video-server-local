# Current Task

## ID

LS-032

## Title

Discovery protocol ADR and service contract

## Status

Not Started

## Goal

Define the Phase B discovery protocol and reusable service boundary without weakening the completed Phase A security gates.

## Completed

- Phase A through LS-031.
- LS-043 bounded FFmpeg process boundary.
- LS-044 normalized metadata probing.
- LS-069 and LS-070 persisted track preferences and subtitle delivery semantics.
- LS-045 explicit client-capability decision engine with Direct Play → remux → transcode precedence and stable reason codes.
- LS-046 bounded media job concurrency, queues, deduplication, cancellation, temporary reservations, cleanup, and safe progress snapshots.
- LS-047 containment-checked MP4/WebM remux jobs with exact selected-track stream-copy mapping and opaque output access.
- LS-048 software-only MP4/WebM transcode profiles with selected audio, subtitle conversion, and text/bitmap burn-in.
- LS-049 local playback coordinator and desktop integration with explicit capabilities, Direct Play precedence, fallback progress/error/cancel states, opaque Range output, and cleanup.
- LS-050 client-gap evidence and ADR-0009 resolve DD-001 in favor of progressive browser HLS.
- LS-071 through LS-074 progressive HLS generation and lifecycle, secure desktop/browser delivery, bundled HLS.js playback, and native controls with a growing seekable range.

## Verification

- End-to-end MKV coverage verifies Direct Play, selected-track remux, transcode, selection changes, cancellation, retry, subtitle conversion/burn-in, and playable outputs.
- Desktop UI tests cover completed fallback URLs and cancellation/release cleanup.

## Remaining

- Draft and accept the discovery protocol ADR.
- Define advertisement, registry, expiry, and trust-boundary contracts.

## Next Exact Step

Review the Phase B requirements and create the LS-032 discovery ADR.
