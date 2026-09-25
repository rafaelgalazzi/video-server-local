# Build Tools

Repository-local Rust build tools live here. They are workspace members so they can share the application's typed packaging boundary without adding another executable target to the Tauri application package.

- `portable-packager/`: builds the single-file Windows portable executable after the native Tauri binary and verified media tools are ready.
