# LocalStream — Code Quality, Testing, and Verification Standard

## 1. Purpose

This document defines the mandatory code-quality, testing, verification, and validation practices for the LocalStream project.

It is intended for:

- AI coding agents.
- Human contributors.
- Code reviewers.
- CI/CD workflows.
- Release validation.

The purpose is to ensure that every implementation step preserves:

- Correctness.
- Security.
- Maintainability.
- Cross-platform compatibility.
- Performance.
- Architectural consistency.
- Testability.
- Reliability.

Quality must be treated as part of implementation, not as a final cleanup phase.

---

# 2. Core Quality Principle

Every meaningful code change must follow this cycle:

```text
Understand
   |
Design
   |
Implement
   |
Format
   |
Lint
   |
Test
   |
Verify behavior
   |
Review impact
   |
Document
   |
Commit-ready
```

No feature should be considered complete merely because it compiles.

---

# 3. Mandatory Rule for AI Agents

Before modifying code, every AI agent must:

1. Read the root project `README.md`.
2. Read the relevant directory `README.md`.
3. Identify the milestone being worked on.
4. Identify affected modules.
5. Identify relevant tests.
6. Identify security implications.
7. Identify cross-platform implications.
8. Avoid unrelated refactors unless required.

After modifying code, every AI agent must:

1. Format changed code.
2. Run applicable linters.
3. Run affected tests.
4. Run broader tests when shared code changed.
5. Verify compilation.
6. Verify runtime behavior where feasible.
7. Update documentation.
8. Report any unverified assumptions or limitations.

---

# 4. Definition of Done

A task is complete only when all applicable items below are satisfied:

- Code compiles.
- Type checks pass.
- Formatting passes.
- Linting passes.
- Unit tests pass.
- Integration tests pass where relevant.
- Security-sensitive behavior has tests.
- Error paths have been considered.
- No unnecessary warnings remain.
- Public interfaces are documented.
- Directory documentation is updated.
- Relevant API documentation is updated.
- No known regression is introduced.
- The feature is manually validated when automated testing cannot fully cover it.
- Platform-specific limitations are documented.

A successful build alone is not sufficient.

---

# 5. Quality Gates

The project should use progressive quality gates.

## Gate 1 — Static Correctness

Must verify:

- Rust compilation.
- TypeScript type checking.
- Vue compilation.
- No obvious dead imports.
- No syntax errors.
- No broken module references.

Suggested commands:

```bash
cargo check
npm run typecheck
npm run build
```

Exact scripts may vary according to the repository.

---

## Gate 2 — Formatting

All code must use project-standard formatters.

Rust:

```bash
cargo fmt --check
```

TypeScript / Vue:

Use the configured formatter, for example:

```bash
npm run format:check
```

Agents must not introduce manually inconsistent formatting.

---

## Gate 3 — Linting

Rust:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Frontend:

Use ESLint or the configured equivalent.

Example:

```bash
npm run lint
```

Warnings should not be ignored without documented justification.

Do not disable lint rules broadly to hide problems.

---

## Gate 4 — Unit Tests

Run targeted tests first.

Rust:

```bash
cargo test
```

Frontend:

```bash
npm run test
```

Unit tests should cover pure logic and important boundary behavior.

---

## Gate 5 — Integration Tests

Integration tests should verify interactions between modules.

Examples:

- HTTP API + database.
- Streaming endpoint + media file.
- Pairing + authentication.
- Library scan + SQLite persistence.
- Node discovery protocol parsing.
- API adapter + mock backend.

---

## Gate 6 — End-to-End Verification

Critical user flows should eventually be covered by end-to-end or system tests.

Example:

```text
Start LocalStream
      |
Select media folder
      |
Scan library
      |
Start LAN server
      |
Open browser client
      |
Fetch library
      |
Play media
      |
Seek
      |
Resume playback
```

Not every feature requires a full automated E2E test immediately, but major workflows must eventually have system-level coverage.

---

# 6. Test Pyramid

Prefer:

```text
           Few
       End-to-End Tests
            /\
           /  \
      Integration Tests
         /        \
        /          \
       Unit Tests
          Many
```

Most tests should remain fast and focused.

Avoid relying exclusively on slow E2E tests.

---

# 7. Rust Testing Standards

Rust modules should use unit tests for local behavior.

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_range() {
        // ...
    }
}
```

Integration tests should live in appropriate test modules or the project-level `tests/` directory.

Important Rust areas requiring strong tests:

- HTTP Range parsing.
- Path validation.
- File authorization.
- Media identifier resolution.
- Pairing token validation.
- Authentication.
- Database migrations.
- Scanner behavior.
- Node capability parsing.
- Transcoding command construction.
- Error conversion.
- Configuration parsing.

---

# 8. Frontend Testing Standards

Vue/TypeScript tests should focus on:

- Composables.
- Backend adapters.
- State transitions.
- Error handling.
- Important reusable components.
- Player state.
- Node state.
- Server state.

Avoid testing Vue implementation details unnecessarily.

Prefer testing observable behavior.

Example targets:

```text
useMediaLibrary
useNodes
useServer
usePlayer
HttpBackend
TauriBackend
```

---

# 9. TypeScript Quality Rules

TypeScript should remain strict.

Avoid:

```ts
any
```

unless unavoidable and locally justified.

Prefer:

```ts
unknown
```

followed by validation.

Public API responses should be typed.

Do not assume server responses are valid merely because TypeScript defines an interface.

Runtime data from:

- HTTP.
- WebSocket.
- Tauri.
- Local storage.
- External metadata.

should be treated as untrusted input.

---

# 10. Rust Quality Rules

Avoid:

```rust
unwrap()
expect()
panic!()
```

in normal runtime paths.

They may be acceptable in:

- Tests.
- Truly impossible invariant cases with explicit reasoning.
- Initialization where failure is intentionally fatal and clearly documented.

Prefer:

```rust
Result<T, E>
```

and typed errors.

Errors should preserve useful internal context while exposing safe messages to users.

---

# 11. Error-Path Testing

Every important feature should include failure cases.

For example, a media stream test suite should include:

- Existing media.
- Missing media.
- Unauthorized media.
- Invalid media ID.
- Invalid Range header.
- Range beyond file size.
- Empty file.
- Client disconnect.
- File removed during request.

Success-only testing is not sufficient.

---

# 12. Boundary Testing

Test important boundaries explicitly.

Examples:

```text
0-byte file
1-byte file
very large file
first byte
last byte
full-range request
partial-range request
invalid UTF-8 metadata
empty library
large library
offline peer
duplicate peer
expired token
```

Bugs often occur at boundaries rather than normal inputs.

---

# 13. Security Testing

Security-sensitive code requires dedicated tests.

Mandatory areas include:

## Filesystem Access

Test:

- Allowed path.
- Path outside approved library.
- `../` traversal.
- Symlink behavior.
- Encoded traversal attempts.
- Removed files.
- Invalid IDs.

Never trust user-provided paths.

---

## HTTP Range

Test:

- Valid single range.
- Missing end.
- Invalid unit.
- Negative-like values.
- Start greater than end.
- Range beyond file.
- Extremely large numeric input.
- Malformed values.

Parsing must not panic.

---

## Pairing

Test:

- Valid pairing.
- Invalid code.
- Expired code.
- Replayed request.
- Unknown peer.
- Revoked peer.
- Unauthorized access before pairing.

---

## API Authorization

Test:

- Trusted peer.
- Unknown peer.
- Missing credentials.
- Invalid credentials.
- Revoked credentials.
- Insufficient capability.

---

# 14. Fuzz Testing

Fuzzing is strongly recommended for parsers and untrusted input.

High-value fuzz targets:

- HTTP Range parser.
- Discovery packet parser.
- Pairing messages.
- API request deserialization.
- Media metadata parser wrappers.
- Configuration parsing.

Rust fuzzing tools may be introduced when those modules become stable enough.

Fuzz targets should prioritize code that consumes network-controlled data.

---

# 15. Property-Based Testing

Property-based tests are useful for logic with many input combinations.

Examples:

- Range normalization.
- Media duration conversions.
- Path canonicalization assumptions.
- Pagination.
- Capability sets.
- Byte offset calculations.

Use property-based testing when it provides better coverage than manually enumerated cases.

---

# 16. Database Testing

Database behavior must be tested using isolated test databases.

Never run automated tests against user databases.

Tests should cover:

- Fresh database initialization.
- Migration from previous schema.
- Insert.
- Update.
- Delete.
- Unique constraints.
- Foreign keys.
- Transaction rollback.
- Corrupted or invalid input handling where practical.

Each migration should have a verification test.

---

# 17. Migration Policy

Database schema changes must:

1. Add a migration.
2. Preserve existing user data where possible.
3. Be reversible where practical.
4. Include migration tests.
5. Update schema documentation.

Never silently rebuild a user's database as a normal migration strategy unless the product explicitly supports that behavior.

---

# 18. Streaming Tests

Streaming is a critical subsystem.

Tests should verify:

- Correct HTTP status.
- Correct content type.
- Correct `Content-Length`.
- Correct `Content-Range`.
- Correct `Accept-Ranges`.
- Exact streamed bytes.
- Seeking behavior.
- Concurrent clients.
- Client disconnect cleanup.

Use small generated binary/media fixtures in automated tests.

Do not require large movie files.

---

# 19. Media Scanner Tests

Scanner tests should cover:

- Empty directory.
- One valid media file.
- Unsupported files.
- Nested directories.
- Duplicate file names.
- File deletion.
- File rename.
- Modified file.
- Permission error.
- Broken symlink.
- Large number of directory entries.

The scanner must fail gracefully when a specific file is unreadable.

One bad file should not necessarily stop the entire scan.

---

# 20. FFmpeg Verification

When FFmpeg integration is introduced, test the wrapper independently.

Verify:

- Binary availability.
- ffprobe JSON parsing.
- Valid media inspection.
- Invalid media inspection.
- Process failure.
- Process cancellation.
- Timeout behavior.
- Output cleanup.
- Argument safety.

Never build FFmpeg commands through unescaped shell strings.

Test generated argument lists where possible.

---

# 21. Transcoding Tests

Transcoding tests should use small fixtures.

Verify:

- Start job.
- Complete job.
- Cancel job.
- Failed FFmpeg process.
- Duplicate request behavior.
- Concurrency limits.
- Temporary file cleanup.
- Cache behavior if enabled.

Do not make all unit tests depend on FFmpeg being installed.

Separate:

```text
pure unit tests
FFmpeg integration tests
```

---

# 22. Network Discovery Tests

mDNS/discovery logic should separate:

```text
network transport
protocol parsing
peer state management
```

Protocol parsing and peer-state logic should be testable without real multicast networking.

Test:

- New node discovered.
- Existing node refreshed.
- Node changes IP.
- Node disappears.
- Duplicate advertisement.
- Unsupported protocol version.
- Malformed metadata.

---

# 23. Web API Contract Tests

Every public API endpoint should have contract tests.

Verify:

- HTTP method.
- Route.
- Status codes.
- Request structure.
- Response structure.
- Error structure.
- Authorization.

Example:

```http
GET /api/v1/media/:id
```

should have tests for:

```text
200 valid media
404 missing media
401/403 unauthorized
400 invalid identifier
```

---

# 24. API Compatibility

Public API changes must be intentional.

Agents must not silently:

- Rename fields.
- Remove fields.
- Change field types.
- Change status-code semantics.
- Change authentication behavior.

Breaking changes require:

- Explicit decision.
- Versioning strategy.
- Documentation update.
- Tests.

---

# 25. Tauri Command Tests

Core business logic should not exist exclusively inside Tauri command functions.

Bad:

```text
#[tauri::command]
scan_everything_and_update_database_and_emit_events()
```

Prefer:

```text
Tauri command
    |
LibraryService
```

Then `LibraryService` can be tested independently.

Tauri commands should remain thin adapters.

---

# 26. Backend Adapter Tests

The frontend has multiple backend transports:

```text
TauriBackend
HttpBackend
```

Both should satisfy the same conceptual contract.

Tests should verify equivalent behavior where applicable.

Example:

```text
getLibrary()
getNodes()
getServerStatus()
```

Transport differences should not leak unnecessarily into Vue components.

---

# 27. Vue Composable Tests

Composable tests should verify behavior such as:

```text
initial state
loading state
success state
error state
retry
refresh
state cleanup
```

Example:

```text
useMediaLibrary()
```

should be tested against a mocked backend interface rather than a real HTTP server in unit tests.

---

# 28. UI Component Testing

Only important reusable interaction components require direct component tests initially.

Examples:

- Pairing dialog.
- Server control.
- Media player controls.
- Error states.
- Node connection state.

Avoid spending excessive effort testing static markup.

---

# 29. Manual Verification

Some behaviors require manual verification.

For each milestone, maintain a short manual checklist.

Example for LAN streaming:

```text
[ ] Start server on Windows.
[ ] Connect phone to same Wi-Fi.
[ ] Open displayed server URL.
[ ] Library loads.
[ ] Play video.
[ ] Seek to middle.
[ ] Pause/resume.
[ ] Reload page.
[ ] Server remains stable.
```

Manual checks should complement, not replace, automated tests.

---

# 30. Cross-Platform Verification

Platform-specific behavior must be tested on actual target platforms before claiming support.

Minimum target matrix over time:

```text
Windows
Linux
macOS
Android
iOS
```

Not every commit requires all platforms locally, but release validation must account for them.

Platform behavior that cannot be verified must be documented as unverified.

---

# 31. Platform Matrix

Maintain a compatibility matrix.

Example:

| Feature | Windows | Linux | macOS | Android | iOS |
|---|---|---|---|---|---|
| Local playback | Yes | Yes | Yes | Yes | Yes |
| LAN server | Yes | Yes | Yes | Yes* | Limited* |
| Folder selection | Yes | Yes | Yes | Scoped | Scoped |
| mDNS discovery | Yes | Yes | Yes | Yes | Yes |
| Background server | Yes | Yes | Yes | Foreground service | Restricted |

`*` Platform restrictions must be documented.

Agents must not mark features supported without evidence.

---

# 32. Performance Verification

Performance matters because LocalStream processes large files and concurrent network streams.

Key measurements:

- Startup time.
- Library scan time.
- Memory usage.
- CPU usage.
- Direct Play throughput.
- Concurrent streams.
- Database query latency.
- Thumbnail-generation cost.
- Transcoding CPU/GPU load.
- Temporary disk usage.

Do not optimize blindly.

Measure first.

---

# 33. Performance Regression Tests

Where feasible, add benchmarks for hot paths.

Potential Rust benchmarks:

- Range parsing.
- Media index lookups.
- Library filtering.
- Metadata normalization.
- Large library serialization.

Major performance regressions should be investigated before merging.

---

# 34. Memory Safety and Resource Usage

Rust provides memory-safety guarantees, but application-level resource exhaustion remains possible.

Test and review for:

- Unbounded queues.
- Unbounded buffers.
- Unlimited concurrent tasks.
- Unlimited FFmpeg jobs.
- Never-ending retries.
- Large JSON responses.
- Large in-memory media data.
- Unclosed files.
- Orphaned child processes.

All concurrency must have defined limits where resource growth is possible.

---

# 35. Concurrency Testing

Important concurrent scenarios:

- Multiple simultaneous streams.
- Scan during playback.
- Server shutdown during streaming.
- Peer disappears during stream.
- Multiple discovery updates.
- Concurrent progress updates.
- Duplicate transcoding requests.

Concurrency bugs should be tested when shared state is involved.

---

# 36. Race Conditions

Shared application state must have clear ownership.

When using:

```text
Arc
Mutex
RwLock
channels
```

document why synchronization exists.

Avoid holding locks across slow network, disk, or FFmpeg operations when possible.

Tests should attempt concurrent operations for critical shared-state components.

---

# 37. Timeout Policy

Network and external-process operations should have reasonable timeouts where appropriate.

Examples:

- Peer API request.
- Pairing request.
- Discovery resolution.
- FFmpeg startup.
- Graceful shutdown.

No external operation should hang indefinitely without intentional design.

---

# 38. Retry Policy

Retries must be bounded.

Avoid:

```text
retry forever every 10ms
```

Use:

- Maximum attempts.
- Backoff.
- Cancellation.
- Clear failure state.

Retries should only apply to transient errors.

---

# 39. Logging Verification

Logs should assist debugging without leaking sensitive information.

Tests or review should ensure logs do not expose:

- Tokens.
- Pairing secrets.
- Raw passwords.
- Sensitive local paths unnecessarily.
- Private headers.

Use structured fields rather than large unstructured strings.

---

# 40. Static Security Review

Before merging security-sensitive features, review:

- Input trust boundary.
- Authentication requirement.
- Authorization requirement.
- Filesystem effect.
- Network exposure.
- Command execution.
- Persistence.
- Secret handling.
- Error leakage.

This review should be documented in the task or pull-request summary.

---

# 41. Dependency Quality

Before adding a dependency, verify:

- Active maintenance.
- License compatibility.
- Security history.
- Cross-platform support.
- Mobile support if relevant.
- Binary size.
- Necessity.

Run dependency security checks regularly.

Rust ecosystem:

```bash
cargo audit
```

JavaScript ecosystem:

Use the project package manager's audit tooling where appropriate.

Do not automatically upgrade dependencies without reviewing breaking changes.

---

# 42. Lockfiles

Commit dependency lockfiles where appropriate for reproducible application builds.

Examples:

```text
Cargo.lock
package-lock.json
pnpm-lock.yaml
yarn.lock
```

Use only the lockfile matching the chosen JavaScript package manager.

Do not keep multiple package-manager lockfiles.

---

# 43. Compiler and Warning Policy

Production code should aim for zero actionable warnings.

Rust CI should eventually treat Clippy warnings as errors.

TypeScript should use strict mode.

Warnings may be temporarily allowed only when:

- Clearly understood.
- Documented.
- Tracked for removal.

---

# 44. CI Pipeline

A typical CI pipeline should run:

```text
Checkout
   |
Install dependencies
   |
Frontend typecheck
   |
Frontend lint
   |
Frontend tests
   |
Frontend build
   |
Rust format check
   |
Rust Clippy
   |
Rust tests
   |
Rust build/check
   |
Security checks
```

Platform-specific build jobs may run separately.

---

# 45. Suggested CI Stages

## Stage 1 — Fast Checks

Run on every commit/pull request:

```text
format
typecheck
lint
unit tests
cargo check
```

## Stage 2 — Full Tests

Run after fast checks:

```text
integration tests
frontend build
Rust full tests
API contract tests
```

## Stage 3 — Platform Builds

Build:

```text
Windows
Linux
macOS
```

Later:

```text
Android
iOS
```

## Stage 4 — Release Verification

Before release:

```text
full platform builds
manual smoke tests
security audit
migration test
package/install test
version verification
```

---

# 46. CI Failure Policy

A failed required quality gate blocks merging.

Do not bypass CI failures by:

- Removing tests.
- Disabling rules.
- Ignoring warnings.
- Marking failures flaky without investigation.

If a test is genuinely flaky:

1. Identify cause.
2. Create a tracked issue.
3. Isolate or temporarily disable only when necessary.
4. Document why.
5. Fix promptly.

---

# 47. Test Isolation

Tests should not depend on:

- Developer home directories.
- Existing user databases.
- Fixed local IP addresses.
- Internet access.
- Real user media.
- Execution order.
- Previous test state.

Use temporary directories and isolated databases.

---

# 48. Deterministic Tests

Tests should produce consistent results.

Avoid relying on:

- Real current time without injection/control.
- Random values without deterministic seed where relevant.
- Arbitrary sleeps.
- Uncontrolled network availability.

Prefer waiting on explicit events or conditions over fixed delays.

Bad:

```text
sleep 5 seconds and hope server started
```

Better:

```text
wait until health endpoint reports ready
```

with a bounded timeout.

---

# 49. Test Fixtures

Maintain small reusable fixtures.

Suggested:

```text
tests/fixtures/
├── tiny-video.mp4
├── tiny-audio.mp3
├── empty.bin
├── subtitle.srt
└── malformed-metadata.json
```

Generated fixtures are preferable where possible.

All committed fixtures must have clear legal provenance.

---

# 50. Mocking Policy

Mock external boundaries, not core logic.

Good mocks:

- HTTP transport.
- Platform APIs.
- FFmpeg process runner.
- Clock.
- Discovery transport.
- Filesystem adapter for narrow unit tests.

Avoid excessive mocking that makes tests validate implementation details instead of behavior.

---

# 51. Integration Over Excessive Mocking

For database, HTTP routes, and streaming behavior, prefer lightweight real integrations where practical.

For example:

```text
real Axum router
temporary SQLite
temporary media file
HTTP test client
```

This provides stronger confidence than mocking every layer.

---

# 52. Code Review Checklist

Reviewers and AI agents should check:

- Does this solve the requested task?
- Is the change minimal and cohesive?
- Is logic in the correct layer?
- Is business logic duplicated?
- Are inputs validated?
- Are errors handled?
- Are tests meaningful?
- Are failure paths tested?
- Are docs updated?
- Are dependencies justified?
- Are platform impacts understood?
- Are security boundaries respected?
- Could this leak local filesystem information?
- Could this create unbounded resource usage?

---

# 53. AI Self-Review Requirement

Before presenting implementation as finished, AI agents must perform a self-review.

The self-review should examine the diff for:

- Incomplete TODOs.
- Debug prints.
- Dead code.
- Accidental API changes.
- Unsafe assumptions.
- Missing tests.
- Missing docs.
- Hardcoded paths.
- Hardcoded addresses.
- Hardcoded credentials.
- Unhandled errors.
- Platform-specific mistakes.

Agents should correct these before declaring completion.

---

# 54. TODO Policy

TODO comments are allowed only when they are specific.

Bad:

```text
TODO fix later
```

Good:

```text
TODO: add HLS cleanup after TranscodeManager lifecycle is implemented.
```

Do not leave TODOs for code necessary to make the current feature correct.

---

# 55. No Placeholder Success

Agents must not return fake or placeholder values in completed production paths merely to satisfy compilation.

Examples of unacceptable final behavior:

```rust
return Ok(vec![]);
```

when a real library scan is required.

Or:

```ts
return {
  running: true
}
```

when actual server state must be queried.

Temporary placeholders must be explicit and removed before the relevant feature is considered complete.

---

# 56. Documentation Verification

Documentation is part of quality.

After implementation, confirm:

- Directory `README.md` is accurate.
- Root documentation remains accurate.
- API docs match behavior.
- Configuration docs match available settings.
- Limitations are current.

Stale documentation is considered a defect.

---

# 57. Documentation Tests

Where feasible, code examples in documentation should be kept valid.

Commands documented in the root README should periodically be exercised in CI or release verification.

---

# 58. Version Control Hygiene

Changes should be logically grouped.

Avoid combining:

```text
feature implementation
mass formatting
unrelated refactor
dependency upgrade
```

in one change unless necessary.

This improves reviewability and rollback safety.

---

# 59. Refactoring Policy

Refactoring is allowed when it:

- Reduces duplication.
- Clarifies ownership.
- Enables testing.
- Fixes architectural drift.
- Simplifies code.

Refactors should preserve behavior unless a behavior change is explicitly requested.

Add characterization tests before risky refactors.

---

# 60. Regression Tests

Every confirmed bug should ideally produce a regression test.

Workflow:

```text
Reproduce bug
    |
Create failing test
    |
Fix implementation
    |
Confirm test passes
```

This prevents the same defect from returning.

---

# 61. Release Readiness

A release candidate should satisfy:

- CI fully green.
- No critical known security issue.
- No destructive migration issue.
- Core manual smoke test completed.
- Supported platforms tested.
- Version updated consistently.
- Changelog updated.
- Installer/package tested.
- Upgrade from previous release tested where applicable.

---

# 62. Smoke Test

Every release should perform at least:

```text
Install
Launch
Create/open database
Select library
Scan media
Start server
Open web UI
Play media
Seek media
Stop server
Restart app
Confirm persistence
```

When peer functionality exists:

```text
Discover peer
Pair peer
Browse remote library
Play remote media
Disconnect peer
Verify graceful handling
```

---

# 63. Release Platform Testing

For each supported platform verify:

- Installation.
- Launch.
- Permissions.
- Folder selection.
- LAN server.
- Firewall interaction if relevant.
- Playback.
- Shutdown.
- Update/uninstall behavior where applicable.

Do not assume behavior on one OS guarantees behavior on another.

---

# 64. Android Verification

Android-specific checks should include:

- Storage permission flow.
- Selected directory persistence.
- LAN permission behavior.
- Server foreground service.
- App background/foreground transition.
- Battery/system restrictions.
- Network changes.
- App termination.

---

# 65. iOS Verification

iOS-specific checks should include:

- Local-network permission.
- Folder/document access.
- Client discovery.
- Playback.
- Foreground server behavior if implemented.
- Background limitation handling.
- App lifecycle transitions.

Do not claim persistent background server behavior unless it has been explicitly implemented and validated against platform rules.

---

# 66. Security Release Checklist

Before public release:

```text
[ ] Filesystem traversal tests pass.
[ ] Pairing/auth tests pass.
[ ] No default open access to private libraries.
[ ] Tokens are securely generated.
[ ] Secrets are not logged.
[ ] HTTP inputs are bounded.
[ ] FFmpeg arguments are safely constructed.
[ ] Dependency audit reviewed.
[ ] Remote errors do not expose internal paths.
[ ] Peer revocation works.
```

---

# 67. Performance Release Checklist

```text
[ ] Direct Play does not buffer whole files.
[ ] Multiple streams have bounded resources.
[ ] Library UI handles expected collection size.
[ ] Server startup remains acceptable.
[ ] Scanner does not block UI.
[ ] FFmpeg processes are bounded.
[ ] Temporary transcoding files are cleaned.
[ ] Shutdown does not leave orphan processes.
```

---

# 68. Quality Levels by Development Stage

## Prototype

Required:

- Compiles.
- Core flow works.
- Critical logic tested.
- No destructive security flaw.
- Basic docs.

## MVP

Required:

- Static checks.
- Unit tests.
- Integration tests.
- Critical security tests.
- CI.
- Manual smoke test.
- Cross-platform desktop validation.

## Beta

Required:

- Broader E2E coverage.
- Migration tests.
- Performance measurements.
- Mobile tests.
- Dependency audits.
- Release packaging tests.

## Stable Release

Required:

- Defined compatibility guarantees.
- Upgrade testing.
- Regression suite.
- Security review.
- Platform matrix.
- Reproducible release process.

---

# 69. Quality Metrics

Useful project metrics may include:

- Test pass rate.
- CI success rate.
- Build warnings.
- Clippy warnings.
- Frontend lint errors.
- Test runtime.
- Known flaky tests.
- Open critical bugs.
- Crash reports if optional diagnostics are introduced.
- Performance benchmark trends.

Do not optimize development around arbitrary coverage percentages alone.

Coverage is useful, but meaningful behavioral testing is more important.

---

# 70. Code Coverage

Coverage tools may be used to identify untested code.

Coverage should prioritize:

- Security.
- Parsers.
- Streaming.
- Persistence.
- Pairing.
- Network state.
- Core services.

Do not create low-value tests solely to increase a percentage.

---

# 71. Mutation Testing

Mutation testing may be introduced later for security-sensitive pure logic.

Useful targets:

- Range parser.
- Authorization rules.
- Capability logic.
- Path validation.

It is optional during early MVP development.

---

# 72. Observability During Development

Development builds should make failures easy to diagnose.

Useful tools:

- Structured logs.
- Clear error UI.
- Server status endpoint.
- Debug-level network events.
- Transcoding process status.

Debug diagnostics must not become insecure production behavior.

---

# 73. Health Endpoint

A simple local health endpoint may be introduced:

```http
GET /api/v1/health
```

Example:

```json
{
  "status": "ok"
}
```

It can support:

- Tests.
- Startup readiness checks.
- Development diagnostics.

Do not expose sensitive data from this endpoint.

---

# 74. Testable Architecture Requirement

Architecture decisions should favor dependency injection at boundaries.

Examples:

```text
Clock
Filesystem
ProcessRunner
DiscoveryTransport
Database
PeerClient
```

This does not require a heavy dependency-injection framework.

Simple Rust traits or explicit constructor dependencies are enough.

---

# 75. Example Rust Boundary Design

Instead of directly calling a process everywhere:

```rust
pub trait MediaProbe {
    async fn probe(&self, path: &Path) -> Result<MediaMetadata, ProbeError>;
}
```

Production:

```text
FfmpegMediaProbe
```

Tests:

```text
FakeMediaProbe
```

Use this pattern where it materially improves testing.

Do not abstract every trivial function.

---

# 76. Example Frontend Boundary Design

```ts
export interface LocalStreamBackend {
  getLibrary(): Promise<MediaItem[]>
}
```

Production implementations:

```text
TauriBackend
HttpBackend
```

Tests:

```text
FakeBackend
```

Composables should depend on this interface where practical.

---

# 77. Feature Development Checklist

For every feature:

```text
[ ] Requirements understood.
[ ] Relevant architecture docs read.
[ ] Inputs and outputs defined.
[ ] Failure cases identified.
[ ] Security impact considered.
[ ] Implementation completed.
[ ] Formatting passes.
[ ] Linting passes.
[ ] Unit tests added/updated.
[ ] Integration tests added if relevant.
[ ] Manual behavior verified if needed.
[ ] Documentation updated.
[ ] Self-review completed.
```

---

# 78. Bug Fix Checklist

For every bug:

```text
[ ] Reproduction understood.
[ ] Root cause identified.
[ ] Regression test added where feasible.
[ ] Minimal fix implemented.
[ ] Related edge cases checked.
[ ] Relevant test suite passes.
[ ] No unrelated behavior changed.
[ ] Documentation updated if behavior changed.
```

---

# 79. Security-Sensitive Change Checklist

```text
[ ] Trust boundary identified.
[ ] Inputs validated.
[ ] Authorization verified.
[ ] Error leakage reviewed.
[ ] Logging reviewed.
[ ] Resource exhaustion reviewed.
[ ] Security tests added.
[ ] No unsafe shell construction.
[ ] No filesystem escape.
[ ] No secret persisted insecurely.
```

---

# 80. Database Change Checklist

```text
[ ] Migration created.
[ ] Fresh DB tested.
[ ] Existing DB migration tested.
[ ] Data preservation verified.
[ ] Rollback/failure considered.
[ ] Queries updated.
[ ] Models updated.
[ ] Docs updated.
```

---

# 81. API Change Checklist

```text
[ ] Route documented.
[ ] Request type documented.
[ ] Response type documented.
[ ] Error responses documented.
[ ] Authentication requirement documented.
[ ] Contract tests updated.
[ ] Compatibility impact reviewed.
[ ] Frontend adapters updated.
```

---

# 82. Streaming Change Checklist

```text
[ ] No full-file buffering.
[ ] Range behavior tested.
[ ] Correct MIME handling.
[ ] Client cancellation handled.
[ ] File disappearance handled.
[ ] Authorization checked.
[ ] Large-file behavior considered.
[ ] Concurrent streams considered.
```

---

# 83. Agent Final Report

After completing a code task, an AI agent should provide a concise report containing:

```text
Implemented:
- ...

Tests:
- ...

Verification:
- ...

Documentation:
- ...

Known limitations:
- ...
```

Do not claim a test was run unless it was actually run.

If a check could not be executed, state that explicitly.

---

# 84. No False Verification

Agents must never say:

```text
All tests pass.
```

unless tests were actually executed successfully.

If testing is unavailable:

```text
Tests were not executed because ...
```

If only targeted tests were executed:

```text
Executed:
- cargo test range_parser
- npm run test -- useMediaLibrary
```

Be precise.

---

# 85. Failure Handling During Implementation

If a quality check fails:

1. Inspect the actual failure.
2. Determine whether the code or test is wrong.
3. Fix root cause.
4. Re-run the failed check.
5. Re-run related checks.
6. Do not hide the failure.

If an unrelated pre-existing failure exists, document it clearly and avoid claiming the repository is fully green.

---

# 86. Scope of Verification

Verification effort should match risk.

Low-risk UI copy change:

```text
typecheck
lint
targeted UI test
```

Streaming parser change:

```text
format
clippy
unit tests
integration tests
boundary tests
security review
```

Pairing/auth change:

```text
all relevant tests
negative authorization tests
security review
integration test
```

Cross-cutting core change:

```text
full test suite
build
integration tests
manual smoke test where necessary
```

---

# 87. Continuous Improvement

The quality process itself should evolve.

When a bug reaches users, ask:

```text
Could an automated check have caught this?
```

If yes, improve:

- Tests.
- Static analysis.
- CI.
- Documentation.
- Review checklist.

The goal is not merely to fix individual bugs but to strengthen the system that allowed them.

---

# 88. Recommended Baseline Commands

The repository should eventually expose convenient scripts.

Example:

```bash
npm run format
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
```

Rust:

```bash
cargo fmt
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

A combined development verification command is recommended.

Example:

```bash
npm run verify
```

or:

```bash
./scripts/verify.sh
```

which may run all required fast checks.

---

# 89. Suggested Verification Script

A project-level verification script may conceptually perform:

```text
Frontend formatting
      |
Frontend lint
      |
Frontend typecheck
      |
Frontend tests
      |
Rust formatting
      |
Rust Clippy
      |
Rust tests
      |
Build
```

It should exit immediately or clearly report failures.

The script itself should be documented.

---

# 90. Final Quality Principle

LocalStream handles:

- User files.
- Local networks.
- Large media.
- Native OS capabilities.
- External processes.
- Trusted and untrusted peers.

Therefore correctness and safety cannot depend on users behaving perfectly.

Every subsystem should assume:

```text
files can disappear
networks can fail
peers can be malicious
media can be malformed
processes can crash
permissions can change
devices can sleep
users can cancel operations
```

The code should remain predictable under these conditions.

The overarching rule for all contributors and AI agents is:

> **Implement, verify, test failure cases, document, and only then consider the work complete.**
