# AGENTS.md

## Setup

- Run `npm install` in the repository root to install front‑end dependencies via npm workspaces.
- Ensure Rust 1.76+ is installed (required by `tauri`).

## Build

- `cargo tauri build` – builds the Tauri binary and places it in `target/debug/ifctauri`.
- For a release build: `cargo tauri build --release`.

## Prerequisites to run

- Install Rust 1.76+ (required by `tauri`).
- Install IfcOpenShell C++ library: e.g., Ubuntu/Debian `sudo apt install libifcopenshell-dev` or build from source.
- Set environment variable `IFCOPENSHELL_ROOT` to the installation prefix (default `/usr`).

## Development

- `cargo tauri dev` – starts the Tauri host and launches the Vite dev server at `http://localhost:5173`.

- `npm run dev` at the repository root runs `npm --prefix app run dev`, which starts Vite in the `app/` directory.
- The front‑end dev server is started automatically via the `beforeDevCommand` defined in `tauri.conf.json`.

## Running the binary

- `cargo run` or `cargo tauri run` will launch the compiled binary directly.

## Lint & Formatting

- `cargo clippy` – checks for common Rust lint issues.
- `cargo fmt` – formats the Rust code.

## Front‑end scripts

- In the `app/` directory, `npm run dev` runs Vite.
- `npm run build` compiles the front‑end (used by `cargo tauri build`).

## Tests

- No explicit tests are defined; `cargo test` will run any discovered Rust tests.

## Misc

- Tauri binary is a Rust static library (`staticlib`) and regular library (`rlib`).
- The binary uses the `tauri` crate with the `wry` feature.
- For macOS builds, the `objc2` dependency is required.

