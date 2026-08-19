# LocalStream Implementation Roadmap

## Purpose

This is the canonical ordered backlog after LS-023. It converts the accepted architecture and product milestones into resumable implementation tasks. Permanent LS IDs are never renumbered. `.ai/CURRENT_TASK.md` remains the authority for the one active task, while `.ai/PROJECT_STATUS.md` records implemented state.

The committed completion target is a release-ready desktop-to-browser LAN MVP: a desktop node can securely expose an approved library, an explicitly trusted browser or native client can discover and pair with it, and compatible media can Direct Play without Internet access. Later mobile and product work is gated on evidence from that MVP.

## Planning Rules

- Execute tasks in dependency order unless a task explicitly says it can run in parallel.
- Write detailed acceptance criteria into `.ai/CURRENT_TASK.md` when starting each task; this roadmap defines scope, not an active-task handoff.
- Do not enable a non-loopback listener before LS-031 completes every remaining ADR-0006/ADR-0007 gate.
- Each task includes focused tests, relevant documentation updates, full affected-stack verification, and a diff review.
- New architecture choices require an ADR. Deferred decisions remain closed until their documented trigger occurs.
- Platform claims require execution on that platform; compilation elsewhere is not verification.

## Phase A — Complete the secure LAN web-server milestone

### LS-024 — Same-origin browser UI hosting

Serve the production Vue application and immutable assets from the dormant HTTPS router, with SPA navigation fallback, correct content types, bounded file reads, safe cache headers, and no fallback that masks `/api/` errors. Keep the active desktop HTTP listener and bind scope unchanged.

Depends on: LS-023.

Completion evidence: route/security tests for assets, SPA fallback, API precedence, traversal/malformed paths, authentication behavior, and unchanged loopback-only lifecycle.

### LS-025 — Remote-browser application bootstrap

Make the Vue application select its same-origin HTTPS API when loaded remotely while retaining Tauri commands and the trusted-local loopback flow for desktop. Provide explicit bootstrap, pairing-required, authenticated, disconnected, and retry states without exposing credentials to JavaScript.

Depends on: LS-024.

Completion evidence: composable/component tests for native and browser modes, cookie-based requests, error transitions, and production build asset compatibility.

### LS-026 — Browser trust-onboarding workflow

Turn the existing trusted-local root export and fingerprint display into a documented, accessible onboarding workflow. Explain platform trust installation and require fingerprint comparison; never install trust automatically or expose a remote CA download.

Depends on: LS-025. Can proceed in parallel with LS-027.

Completion evidence: UI tests, Windows manual verification, platform-specific instructions clearly marked verified or unverified, and negative review for certificate-warning bypass guidance.

### LS-027 — Native peer trust and secure credential storage

Add an isolated native-client trust store that binds the stable node ID, verified root fingerprint, endpoint hints, and bearer credential using platform-protected storage. Provide explicit replacement/re-pair and deletion behavior; never accept discovery or first contact as identity proof.

Depends on: LS-023.

Completion evidence: injected-store unit tests for persistence/corruption/deletion, pin mismatch and identity-change tests, and at least one real platform adapter smoke test.

### LS-028 — Explicit LAN endpoint configuration

Model disabled-by-default LAN serving, validated explicit interface/address selection, port policy, safe network-address enumeration, and public status values. Wildcard binding must never be an implicit default and raw internal interface details must not leak into public APIs.

Depends on: LS-023.

Completion evidence: address/config parsing tests, loopback/wildcard/invalid-address rejection tests, persistence tests, and UI state tests. This task does not bind a LAN socket.

### LS-029 — TLS leaf lifecycle and address changes

Issue and rotate short-lived leaves for the configured endpoint names/addresses, retain the stable root identity, and handle renewal or address changes without crashing trusted-local functionality. New connections receive current material; existing connections shut down cleanly.

Depends on: LS-028.

Completion evidence: restart, renewal, changed-SAN, expiry, wrong-name, wrong-root, partial-failure, and graceful-rotation tests.

### LS-030 — Secure LAN listener orchestration

Compose identity loading, current leaf material, the authenticated HTTPS router, static UI assets, sessions, limits, and explicit configuration into a fail-closed lifecycle owned by reusable core services. Startup failures must leave the desktop application usable and must never fall back to plaintext.

Depends on: LS-024, LS-028, LS-029.

Completion evidence: orchestration tests for every failure gate, shutdown/restart tests, safe status reporting, and proof that default startup remains LAN-disabled.

### LS-031 — LAN activation and security gate audit

Permit non-loopback binding only after an auditable runtime preflight confirms all ADR-0006/ADR-0007 gates. Add end-to-end negative tests for unauthorized access, downgrade, hostile authority/origin, session misuse, revocation, identity change, oversized input, saturation, and secret/path leakage.

Depends on: LS-026, LS-027, LS-030.

Completion evidence: a completed security checklist, real two-device or isolated-network verification, full workspace checks, and updated threat model/API documentation. This is the first task allowed to enable LAN binding.

## Phase A.1 — MKV track support and browser compatibility

This mandatory tranche runs before Phase B. It establishes reliable local playback for MKV files with multiple audio streams and embedded subtitles before discovery adds more transport variables. Existing permanent IDs LS-043 through LS-049 move here without renumbering; LS-069 and LS-070 are the next available permanent IDs.

### LS-043 — FFmpeg tool discovery and process boundary

Define supported ffprobe/ffmpeg acquisition/configuration, validate executables, isolate platform process spawning, bound output/time, support cancellation, and prohibit shell interpolation.

Depends on: LS-031.

Completion evidence: packaging decision, fake-process and hostile filename/argument tests, timeout/cancellation tests, and a real-tool smoke test.

### LS-044 — ffprobe metadata service

Probe and persist normalized container, codec, dimension, duration, disposition, language/title, and complete audio/subtitle stream metadata. One corrupt file must not abort a scan.

Depends on: LS-043.

Completion evidence: fixtures including MKV with dual audio and embedded text/bitmap subtitles, malformed/missing streams, timeout/inaccessible media, and migration/restart tests.

### LS-069 — Audio-track selection

Expose path-free audio metadata and opaque track identifiers, render accessible language/title/default labels, persist safe per-item choices, and apply the selection during Direct Play, remux, or transcode.

Depends on: LS-044.

Completion evidence: core and Vue tests for dual/multiple audio, missing tags, defaults, changed scans, invalid identifiers, and selection retention/reset.

### LS-070 — Embedded subtitle selection

Expose path-free subtitle metadata and accessible Off/default/forced/language choices. Use browser-compatible embedded text tracks directly where possible; otherwise extract, convert, remux, or burn through bounded structured FFmpeg jobs. Bitmap subtitles must fail clearly or use an explicitly supported transform.

Depends on: LS-044.

Completion evidence: fixture/UI/integration tests for common text subtitles, representative bitmap subtitles, forced/default flags, Off state, malformed tracks, cleanup, and language/title labels.

### LS-045 — Direct Play compatibility decisions

Select Direct Play first, then remux, then transcode using container, video codec, selected audio codec, subtitle format/mode, and client capabilities, with an explainable fallback reason.

Depends on: LS-044, LS-069, LS-070.

Completion evidence: decision-table tests for MKV dual-audio/subtitle combinations across representative browser profiles.

### LS-046 — Bounded media job manager

Own transform concurrency, queue limits, deduplication, cancellation, process cleanup, temporary quotas, stale cleanup, and safe progress models.

Depends on: LS-043.

Completion evidence: saturation, cancellation, crash/quota/restart cleanup, and no-orphan-process tests.

### LS-047 — Remux fallback

Stream-copy compatible selected tracks from MKV or other unsupported containers into a browser-compatible container using structured arguments and bounded output.

Depends on: LS-045, LS-046.

Completion evidence: dual-audio/subtitle mapping, output/playability, cancellation, unsupported-input, and resource-limit tests.

### LS-048 — Transcode fallback

Implement bounded video/audio transcoding and subtitle conversion/burn-in profiles for measured browser gaps. Hardware acceleration remains disabled until separately validated.

Depends on: LS-045, LS-046.

Completion evidence: representative MKV outputs, selected-track correctness, subtitle modes, cancellation/concurrency, resource limits, and Direct Play precedence.

### LS-049 — Local playback fallback integration

Integrate track selectors, compatibility decisions, and jobs into local playback with clear progress/error/cancel states and cleanup. Later remote playback must reuse these core decisions.

Depends on: LS-047, LS-048.

Completion evidence: end-to-end MKV Direct Play/remux/transcode tests with dual audio, embedded subtitles, selection changes, and recovery.

## Phase B — Discovery and native pairing

### LS-032 — Discovery protocol ADR and service contract

Define the offline mDNS service type, minimal non-secret TXT records, stable-node-ID semantics, endpoint validation, conflict behavior, TTL policy, and trust boundary. Discovery locates nodes but never establishes identity.

Depends on: LS-049.

Completion evidence: accepted ADR, parser/model unit tests, and documented privacy/security limits.

### LS-033 — mDNS advertisement

Advertise only an enabled healthy HTTPS LAN endpoint, stable node ID, and non-secret version/capability hints. Withdraw promptly on disable/shutdown and recover from network changes without crashing local functionality.

Depends on: LS-032.

Completion evidence: lifecycle tests with an injectable backend plus real-network smoke verification where available.

### LS-034 — mDNS discovery and bounded node registry

Browse and validate advertisements into a bounded core-owned registry with deduplication, TTL expiry, interface changes, and safe rejection of malformed or oversized records.

Depends on: LS-032. Can proceed in parallel with LS-033.

Completion evidence: deterministic fake-clock/backend tests for add/update/expire/conflict and malformed hostile input.

### LS-035 — Discovered-node UI

Expose path/token-free node summaries through thin Tauri adapters and Vue composables. Show identity, endpoint hint, trust state, availability, refresh, empty, and failure states accessibly.

Depends on: LS-034.

Completion evidence: adapter tests and Vue state/component tests; discovery metadata is never presented as trusted proof.

### LS-036 — Native remote pairing flow

Implement the native client side of encrypted pairing: pin-verification ceremony, request creation, human code comparison, approval polling/claim, and protected credential persistence. Handle denial, expiry, replay, network loss, and changed identity safely.

Depends on: LS-027, LS-035.

Completion evidence: end-to-end tests using two identities and negative cases for MITM/pin mismatch, replay, cancellation, and storage failure.

### LS-037 — Availability and endpoint tracking

Combine discovery observations with authenticated health checks without conflating reachability, trust, and authorization. Add bounded backoff, jitter, cancellation, offline transitions, and network-change recovery.

Depends on: LS-034, LS-036.

Completion evidence: fake-time transition tests and failure containment tests.

## Phase C — Distributed library and playback

### LS-038 — Authenticated remote-client transport

Create a reusable Rust client transport using pinned TLS and protected bearer retrieval, strict response/body/time limits, capability checks, and cancellation. Do not duplicate API logic in Vue.

Depends on: LS-036.

Completion evidence: trusted/untrusted TLS tests, auth failure/revocation tests, size/timeout tests, and no-secret logging review.

### LS-039 — Remote library query service

Fetch and validate versioned path-free library responses from trusted online peers, with bounded concurrency and per-peer error isolation.

Depends on: LS-038.

Completion evidence: schema compatibility, hostile payload, partial failure, timeout, and revocation tests.

### LS-040 — Unified library model and UI

Merge local and remote items without losing source identity, trust, or availability. Define stable collision-safe view identities and accessible filtering/grouping states.

Depends on: LS-039.

Completion evidence: deterministic merge tests and Vue tests for mixed, duplicate, offline, empty, and partial-error libraries.

### LS-041 — Remote Direct Play

Stream remote media over pinned TLS with Range support, cancellation, bounded buffering, capability validation, and clear compatibility/network errors. Prefer Direct Play before any transform.

Depends on: LS-038, LS-040.

Completion evidence: full/ranged playback integration tests, wrong-peer/revoked/offline tests, cancellation, and memory/backpressure checks.

### LS-042 — Distributed resilience and refresh

Handle peers disappearing during queries or playback, stale results, library revision changes, retry policy, and UI recovery without crashing or silently switching identity.

Depends on: LS-037, LS-041.

Completion evidence: deterministic churn tests and an end-to-end two-node interruption/recovery scenario.

## Phase D — Adaptive playback decision

Compatibility metadata, track selection, and progressive FFmpeg fallback moved to mandatory Phase A.1 so reliable MKV playback precedes discovery. This phase retains only the evidence-gated adaptive-streaming decision.

### LS-050 — Adaptive streaming decision gate

Measure actual client gaps after LS-049 and resolve DD-001. Implement HLS only if the recorded evidence requires it; otherwise record a rejected/postponed ADR and keep progressive fallback.

Depends on: LS-049.

Completion evidence: compatibility evidence and an accepted ADR. If HLS is selected, the task expands into separately numbered segmenting, manifest, cache/quota, authorization, cancellation, and cleanup tasks before implementation begins.

## Phase E — Release engineering and desktop MVP completion

### LS-051 — CI and supported toolchain policy

Resolve TD-001, define the actual Rust/Node baselines, and run format, lint, typecheck, unit, integration, build, dependency, and platform jobs in CI with reproducible locked dependencies.

Depends on: may begin after LS-024 and evolve through LS-050.

Completion evidence: green CI on every claimed platform and documented local parity commands.

### LS-052 — End-to-end and security regression suite

Automate desktop startup, library approval, browser trust/pairing, discovery, remote browsing, Range playback, revocation, restart, identity reset, and transform fallback in isolated test environments.

Depends on: LS-050, LS-051.

Completion evidence: stable CI suite with secrets/paths redacted and failures preserving useful non-sensitive diagnostics.

### LS-053 — Operational diagnostics and privacy review

Add structured bounded diagnostics for lifecycle and failures, explicit support export, retention/redaction policy, and a privacy review proving paths, tokens, sessions, claim secrets, and sensitive media data are not logged.

Depends on: LS-050.

Completion evidence: redaction tests, bounded-log tests, and security documentation.

### LS-054 — Configuration, migration, backup, and recovery

Version all persisted configuration/state, test forward migrations and unsupported-version failure, document backup boundaries, and provide safe recovery for corrupt non-secret state without silently replacing identity.

Depends on: LS-050.

Completion evidence: upgrade/restart/corruption fixtures and recovery tests.

### LS-055 — Windows packaging and verification

Package the Vue assets, Rust core, platform secret storage, and any selected FFmpeg distribution; verify install, upgrade, uninstall, firewall UX, trust onboarding, playback, and data retention on supported Windows versions.

Depends on: LS-052 through LS-054.

Completion evidence: signed release-candidate artifacts and a completed Windows test matrix.

### LS-056 — Linux packaging and verification

Define supported distributions and secret-service/system dependencies; package and verify install, upgrade, uninstall, browser trust, firewall/network behavior, playback, and optional headless prerequisites.

Depends on: LS-052 through LS-054.

Completion evidence: release-candidate artifacts and completed per-distribution test evidence.

### LS-057 — macOS packaging and verification

Package, sign/notarize as applicable, and verify Keychain, trust onboarding, permissions, install/upgrade/uninstall, network behavior, and playback on supported macOS versions.

Depends on: LS-052 through LS-054.

Completion evidence: release-candidate artifacts and a completed macOS test matrix.

### LS-058 — Performance and soak qualification

Measure large libraries, repeated scans, concurrent streams, discovery churn, pairing abuse, slow TLS clients, remote failures, and transform pressure against explicit budgets.

Depends on: LS-052.

Completion evidence: recorded budgets/results, no unbounded growth, and tracked issues for regressions.

### LS-059 — Independent security review and remediation

Review the threat model, private-PKI onboarding, storage boundaries, HTTP/TLS parsing, authorization, filesystem containment, process execution, dependency risks, and negative tests. Remediation receives separate KI/LS IDs when non-trivial.

Depends on: LS-052 through LS-054.

Completion evidence: review report, triaged findings, completed release-blocking remediation, and rerun regression suite.

### LS-060 — Desktop LAN MVP release readiness

Reconcile every acceptance criterion, known issue, limitation, deferred decision, test matrix cell, user document, license, and artifact. Produce a release candidate only when unsupported behavior is explicit and no security gate is bypassed.

Depends on: LS-055 through LS-059.

Completion evidence: release checklist, supported-platform matrix, clean full verification, install/upgrade smoke tests, and tagged release candidate.

## Phase F — Post-MVP gated work

These tasks are planned but must not delay LS-060. Their detailed implementation tasks are created only after their decision gates produce evidence.

### LS-061 — Android requirements and lifecycle ADR

Resolve DD-003 for Android using the desktop MVP and target-device experiments: client/server roles, foreground service, storage access, discovery, battery/network constraints, and reusable-core boundaries.

Depends on: LS-060.

### LS-062 — Android client vertical slice

After LS-061, package the reusable UI/core for discovery, trust/pairing, remote library, and playback with platform-secure storage and lifecycle-safe cancellation.

Depends on: LS-061; split into permanent subtasks after the ADR is accepted.

### LS-063 — Android server-mode evaluation

Implement server mode only if LS-061 accepts it and platform policy allows reliable, user-visible operation. Otherwise document the limitation.

Depends on: LS-061.

### LS-064 — iOS requirements and lifecycle ADR

Evaluate client discovery, pairing, playback, secure storage, local-network permission, and foreground/background server limits using real devices.

Depends on: LS-060.

### LS-065 — iOS client vertical slice

After LS-064, implement the accepted client scope and split it into platform-specific permanent tasks.

Depends on: LS-064.

### LS-066 — Optional metadata and artwork decision

Resolve DD-002 using MVP feedback and a privacy/offline/cache design. External metadata remains explicit opt-in; local thumbnails may proceed independently through newly planned tasks.

Depends on: LS-060.

### LS-067 — Headless/NAS distribution ADR

Define the protected-secret fallback, administration/onboarding surface, service lifecycle, filesystem permissions, update model, and Docker/NAS constraints before implementing a headless distribution.

Depends on: LS-060.

### LS-068 — Product-feature prioritization

Use MVP evidence to prioritize continue-watching, music, profiles, casting, TV applications, auto-update, backup/export, and synchronization. Audio-track and subtitle support are already accepted in mandatory Phase A.1. Create new permanent LS tasks only for other accepted work; honor DD-004 through DD-006.

Depends on: LS-060.

## Explicitly Outside the Current Completion Target

- Remote Internet access, cloud identity, or relays (DD-006).
- Network filesystem protocols (DD-005).
- Native TV applications before browser-client evidence (DD-004).
- HLS before LS-050 resolves DD-001.
- External metadata before LS-066 resolves DD-002.
- Unprioritized Milestone 8 ideas.
