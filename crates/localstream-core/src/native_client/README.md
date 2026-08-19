# Native Client Trust

## Purpose

Persist a native client's explicitly verified LocalStream node pin and bearer credential through an injected protected-secret boundary.

## Public Interfaces

- `NativePeerTrustService`: save, load, verify, explicitly replace, and delete one node trust record.
- `NativePeerTrustStore`: isolated protected-storage boundary.
- `KeyringNativePeerTrustStore`: operating-system credential-store adapter.

Discovery and first contact provide endpoint hints only. Callers must compare the complete root fingerprint through an authenticated pairing ceremony before saving or replacing a record.

## Limitations

Connection and discovery clients are planned for Phase B. This module does not perform network I/O or trust-on-first-use.
