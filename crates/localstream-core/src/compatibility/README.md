# Playback Compatibility

## Purpose

Choose Direct Play before remuxing or transcoding from normalized media metadata, persisted track choices, and explicit client capabilities.

## Features

- Transport-neutral capability input with no user-agent inference.
- Effective default/selected audio and Automatic/Off/selected subtitle resolution.
- Direct Play → remux → transcode precedence.
- External WebVTT, embedded text, bitmap, and burn-in subtitle modes.
- Stable explainable reason codes and optional remux/transcode target container.

## Important Files

- `mod.rs`: capability models, deterministic decision engine, and browser-profile decision table.

## Public Interfaces

- `decide_playback`: evaluates one safe `MediaItem` against `ClientCapabilities`.
- `ClientCapabilities`: caller-supplied supported containers/codecs and track behavior.
- `PlaybackDecision`: selected method, reason, effective tracks, subtitle delivery, and target.

## Dependencies

Uses normalized models from `media/` only. It does not access files, SQLite, HTTP, Tauri, or FFmpeg.

## Limitations

Capability discovery and user-agent mapping are intentionally absent pending target-device evidence under DD-001. Remux decisions can execute through the bounded job manager; transcoding is not implemented.

## Planned Work

LS-048 will consume transcode decisions through the LS-046 bounded job manager; LS-049 will integrate both fallbacks into playback.
