# Run LocalStream

Open PowerShell in this repository.

## First Time Only

```powershell
npm install
```

You also need Node.js 22.13+, Rust, and the [Tauri prerequisites for Windows](https://v2.tauri.app/start/prerequisites/).

## Run the Desktop App

```powershell
npm run tauri dev
```

Keep the terminal open while using LocalStream. Stop it with `Ctrl+C`.

In the app, click **Choose folder** and select a folder containing MP4, MKV, WebM, MOV, or M4V videos.

## Web Preview Only

```powershell
npm run dev
```

Open `http://localhost:1420`. Native features such as choosing a folder do not work in the browser preview.

## Check the Project

```powershell
npm run verify
cargo test --workspace
```

## If It Does Not Start

Run:

```powershell
npm install
cargo check --workspace
npm run tauri dev
```

Read the first error printed in the terminal. Do not delete `Cargo.lock`, `package-lock.json`, or uncommitted files while troubleshooting.
