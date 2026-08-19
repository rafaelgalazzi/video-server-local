# LocalStream Architecture Map

LocalStream is planned as a local-first system with a Vue interface and thin transport adapters around reusable Rust services.

```text
                 Vue 3 + TypeScript
                         |
             +-----------+-----------+
             |                       |
      Tauri Backend             HTTP Backend
             |                       |
       Tauri Commands              REST API
             |                       |
             +-----------+-----------+
                         |
                      Rust Core
                         |
          +--------------+--------------+
          |              |              |
   LibraryService StreamingService NodeService
          |              |              |
        SQLite         FFmpeg           mDNS
```

The Vue/Tauri adapters, shared Rust core, library scanning, SQLite persistence, bounded Direct Play/WebVTT delivery, track preferences, compatibility decisions, bounded remux/transcode jobs, local playback fallback, private-PKI HTTPS, pairing, browser sessions, and audited LAN exposure exist. Discovery remains unimplemented.

## Responsibilities

- Vue: presentation, navigation, playback controls, and state composed with Vue-native primitives.
- Tauri commands: native UI transport adapters; no business logic.
- HTTP handlers: versioned LAN transport adapters, static web hosting, and streaming responses; no duplicated business logic.
- Rust core: approved-library management, media scanning, persistence, bounded streaming, discovery, pairing, security, FFmpeg integration, and shared domain types.
- Platform layer: isolated filesystem selection, lifecycle, permissions, and OS-specific integrations.

## Primary Flow

The first planned vertical slice is approved folder → Rust scanner → SQLite library → Vue list → Axum API → Direct Play with HTTP Range. Transcoding follows only when client compatibility requires it.

## Invariants

The mandatory invariants are canonical in `AGENTS.md`. In summary: native and media business logic stays out of Vue and transport adapters; raw paths never cross public APIs; streaming is bounded; approved libraries are the only exposure boundary; peers are untrusted until paired; resource-heavy work is limited; network and per-file failures are contained; and primary LAN operation needs no Internet access.

## Decisions

See `docs/architecture/adr/` for accepted decisions. Significant changes require a new ADR rather than silent drift.

The future network boundary has two separate listeners: a trusted-local loopback adapter for the desktop shell and an explicitly enabled HTTPS LAN adapter. The latter must load a persistent node identity and satisfy ADR-0006 and ADR-0007 before it may bind beyond loopback.
