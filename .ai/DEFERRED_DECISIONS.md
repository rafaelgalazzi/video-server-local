# Deferred Decisions

Do not repeatedly reopen these decisions until the documented trigger is reached. IDs are permanent.

## DD-001 — HLS strategy

Status: Deferred

Reason for deferral: The MVP must prove Direct Play and HTTP Range before adaptive streaming is designed.

Questions:

- Which clients require HLS?
- Should segments be generated on demand and cached?

Do not decide before: Direct Play compatibility gaps are measured in a working MVP.

## DD-002 — External metadata provider

Status: Deferred

Reason for deferral: Basic local metadata is sufficient for the first end-to-end flow and cloud metadata would complicate local-first privacy.

Questions:

- Is an opt-in Internet provider necessary?
- How are identities and cache lifetimes managed?

Do not decide before: Basic scanning and metadata storage work end to end.

## DD-003 — Mobile background architecture

Status: Deferred

Reason for deferral: Desktop is the first implementation target; Android and iOS impose different lifecycle constraints.

Questions:

- Which server behavior is viable under each mobile OS lifecycle?
- Does Android require a foreground service?

Do not decide before: The desktop MVP and mobile client requirements are validated.

## DD-004 — TV application framework

Status: Deferred

Reason for deferral: A browser client is the initial TV-access path.

Questions:

- Which TV platforms need native applications?
- Can the web interface meet remote-control accessibility needs?

Do not decide before: Browser-client limitations are observed on target TVs.

## DD-005 — Network filesystem support

Status: Deferred

Reason for deferral: SMB, NFS, and WebDAV are outside the initial local-folder MVP.

Questions:

- Which protocols and credential-storage mechanisms are required?
- How should unavailable shares affect scans?

Do not decide before: Local filesystem libraries are reliable and platform requirements are gathered.

## DD-006 — Remote Internet access

Status: Deferred

Reason for deferral: LocalStream is LAN-first and must not require Internet access for primary functionality.

Questions:

- Is remote access a product requirement?
- What identity, transport security, and relay model would be acceptable?

Do not decide before: LAN functionality, pairing, and threat modeling are mature.
