# Local Playback Coordination

## Purpose

Integrate explicit client compatibility decisions with Direct Play, remux, and transcode execution through one reusable core service.

## Features

- Preserves Direct Play precedence.
- Submits remux/transcode work only when selected by the compatibility engine.
- Exposes path-free preparation and job snapshots with progress and safe failures.
- Supports cancellation, completed-output opening, and explicit cleanup.
- Reuses persisted audio/subtitle selections on every preparation.

## Public Interfaces

- `LocalPlaybackService::start`: creates the bounded transform manager.
- `prepare`: returns Direct Play immediately or an opaque fallback job.
- `snapshot`, `cancel`, `open_output`, and `release`: manage fallback lifecycle.

## Limitations

The service is transport-neutral. Browser capability collection and HTTP/Tauri adapters must pass explicit capabilities; user-agent inference remains prohibited. Adaptive streaming remains deferred under DD-001.
