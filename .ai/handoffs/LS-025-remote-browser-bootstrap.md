# LS-025 Handoff — Remote browser bootstrap

Completed 2026-08-18. Vue detects Tauri synchronously; native mode retains commands/admin panels, while browser mode uses credential-inaccessible same-origin fetch and exposes bootstrap, pairing-required, authenticated, disconnected, and retry states. Browser pairing keeps the short-lived claim secret only in memory and receives the long-term session only as HttpOnly cookie state. Tests cover both modes, pairing, and transitions.

Next dependency: LS-026.
