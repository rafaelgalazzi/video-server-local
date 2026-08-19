# Current Task

## ID

LS-032

## Title

Discovery protocol ADR and service contract

## Status

Not Started

## Goal

Define the offline mDNS discovery contract, validation rules, lifecycle semantics, and trust boundary before implementation.

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

## Verification

- End-to-end MKV coverage verifies Direct Play, selected-track remux, transcode, selection changes, cancellation, retry, subtitle conversion/burn-in, and playable outputs.
- Desktop UI tests cover completed fallback URLs and cancellation/release cleanup.

## Remaining

- Define the local-only mDNS service type and minimal non-secret records.
- Specify endpoint, stable node ID, size, TTL, conflict, and trust validation.
- Add parser/model tests and document privacy/security limits in an accepted ADR.

## Next Exact Step

Implement LS-032 discovery protocol ADR and transport-neutral service contract.
