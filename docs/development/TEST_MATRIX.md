# Test Matrix

Legend: ✅ Verified · 🟡 Partial · ❌ Failing · — Not implemented · ? Unknown / not verified

The frontend/Rust foundation and a large Phase A media/security vertical slice have automated coverage plus Windows smoke evidence. Other platforms remain unknown / not verified.

| Feature             | Unit | Integration | Windows | Linux | macOS | Android | iOS |
| ------------------- | ---- | ----------- | ------- | ----- | ----- | ------- | --- |
| Frontend foundation | ✅   | 🟡          | ✅      | ?     | ?     | ?       | ?   |
| Rust core boundary  | ✅   | 🟡          | ✅      | ?     | ?     | ?       | ?   |
| Library scanner     | ✅   | 🟡          | 🟡      | ?     | ?     | ?       | ?   |
| SQLite              | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| HTTP server         | ✅   | ✅          | ✅      | ?     | ?     | ?       | ?   |
| HTTP Range          | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| Direct Play         | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| Peer credentials    | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| API authorization   | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| Node-root identity  | ✅   | 🟡          | 🟡      | ?     | ?     | ?       | ?   |
| mDNS                | —    | —           | ?       | ?     | ?     | ?       | ?   |
| Pairing             | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| Remote library      | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| FFmpeg              | ✅   | ✅          | 🟡      | ?     | ?     | ?       | ?   |
| Portable EXE        | ✅   | ✅          | 🟡      | —     | —     | —       | —   |

The portable Windows x64 check copied only `LocalStream.exe` to an empty directory, launched it with FFmpeg absent from `PATH`, verified extracted FFmpeg/ffprobe 9.0.2, and fetched the private HTTPS health route and browser UI. A second physical computer, signing, cleanup/update UX, WebView2-missing behavior, and full firewall/trust/pairing/playback qualification remain unverified.

Update a cell only from concrete evidence. Record exact commands and environments in the relevant task or handoff; never infer platform support from compilation on another platform.
