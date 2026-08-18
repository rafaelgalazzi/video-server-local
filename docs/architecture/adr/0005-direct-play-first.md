# ADR-0005 — Prefer Direct Play before transcoding

## Status

Accepted

## Context

Transcoding consumes significant CPU, memory, storage, and power. Many clients can play source media directly.

## Decision

Attempt Direct Play when the client supports the media. Use FFmpeg only when compatibility requires remuxing or transcoding.

```text
Client supports media? -- yes --> Direct Play
          |
          no
          |
        FFmpeg --> Transcoding
```

## Alternatives Considered

- Always transcode to a uniform format.
- Make HLS the only delivery mode.

## Consequences

Compatible playback is cheaper and preserves source quality. Capability negotiation and correct HTTP Range behavior become important. Transcoding requires safe arguments, lifecycle cleanup, and concurrency limits when introduced.
