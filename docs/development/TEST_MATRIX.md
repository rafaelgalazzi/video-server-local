# Test Matrix

Legend: ✅ Verified · 🟡 Partial · ❌ Failing · — Not implemented · ? Unknown / not verified

LS-001 has verified frontend and Rust unit checks plus a Windows Tauri development smoke test. Product media features remain unimplemented. Other platforms are unknown / not verified.

| Feature             | Unit | Integration | Windows | Linux | macOS | Android | iOS |
| ------------------- | ---- | ----------- | ------- | ----- | ----- | ------- | --- |
| Frontend foundation | ✅   | 🟡          | ✅      | ?     | ?     | ?       | ?   |
| Rust core boundary  | ✅   | 🟡          | ✅      | ?     | ?     | ?       | ?   |
| Library scanner     | ✅   | 🟡          | 🟡      | ?     | ?     | ?       | ?   |
| SQLite              | —    | —           | ?       | ?     | ?     | ?       | ?   |
| HTTP server         | —    | —           | ?       | ?     | ?     | ?       | ?   |
| HTTP Range          | —    | —           | ?       | ?     | ?     | ?       | ?   |
| Direct Play         | —    | —           | ?       | ?     | ?     | ?       | ?   |
| mDNS                | —    | —           | ?       | ?     | ?     | ?       | ?   |
| Pairing             | —    | —           | ?       | ?     | ?     | ?       | ?   |
| Remote library      | —    | —           | ?       | ?     | ?     | ?       | ?   |
| FFmpeg              | —    | —           | ?       | ?     | ?     | ?       | ?   |

Update a cell only from concrete evidence. Record exact commands and environments in the relevant task or handoff; never infer platform support from compilation on another platform.
