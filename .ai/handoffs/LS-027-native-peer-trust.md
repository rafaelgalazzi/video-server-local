# LS-027 Handoff — Native peer trust

Completed 2026-08-18. `native_client` isolates node ID, verified root fingerprint, endpoint hints, and bearer credential behind a protected-store trait/keyring adapter. Corruption and pin changes fail closed; replacement and deletion are explicit. A real credential write was not performed to avoid modifying user state.

Next dependency: LS-031 audit.
