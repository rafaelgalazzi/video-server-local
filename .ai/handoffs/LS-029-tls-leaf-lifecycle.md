# LS-029 Handoff — TLS leaf lifecycle

Completed 2026-08-18. The lifecycle reuses a valid leaf, renews seven days before expiry, rotates on SAN/address change, keeps the stable root identity, and exposes generation/current material for orchestration. Existing wrong-root/name and TLS-version tests remain green.

Next dependency: LS-030.
