# LocalStream — AI Agent Project Blueprint

## 1. Project Overview

**LocalStream** is a cross-platform local-network media streaming application.

The application should run on:

- Windows
- Linux
- macOS
- Android
- iOS, with platform limitations documented where applicable

The application should be able to act as:

1. A local media server.
2. A local media player.
3. A client for other LocalStream nodes on the same LAN.
4. A web server that exposes a browser-based media interface to other devices on the local network.
5. A distributed media-library node capable of sharing selected local folders with trusted peers.

The primary goal is to provide a simple, private, local-first media experience without requiring cloud infrastructure.

The project should favor:

- Local-first operation.
- Privacy.
- Zero or minimal configuration.
- Reusable architecture.
- Strong separation between UI, application shell, and media/server core.
- Cross-platform behavior wherever feasible.
- Progressive enhancement for advanced media features.

---

# 2. Core Technology Stack

## Frontend

Use:

- Vue 3
- TypeScript
- Vite
- Vue Router
- Vue Composition API

Do **not** use Pinia.

Application state should use Vue-native primitives such as:

- `ref()`
- `reactive()`
- `computed()`
- `watch()`
- `provide()`
- `inject()`

Reusable stateful logic should be placed in composables.

Examples:

```ts
const media = ref<MediaItem[]>([])
const activeNode = ref<NodeInfo | null>(null)
const isServerRunning = ref(false)
```

For reusable features:

```ts
export function useMediaLibrary() {
  const media = ref<MediaItem[]>([])
  const loading = ref(false)

  return {
    media,
    loading,
  }
}
```

Avoid introducing an external global state manager unless a future architectural requirement clearly justifies it.

---

## Native Application Layer

Use:

- Tauri 2
- Rust

Tauri is responsible for:

- Desktop/mobile application shell.
- Native lifecycle integration.
- Native commands.
- Platform integration.
- Application packaging.
- Starting and stopping LocalStream services.
- Permissions.
- System tray where supported.
- Native file/folder selection.
- Platform-specific functionality where necessary.

---

## Backend / Core

Use Rust for the application core.

Recommended libraries/categories:

- Tokio — async runtime.
- Axum — local HTTP server.
- Serde — serialization/deserialization.
- SQLite — local persistent database.
- tracing — application logging.
- FFmpeg / ffprobe — media analysis and transcoding.
- mDNS/DNS-SD implementation — LAN node discovery.
- UUID or cryptographically random identifiers.
- Secure token generation for node pairing.

The Rust core should be designed so that the business logic is not tightly coupled to Tauri.

The same core should eventually be reusable by:

- Tauri desktop application.
- Tauri mobile application.
- Headless server.
- CLI server.
- NAS/server distribution.

---

# 3. High-Level Architecture

```text
                     Local Network
                         |
        +----------------+----------------+
        |                |                |
        v                v                v
   LocalStream       LocalStream       Browser
   Node A            Node B            Client
        |                |
        +------- LAN ----+
                 |
                 v
          Distributed Library
                 |
                 v
            Media Streaming
```

Each LocalStream installation is a **Node**.

A node may act as:

- Media provider.
- Media consumer.
- Web server.
- Media player.
- Peer discovery participant.

A node should expose only media folders explicitly approved by the user.

---

# 4. Core Architectural Rule

The frontend must not contain core media/server logic.

Use this separation:

```text
Vue 3 / TypeScript
        |
        | Tauri commands
        | HTTP
        | WebSocket
        v
     Rust Core
        |
        +-- media library
        +-- streaming
        +-- server
        +-- networking
        +-- discovery
        +-- database
        +-- FFmpeg
        +-- authentication
```

The same Rust services should be callable from both:

1. Tauri commands.
2. HTTP API routes.

Do not implement duplicate business logic for the Tauri and HTTP interfaces.

Example:

```text
Tauri Command ----+
                  |
                  v
            LibraryService
                  ^
                  |
HTTP API ---------+
```

---

# 5. Suggested Repository Structure

Start with a single repository.

```text
localstream/
|
+-- src/
|   +-- assets/
|   +-- components/
|   +-- composables/
|   +-- layouts/
|   +-- pages/
|   +-- router/
|   +-- services/
|   +-- types/
|   +-- utils/
|   +-- App.vue
|   +-- main.ts
|
+-- src-tauri/
|   +-- src/
|   |   +-- commands/
|   |   +-- core/
|   |   +-- database/
|   |   +-- discovery/
|   |   +-- media/
|   |   +-- network/
|   |   +-- platform/
|   |   +-- security/
|   |   +-- streaming/
|   |   +-- transcoding/
|   |   +-- app.rs
|   |   +-- lib.rs
|   |   +-- main.rs
|   |
|   +-- capabilities/
|   +-- Cargo.toml
|   +-- tauri.conf.json
|
+-- docs/
|   +-- architecture/
|   +-- api/
|   +-- development/
|   +-- security/
|
+-- public/
+-- tests/
+-- package.json
+-- vite.config.ts
+-- tsconfig.json
+-- README.md
```

Do not create unnecessary layers prematurely.

AI agents may simplify the structure during the earliest milestone, but the separation of responsibilities described in this document must remain intact.

---

# 6. Mandatory Directory Documentation Rule

Every meaningful source-code directory created by an AI agent must contain a small `README.md`.

The mini-documentation should explain:

1. The purpose of the directory.
2. The features implemented there.
3. Important files.
4. Public APIs or important exported types.
5. Dependencies on other modules.
6. Current limitations.
7. Planned or unfinished work if applicable.

Example:

```text
src-tauri/src/streaming/
├── README.md
├── mod.rs
├── direct_play.rs
└── range.rs
```

Example `README.md`:

```md
# Streaming

This directory contains LocalStream media streaming functionality.

## Features

- Direct Play
- HTTP byte-range support
- MIME type detection
- Stream request validation

## Important Files

- `direct_play.rs` — streams supported source media directly.
- `range.rs` — parses and handles HTTP Range requests.

## Dependencies

- media
- security
- database

## Limitations

HLS transcoding is not implemented yet.
```

This documentation must remain concise and practical.

Do not require mini-documentation inside generated or dependency directories such as:

- `node_modules`
- `target`
- `dist`
- generated Tauri mobile projects
- build directories
- temporary/cache directories

Agents must update the local `README.md` whenever the responsibilities or public behavior of that directory materially changes.

---

# 7. Frontend Responsibilities

## `src/components`

Contains reusable presentation components.

Examples:

- MediaCard
- MediaGrid
- MediaList
- NodeCard
- ServerStatus
- PlayerControls
- SearchBar
- FolderPickerButton
- ConnectionStatus
- PairingDialog
- EmptyState
- LoadingIndicator

Components should avoid owning application-level business logic.

---

## `src/composables`

Contains reusable Vue Composition API logic.

Suggested composables:

### `useMediaLibrary`

Responsibilities:

- Fetch library.
- Refresh library.
- Search/filter locally.
- Track loading and error state.

### `useNodes`

Responsibilities:

- Load discovered nodes.
- Track connected nodes.
- Pair/unpair nodes.
- Track node availability.

### `useServer`

Responsibilities:

- Read server status.
- Start server.
- Stop server.
- Expose local server URL.
- Expose network addresses.

### `usePlayer`

Responsibilities:

- Current media.
- Playback state.
- Resume position.
- Volume.
- Playback speed.
- Subtitle selection.
- Stream URL generation.

### `useSettings`

Responsibilities:

- UI settings.
- Server settings.
- Library preferences.
- Local device preferences.

Use Vue primitives directly rather than Pinia.

---

# 8. Frontend Pages

Recommended initial pages:

```text
/
 /library
 /movies
 /music
 /nodes
 /settings
 /player/:id
```

Possible future pages:

```text
/series
/artists
/albums
/history
/downloads
/users
/admin
```

---

## Home

Should show:

- Continue watching.
- Recently added.
- Local node status.
- Available peer nodes.
- Quick access to media categories.

---

## Library

Should show:

- Media from the current node.
- Media from connected trusted nodes.
- Filters.
- Search.
- Source node information when useful.

The UI should eventually support a unified library where the physical file location does not dominate the user experience.

---

## Nodes

Should show:

- Current device/node.
- Discovered nodes.
- Paired nodes.
- Connection state.
- Node capabilities.
- Pairing actions.
- Shared library counts.
- Last-seen state.

---

## Settings

Suggested categories:

- General.
- Server.
- Libraries.
- Network.
- Playback.
- Transcoding.
- Security.
- Device.
- Advanced.

---

# 9. Frontend Backend Abstraction

The Vue frontend may run in two contexts:

1. Inside Tauri.
2. In a normal browser served by LocalStream.

Therefore create a backend abstraction.

Example:

```ts
export interface LocalStreamBackend {
  getLibrary(): Promise<MediaItem[]>
  getNodes(): Promise<NodeInfo[]>
  getServerStatus(): Promise<ServerStatus>
}
```

Possible implementations:

```text
TauriBackend
HttpBackend
```

Inside Tauri:

```text
Vue
  |
invoke()
  |
Rust
```

Inside browser:

```text
Vue
  |
fetch()
  |
Rust HTTP Server
```

Do not spread environment-specific checks throughout components.

Centralize backend selection.

---

# 10. Rust Core Responsibilities

The Rust backend should contain the actual LocalStream domain logic.

Suggested major areas:

```text
core/
database/
discovery/
media/
network/
security/
streaming/
transcoding/
platform/
```

---

# 11. `core`

The core module contains domain-level services and models.

Suggested responsibilities:

- Application state.
- Shared service registry.
- Node identity.
- Core configuration.
- Media service.
- Library service.
- Node service.
- Server service.

Avoid placing HTTP or Tauri-specific behavior here.

Core APIs should remain reusable.

---

# 12. `media`

Responsible for local media discovery and metadata.

Features:

- Scan approved folders.
- Detect supported media.
- Identify files by internal ID.
- Read basic metadata.
- Detect MIME/container.
- Read duration.
- Read codec information.
- Read audio tracks.
- Read subtitle tracks.
- Read resolution.
- Read bitrate when available.
- Generate or cache thumbnails later.

Initial supported categories:

- Video.
- Audio.

Initial file types may include:

```text
.mp4
.mkv
.webm
.mov
.m4v
.mp3
.flac
.m4a
.ogg
.wav
```

Do not assume extension alone guarantees compatibility.

Use ffprobe where deeper inspection is required.

---

# 13. Library Scanner

The scanner must:

1. Only scan directories approved by the user.
2. Recursively scan when configured.
3. Avoid uncontrolled traversal outside allowed directories.
4. Ignore temporary/system files.
5. Detect deleted files.
6. Detect newly added files.
7. Update changed metadata.
8. Avoid full rescans when incremental scanning is possible.

Future optimization may include:

- File watchers.
- Content hashing.
- Partial hashes.
- Directory modification tracking.

---

# 14. Media Identity

Never expose raw local filesystem paths as media identifiers.

Bad:

```text
/api/play?path=C:\Users\User\Videos\movie.mkv
```

Good:

```text
/api/media/018f3c.../stream
```

Use internal media IDs.

Example model:

```text
MediaItem
- id
- node_id
- library_id
- media_type
- title
- relative_location
- duration
- container
- video_codec
- audio_codec
- width
- height
- size
- created_at
- updated_at
```

The exact filesystem path must remain internal to the server.

---

# 15. Database

Use SQLite initially.

Suggested entities:

```text
Node
Library
MediaItem
MediaTrack
Peer
Pairing
PlaybackProgress
Setting
Thumbnail
```

Potential tables:

### Node

- id
- name
- created_at
- public_key or device identity metadata
- version

### Library

- id
- name
- local_path
- media_type
- enabled

### MediaItem

- id
- library_id
- title
- relative_path or internal locator
- size
- duration
- container
- video_codec
- audio_codec
- width
- height
- modified_at

### Peer

- id
- node_id
- name
- address
- paired
- last_seen

### PlaybackProgress

- id
- media_id
- profile/user if later supported
- position
- duration
- updated_at

Database migrations must be versioned.

Do not manually mutate production schema without migrations.

---

# 16. Local HTTP Server

Use Axum.

The LocalStream node should expose a LAN HTTP server.

Example:

```text
http://192.168.1.20:8090
```

The server should provide:

1. Web UI.
2. REST API.
3. Media streaming endpoints.
4. Node discovery information.
5. Pairing endpoints.
6. Optional WebSocket/event channel.

---

# 17. Initial HTTP API

Suggested initial API:

```http
GET /api/v1/server/info
GET /api/v1/server/status

GET /api/v1/library
GET /api/v1/media
GET /api/v1/media/:id

GET /api/v1/media/:id/stream

GET /api/v1/nodes
GET /api/v1/nodes/:id

POST /api/v1/pair
POST /api/v1/pair/confirm

GET /api/v1/playback/:mediaId
PUT /api/v1/playback/:mediaId
```

Routes may evolve.

All API endpoints should be versioned under `/api/v1`.

---

# 18. Web Interface Serving

The Rust HTTP server should be able to serve the production Vue build.

Example:

```text
GET /
GET /assets/*
```

This allows any browser on the LAN to open:

```text
http://192.168.1.20:8090
```

and use the LocalStream interface without installing an application.

The same Vue application should be reused where practical.

---

# 19. Streaming

Streaming should be implemented incrementally.

## Phase 1: Direct Play

Preferred path:

```text
media file
   |
   v
HTTP Range
   |
   v
browser/player
```

The server must support HTTP byte-range requests.

Do not load entire media files into memory.

Streams should use bounded buffering.

---

## HTTP Range

Support requests such as:

```http
Range: bytes=1000000-
```

The server should respond appropriately with:

```http
206 Partial Content
```

and headers including:

```text
Content-Range
Accept-Ranges
Content-Length
Content-Type
```

Range parsing must be validated.

Malformed or unsafe requests must not cause crashes or excessive resource use.

---

# 20. Direct Play Decision

Before transcoding, determine whether the target client can directly play the media.

Potential inputs:

- Container.
- Video codec.
- Audio codec.
- Browser/client type.
- Resolution.
- Subtitle format.
- Network constraints.

Initial implementation may use simple compatibility rules.

Later versions may include client capability negotiation.

---

# 21. Transcoding

Do not make transcoding mandatory for the first MVP.

When introduced, FFmpeg should handle:

- Video transcoding.
- Audio transcoding.
- Container remuxing.
- Resolution scaling.
- Bitrate changes.
- Subtitle processing where needed.
- HLS generation.

Suggested pipeline:

```text
Source Media
    |
 ffprobe
    |
Compatibility Decision
    |
 +-- compatible --> Direct Play
 |
 +-- incompatible --> FFmpeg
                        |
                        v
                      HLS
                        |
                        v
                     Client
```

---

# 22. FFmpeg Integration

Prefer invoking packaged FFmpeg/ffprobe binaries initially rather than deeply binding libav through FFI.

Responsibilities:

- Detect media metadata.
- Spawn transcoding jobs.
- Capture logs.
- Stop jobs.
- Track process lifetime.
- Limit concurrent transcodes.
- Clean temporary files.
- Report errors.

Desktop distributions may package FFmpeg as a Tauri sidecar where appropriate.

Agents must document FFmpeg licensing/build assumptions before commercial distribution.

---

# 23. Transcoding Job Manager

Future implementation should support:

```text
TranscodeManager
```

Responsibilities:

- Create jobs.
- Avoid duplicate jobs.
- Enforce concurrency limits.
- Cancel jobs when clients disconnect.
- Clean stale jobs.
- Track process status.
- Surface errors.
- Cache useful output if configured.

Do not spawn unlimited FFmpeg processes.

---

# 24. HLS

HLS may be used when transcoding or adaptive delivery is required.

Potential output:

```text
master.m3u8
1080p/
720p/
480p/
```

Adaptive streaming is not required for the first release.

Start with one output quality if necessary.

---

# 25. LAN Node Discovery

Nodes should automatically discover other LocalStream nodes.

Preferred mechanism:

- mDNS / DNS-SD.

Conceptual service:

```text
_localstream._tcp.local
```

Advertised metadata may contain:

- Node ID.
- Node name.
- Port.
- Protocol version.
- App version.
- Supported capabilities.

Do not advertise sensitive filesystem paths or secrets.

---

# 26. Peer Model

A discovered peer is not automatically trusted.

States may include:

```text
Discovered
Pairing
Trusted
Offline
Blocked
```

A peer should not automatically gain access to the full local library.

---

# 27. Pairing

Implement explicit user-approved pairing.

Possible flow:

```text
Node B -> requests access
Node A -> shows confirmation
Node A -> displays/verifies code
Nodes -> exchange secure credentials
Nodes -> become trusted
```

Potential UI:

```text
Laptop wants to connect.

Verification code:

482 914

[Allow] [Reject]
```

Avoid relying solely on source IP as authentication.

---

# 28. Security

Security is part of the architecture, not a later optional feature.

Core rules:

- Never expose arbitrary file paths.
- Only share explicitly approved media libraries.
- Validate all IDs.
- Validate HTTP Range headers.
- Sanitize metadata.
- Restrict filesystem access.
- Use secure random pairing tokens.
- Rate-limit sensitive endpoints where appropriate.
- Never trust LAN traffic merely because it is local.
- Do not expose control APIs to unauthenticated peers.
- Protect against directory traversal.
- Protect against malformed media metadata.
- Avoid shell command construction with untrusted values.

FFmpeg commands should use structured process arguments, not concatenated shell strings.

Bad:

```rust
Command::new("sh")
  .arg("-c")
  .arg(format!("ffmpeg -i {}", user_path))
```

Prefer:

```rust
Command::new("ffmpeg")
  .arg("-i")
  .arg(validated_path)
```

---

# 29. Distributed Library

One of the main differentiators of LocalStream should be distributed media.

Example:

```text
Desktop
- 300 movies

Laptop
- 120 episodes

Phone
- 800 songs
```

The UI may present:

```text
Movies: 300
Episodes: 120
Songs: 800
```

without requiring users to manually navigate each server.

Media internally must retain source node identity.

---

# 30. Remote Media Model

Example:

```text
RemoteMediaItem
- media_id
- node_id
- title
- type
- duration
- codecs
- source_capabilities
```

When playback begins:

```text
media_id
   |
node_id
   |
node address
   |
/api/v1/media/:id/stream
```

The receiving LocalStream node does not need access to the peer's physical filesystem path.

---

# 31. Network Resilience

Peers may disappear at any time.

The application must handle:

- Laptop sleeping.
- Wi-Fi changes.
- DHCP/IP changes.
- Node shutdown.
- Mobile network transition.
- Server restart.
- Temporary packet loss.

Do not treat connection failure as exceptional corruption.

Peer status should transition cleanly to offline/unavailable.

---

# 32. WebSocket / Events

Use WebSocket or another event mechanism only when useful.

Potential events:

```text
node.discovered
node.offline
library.updated
server.started
server.stopped
scan.started
scan.completed
transcode.started
transcode.completed
transcode.failed
```

Do not use WebSockets for functionality that works better as ordinary REST requests.

---

# 33. Playback

The browser/player should support:

- Play.
- Pause.
- Seek.
- Volume.
- Fullscreen.
- Playback progress.
- Resume playback.
- Basic subtitle selection when available.

Future features:

- Playback speed.
- Audio track selection.
- External subtitle loading.
- Picture-in-picture.
- Chapters.
- Casting.

---

# 34. Playback Progress

Persist playback progress locally.

Potential behavior:

- Save periodically.
- Save on pause.
- Save on page/app exit.
- Save when media ends.
- Mark completed near the end according to a defined threshold.

Progress should eventually synchronize across trusted nodes if a user chooses that behavior.

---

# 35. Music Support

Music should use the same core architecture where possible.

Features:

- Audio library.
- Albums.
- Artists.
- Songs.
- Duration.
- Album artwork if available.
- Queue.
- Play/pause.
- Seek.
- Next/previous.

Advanced music-library functionality can come after the video MVP.

---

# 36. Platform Layer

Platform-specific code should remain isolated.

Suggested:

```text
platform/
├── desktop.rs
├── android.rs
├── ios.rs
└── mod.rs
```

Examples:

- Android foreground service integration.
- iOS lifecycle constraints.
- Native folder selection.
- Notifications.
- Auto-start.
- Tray behavior.
- OS-specific networking permissions.

Core domain logic should not directly depend on platform APIs.

---

# 37. Android

Android should ideally support:

- Client mode.
- Server mode.
- Folder/library access through supported Android storage APIs.
- LAN discovery.
- Playback.
- Pairing.

When persistent server operation is needed, use the appropriate Android foreground-service model.

Do not assume unrestricted filesystem access.

Storage permissions and document-tree access must be handled according to Android APIs.

---

# 38. iOS

iOS should primarily be considered:

- A client.
- A controller.
- A foreground server where technically appropriate.

Do not architect the product around a permanently running arbitrary HTTP server in iOS background mode.

The UI should clearly communicate platform restrictions.

---

# 39. Desktop

Desktop platforms should support the full feature set first.

Priority:

1. Windows.
2. Linux.
3. macOS.

Desktop features should include:

- Server.
- Client.
- Web UI.
- File/folder libraries.
- Direct Play.
- Discovery.
- Pairing.
- FFmpeg.
- Tray/minimize behavior later.
- Headless server support later.

---

# 40. Headless Server

The Rust core must eventually support a headless binary.

Example:

```bash
localstream-server --port 8090
```

Possible future use:

- Linux server.
- Raspberry Pi.
- NAS.
- Docker.
- Mini PC.

Do not make the media/server core dependent on a graphical window.

---

# 41. Configuration

Configuration should include:

```text
server port
node name
enabled libraries
discovery enabled
transcoding enabled
transcode concurrency
temporary directory
logging level
pairing behavior
```

Configuration should be persistent.

Secrets should not be stored as plain settings if secure alternatives exist.

---

# 42. Logging

Use structured logging.

Recommended:

- `tracing`
- configurable log level

Useful scopes:

```text
server
media
scanner
database
discovery
streaming
transcoding
security
platform
```

Do not log:

- Authentication secrets.
- Pairing tokens.
- Sensitive filesystem information unnecessarily.

---

# 43. Error Handling

Rust code should prefer typed errors.

Avoid uncontrolled `.unwrap()` in runtime paths.

Errors should have:

- Internal details for logs.
- Safe user-facing messages.

Frontend should distinguish:

- Network error.
- Node unavailable.
- Media unsupported.
- Media removed.
- Pairing rejected.
- Server unavailable.
- Transcoding failed.

---

# 44. API Types

Avoid manually duplicating inconsistent types.

Maintain clear shared domain definitions.

Example frontend:

```ts
export interface MediaItem {
  id: string
  nodeId: string
  title: string
  type: 'video' | 'audio'
  duration?: number
}
```

Rust:

```rust
#[derive(Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub node_id: String,
    pub title: String,
    pub media_type: MediaType,
    pub duration: Option<f64>,
}
```

Agents should keep frontend and backend schemas synchronized.

Schema/code generation may be introduced later if useful.

---

# 45. API Versioning

Use:

```text
/api/v1/
```

from the beginning.

Peer protocol compatibility should eventually include:

```text
protocolVersion
appVersion
capabilities
```

A newer node must handle incompatible peers gracefully.

---

# 46. Capability Negotiation

Future node capabilities may include:

```text
direct-play
transcode
hls
subtitles
music
video
thumbnail
remote-control
```

A node should not assume every peer supports every feature.

---

# 47. MVP Scope

The first MVP should remain intentionally small.

The MVP is complete when this flow works:

```text
Desktop app starts
       |
User selects media folder
       |
Rust scans media
       |
Vue displays media
       |
Rust HTTP server starts
       |
Phone opens browser
       |
Phone sees media library
       |
User chooses video
       |
Video streams over LAN
```

---

# 48. MVP Features

Required:

- Vue 3 interface.
- Tauri desktop application.
- Rust backend.
- Select media folder.
- Scan video files.
- Store basic metadata.
- SQLite persistence.
- Start local Axum HTTP server.
- Serve Vue web interface.
- List media via HTTP API.
- Direct Play.
- HTTP Range support.
- Basic playback.
- Basic server settings.
- Basic error handling.

Not required initially:

- FFmpeg transcoding.
- HLS.
- DLNA.
- Chromecast.
- SMB.
- NFS.
- Cloud synchronization.
- Internet remote access.
- Multiple user accounts.
- Advanced metadata providers.
- TV-native application.
- AI features.

---

# 49. Milestone 1 — Local Desktop Playback

Goal:

```text
folder -> scanner -> library -> Vue -> local player
```

Features:

- Folder selection.
- Media scan.
- SQLite storage.
- Basic metadata.
- Library UI.
- Local playback.

---

# 50. Milestone 2 — LAN Web Server

Goal:

```text
PC -> LAN -> browser
```

Features:

- Axum server.
- Vue static hosting.
- API.
- Direct media streaming.
- HTTP Range.
- Server URL display.
- Network address detection.

---

# 51. Milestone 3 — Discovery

Goal:

```text
Node A <-> Node B
```

Features:

- mDNS advertisement.
- mDNS discovery.
- Node list.
- Online/offline tracking.
- Node identity.

---

# 52. Milestone 4 — Pairing and Trust

Features:

- Pair request.
- User confirmation.
- Secure peer token.
- Trusted peer persistence.
- Revoke peer.
- Reject unauthorized API access.

---

# 53. Milestone 5 — Distributed Library

Features:

- Query remote libraries.
- Merge local and remote media views.
- Show node availability.
- Stream remote media.
- Handle disappearing peers.

---

# 54. Milestone 6 — FFmpeg

Features:

- ffprobe metadata.
- Compatibility detection.
- Transcoding jobs.
- Remuxing.
- HLS where required.
- Transcode cancellation.
- Concurrency limits.

---

# 55. Milestone 7 — Mobile

Android first.

Features:

- Reuse Vue interface.
- Local playback.
- Peer discovery.
- Remote playback.
- Pairing.
- Server mode.
- Foreground-service integration where required.
- Android storage access.

Then iOS:

- Client.
- Discovery.
- Playback.
- Pairing.
- Foreground server where supported.
- Platform limitations clearly handled.

---

# 56. Milestone 8 — Product Features

Potential features:

- Continue watching.
- Rich media metadata.
- Posters.
- Thumbnails.
- Subtitle management.
- Music library.
- User profiles.
- Casting.
- TV apps.
- NAS support.
- Docker/headless distribution.
- Auto-update.
- Backup/export.
- Library synchronization.

These should not delay the core MVP.

---

# 57. Testing Strategy

Tests should cover high-risk logic.

Rust:

- Range parsing.
- Path authorization.
- Media ID resolution.
- API authorization.
- Pairing.
- Database migrations.
- Scanner behavior.
- Node capability parsing.

Frontend:

- Composables.
- API adapters.
- Critical components.
- Player state transitions.

Integration tests:

- Start server.
- Fetch library.
- Range-stream test media.
- Simulated peer discovery where feasible.

Do not require exhaustive UI testing before basic functionality exists.

---

# 58. Sample Media

Testing must use legally distributable sample media or generated test fixtures.

Do not commit copyrighted commercial movies/music to the repository.

---

# 59. Code Quality Rules for AI Agents

AI agents working on this project must:

1. Read the root `README.md`.
2. Read the `README.md` of the directory being modified.
3. Preserve architectural separation.
4. Avoid duplicate business logic.
5. Avoid unnecessary dependencies.
6. Avoid premature abstraction.
7. Prefer typed APIs.
8. Handle errors explicitly.
9. Avoid unchecked filesystem access.
10. Avoid exposing filesystem paths through APIs.
11. Add tests for security-sensitive parsing/validation.
12. Update directory documentation after meaningful changes.
13. Keep implementation scoped to the requested milestone.
14. Avoid implementing unrelated future features.
15. Do not silently change public API behavior.

---

# 60. Dependency Rules

Before adding a dependency, an agent should evaluate:

- Why it is necessary.
- Whether the standard library already solves the problem.
- Maintenance status.
- Cross-platform support.
- Mobile compatibility.
- Licensing implications.
- Binary size impact.
- Security implications.

Avoid adding multiple libraries that solve the same problem.

---

# 61. Documentation Rules

Root documentation should include:

```text
README.md
docs/
```

The root README should explain:

- What LocalStream is.
- Supported platforms.
- Current project status.
- Development prerequisites.
- How to run frontend.
- How to run Tauri.
- How to test.
- Architecture summary.
- Current limitations.

`docs/architecture` should contain deeper architectural decisions.

`docs/api` should describe the HTTP API.

`docs/security` should document the threat model and trust model.

`docs/development` should document setup and contributor workflows.

---

# 62. Architecture Decision Records

For significant architectural decisions, agents may create ADR files.

Example:

```text
docs/architecture/adr/
├── 0001-use-rust-core.md
├── 0002-use-axum.md
├── 0003-use-mdns.md
└── 0004-direct-play-first.md
```

Each ADR should contain:

- Context.
- Decision.
- Alternatives considered.
- Consequences.

Do not create ADRs for trivial implementation details.

---

# 63. Privacy Requirements

LocalStream should be local-first.

Default behavior:

- No cloud account required.
- No media uploaded externally.
- No required remote API.
- No mandatory telemetry.
- LAN operation should work without Internet access.

If telemetry is ever introduced, it must be:

- Explicit.
- Optional.
- Documented.
- Privacy-preserving.

---

# 64. Internet Access

Initial LocalStream functionality must not require Internet access.

External metadata providers may be added later as optional enhancements.

The application must still work when:

```text
Internet = unavailable
LAN = available
```

---

# 65. UX Principles

The product should aim for:

```text
Install
  |
Choose folder
  |
Server ready
```

Avoid requiring users to understand:

- Ports.
- Docker.
- Codec details.
- IP addresses.
- Filesystem permissions beyond necessary user prompts.
- FFmpeg configuration.

Advanced settings may expose these concepts later.

---

# 66. Server Address UX

Instead of only displaying:

```text
192.168.1.20:8090
```

prefer:

```text
LocalStream is available at:

http://192.168.1.20:8090

Devices on this Wi-Fi can open this address.
```

Future enhancement:

```text
http://localstream.local
```

where platform/network conditions support local hostname discovery.

QR-code access may also be added.

---

# 67. Non-Goals for Initial Development

Do not initially build:

- A Plex clone.
- A Netflix-like recommendation engine.
- A cloud service.
- DRM.
- Torrent functionality.
- Remote Internet streaming.
- Social features.
- Large-scale user accounts.
- Full video editor.
- AI recommendation engine.

The first goal is reliable local streaming.

---

# 68. Definition of a Node

A LocalStream Node is an installation with:

```text
Node ID
Node Name
Network Address
Protocol Version
Capabilities
Shared Libraries
Trust Relationships
```

Each installation generates and persists a stable node identity.

Node identity should not depend solely on hostname or IP address.

---

# 69. Node Lifecycle

Conceptual states:

```text
Starting
Ready
Serving
Degraded
Stopping
Stopped
```

Peer states:

```text
Discovered
Trusted
Online
Offline
Blocked
```

State transitions should be explicit where useful.

---

# 70. Application Startup

Potential startup sequence:

```text
Load configuration
      |
Open database
      |
Load node identity
      |
Initialize services
      |
Start discovery
      |
Optionally start HTTP server
      |
Load library
      |
Notify frontend
```

Failure of a non-critical service should not necessarily terminate the entire application.

Example:

mDNS failure may disable discovery while local playback continues.

---

# 71. Shutdown

Shutdown should:

- Stop accepting new server work.
- Stop discovery.
- Cancel active background tasks.
- Persist pending state.
- Terminate FFmpeg jobs.
- Close database resources.
- Stop HTTP server cleanly.

Avoid orphaned FFmpeg processes.

---

# 72. Resource Management

The application may serve very large files.

Therefore:

- Never load full videos into RAM.
- Use streaming I/O.
- Bound buffers.
- Limit transcodes.
- Clean temporary files.
- Avoid unnecessary media duplication.
- Track disk usage from transcode caches.

---

# 73. Future Hardware Acceleration

Future transcoding support may detect:

- NVIDIA NVENC.
- Intel Quick Sync.
- AMD hardware encoding.
- Apple VideoToolbox.
- Android hardware codecs where feasible.

Software transcoding should remain the fallback.

Hardware acceleration is not part of the MVP.

---

# 74. Subtitle Support

Future subtitle support should include:

- Embedded tracks.
- External `.srt`.
- Web-compatible subtitle delivery.
- Track selection.

Avoid transcoding subtitles into video unless necessary.

---

# 75. Thumbnail and Poster System

Future implementation may:

- Extract thumbnails with FFmpeg.
- Cache generated thumbnails.
- Generate preview frames.
- Support external poster metadata.

Thumbnail generation must occur asynchronously and must not block library browsing.

---

# 76. File Watching

After initial scanning is stable, add filesystem watchers where supported.

Expected events:

- Created.
- Modified.
- Removed.
- Renamed.

File watchers are an optimization.

Periodic reconciliation should remain possible because watchers can miss events.

---

# 77. SMB / NFS / WebDAV

Support may be added later.

Treat external network filesystems as library providers.

Do not make SMB/NFS required for peer-to-peer LocalStream sharing.

Preferred first approach:

```text
LocalStream Node
      |
LocalStream API
      |
Peer Node
```

This keeps permissions and protocol behavior under application control.

---

# 78. DLNA / UPnP

DLNA/UPnP may later provide compatibility with TVs and legacy media devices.

Keep it outside the core domain.

Implement as an adapter over the media library and streaming services.

---

# 79. Casting

Potential future targets:

- Chromecast.
- AirPlay where feasible.
- DLNA renderers.
- LocalStream-to-LocalStream remote playback.

Casting must be an optional feature layered over the playback architecture.

---

# 80. Suggested Initial Rust Modules

A practical first version may begin with:

```text
src-tauri/src/
├── main.rs
├── app.rs
├── commands.rs
├── server.rs
├── library.rs
├── streaming.rs
├── database.rs
└── discovery.rs
```

Do not force the full directory architecture on day one.

Refactor when responsibilities become sufficiently large.

When a file becomes a directory/module, add the mandatory directory `README.md`.

---

# 81. Suggested Initial Vue Structure

```text
src/
├── components/
│   ├── README.md
│   ├── MediaCard.vue
│   └── ServerStatus.vue
│
├── composables/
│   ├── README.md
│   ├── useLibrary.ts
│   └── useServer.ts
│
├── pages/
│   ├── README.md
│   ├── HomePage.vue
│   ├── LibraryPage.vue
│   └── SettingsPage.vue
│
├── services/
│   ├── README.md
│   ├── backend.ts
│   ├── httpBackend.ts
│   └── tauriBackend.ts
│
├── types/
│   ├── README.md
│   └── media.ts
│
├── App.vue
└── main.ts
```

---

# 82. Vue State Guidance

Do not introduce Pinia.

Use local state whenever possible:

```ts
const loading = ref(false)
```

Use composables for reusable feature state:

```ts
const {
  media,
  loading,
  refresh,
} = useMediaLibrary()
```

For app-wide shared state, use one of:

- Module-scoped refs inside a composable.
- `provide` / `inject`.
- A simple reactive service.

Example:

```ts
const nodes = ref<NodeInfo[]>([])

export function useNodes() {
  return {
    nodes,
  }
}
```

Do not create global state merely because data exists.

Prefer ownership close to the feature that consumes the state.

---

# 83. Agent Implementation Workflow

When an AI agent receives a task:

1. Identify the milestone and feature.
2. Read relevant README documentation.
3. Inspect existing interfaces.
4. Implement the smallest cohesive change.
5. Add or update tests.
6. Update local directory README.
7. Update API documentation if applicable.
8. Run formatting/linting/tests.
9. Report changed behavior and remaining limitations.

Agents should not rewrite large unrelated sections unless explicitly instructed.

---

# 84. Completion Criteria for Features

A feature is not considered complete until:

- Code compiles.
- Relevant tests pass.
- Error paths are handled.
- Public API behavior is documented.
- Directory README is updated where necessary.
- Security implications are considered.
- Cross-platform limitations are documented.

---

# 85. Naming Guidelines

Prefer clear domain terminology.

Good:

```text
MediaLibrary
MediaItem
Node
Peer
StreamSession
TranscodeJob
PlaybackProgress
PairingRequest
```

Avoid vague names:

```text
Manager2
Helper
UtilsManager
DataThing
CommonService
```

Utility modules should remain small and specific.

---

# 86. API Error Format

Use a consistent API error format.

Example:

```json
{
  "error": {
    "code": "MEDIA_NOT_FOUND",
    "message": "The requested media item does not exist."
  }
}
```

Do not expose internal Rust errors, SQL details, stack traces, or filesystem paths to remote clients.

---

# 87. Server Capabilities Endpoint

Suggested:

```http
GET /api/v1/server/info
```

Example response:

```json
{
  "nodeId": "node-123",
  "name": "Living Room PC",
  "protocolVersion": 1,
  "appVersion": "0.1.0",
  "capabilities": [
    "video",
    "audio",
    "direct-play"
  ]
}
```

Later:

```text
transcode
hls
subtitles
casting
```

---

# 88. Local Network First

Every architecture decision should preserve this scenario:

```text
Router
 |
 +-- Windows PC
 +-- Android phone
 +-- Linux laptop
 +-- Smart TV browser

Internet connection: NONE
```

LocalStream should still provide its core functionality.

---

# 89. Long-Term Product Direction

LocalStream may eventually become:

```text
LocalStream Desktop
LocalStream Mobile
LocalStream Server
LocalStream Web
LocalStream TV
```

All sharing:

```text
LocalStream Core
LocalStream Protocol
LocalStream Library Model
```

The initial repository should not prematurely implement all distributions, but architectural choices must not unnecessarily prevent them.

---

# 90. First Development Target

AI agents should prioritize achieving the following end-to-end demonstration:

1. Launch LocalStream on a desktop computer.
2. Select a folder containing test videos.
3. Scan the folder.
4. Display the videos in Vue.
5. Start the LAN server.
6. Display the LAN URL.
7. Open that URL from a phone browser.
8. Display the same media library.
9. Select a compatible MP4 file.
10. Stream it using HTTP Range.
11. Seek forward and backward successfully.
12. Restart the server and retain the indexed library in SQLite.

Once this works reliably, move to node discovery.

---

# 91. Instructions to AI Agents

When implementing LocalStream:

- Build incrementally.
- Keep the application runnable after each meaningful change.
- Favor working end-to-end paths over incomplete large abstractions.
- Keep Rust core logic separate from transport layers.
- Keep Vue UI separate from backend implementation details.
- Do not introduce Pinia.
- Use Vue Composition API state.
- Prefer Direct Play before transcoding.
- Prefer explicit user-approved sharing.
- Assume LAN peers may be hostile until paired.
- Never expose arbitrary filesystem access.
- Document every meaningful source directory with a concise `README.md`.
- Update documentation when features change.
- Record significant architecture decisions.
- Keep mobile constraints in mind from the beginning.
- Do not sacrifice desktop MVP progress to prematurely solve every mobile limitation.
- Do not implement cloud requirements for a local-first product.

---

# 92. Summary Architecture

```text
                        LocalStream
                            |
              +-------------+-------------+
              |                           |
              v                           v
       Vue 3 + TypeScript              Rust Core
              |                           |
              |                           +-- SQLite
              |                           +-- Media Scanner
              |                           +-- Streaming
              |                           +-- mDNS
              |                           +-- Security
              |                           +-- FFmpeg
              |                           +-- HTTP Server
              |                           |
              +-----------+---------------+
                          |
                        Tauri
                          |
          +---------------+----------------+
          |               |                |
       Windows          Linux            macOS
          |
       Android
          |
        iOS*
```

`*` iOS server behavior must respect platform background-execution limitations.

The primary architectural principle is:

> **One reusable local-media core, multiple ways to access it.**

The Vue application handles the user experience.

Rust handles media, filesystem access, networking, persistence, discovery, and streaming.

Tauri provides the native cross-platform application shell.

The LocalStream HTTP server makes the same media system accessible from any browser on the local network.
