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

User-agent mapping is intentionally absent pending target-device evidence under DD-001. Desktop playback supplies explicit runtime capability probes; remux and transcode decisions execute through the bounded job manager.

## Planned Work

Future native and remote clients must reuse this decision engine and provide explicit capabilities.
