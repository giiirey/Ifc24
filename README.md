# IFC Tauri — IFC Model Visualizer

A desktop application for opening and exploring IFC (Industry Foundation Classes) building model files. The app is built with [Tauri 2](https://tauri.app), Rust, and React, and uses [IfcOpenShell](https://ifcopenshell.org/) on the back end to parse IFC models and extract geometry.

## Features

- Open `.ifc` files through a native file dialog
- 3D visualization of the model in a WebGL viewer (Three.js / react-three-fiber)
- Entity tree with the IFC object hierarchy
- Search entities by name and filter by IFC type (walls, slabs, columns, beams, doors, windows, spaces, …)
- Per-entity properties panel (properties, placement, bounding box, materials)
- Fast entity lookups backed by an index stored next to the model file

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Front end (React 19 + TypeScript + Vite)           │
│  • 3D viewer (three / @react-three/fiber)          │
│  • entity tree, search & filters, properties panel │
│  • state management (zustand)                      │
├─────────────────────────────────────────────────────┤
│  Tauri IPC (commands)                              │
├─────────────────────────────────────────────────────┤
│  Back end (Rust)                                   │
│  • IfcOpenShell (C++) via bindgen FFI               │
│  • IFC parsing, hierarchy building, geometry       │
│    extraction, placement resolution, bounding boxes│
│  • entity index (.idx files)                       │
│  • geometry cache                                  │
└─────────────────────────────────────────────────────┘
```

## Repository layout

```
├── app/                Front end (React + Vite)
├── src/                Rust back end
│   ├── commands/       Tauri IPC commands
│   ├── services/       IFC parsing, geometry, hierarchy, caching
│   ├── models/         Shared data types (IPC, entities, geometry)
│   ├── bindings/       IfcOpenShell C++ wrapper
│   └── state.rs        Global app state
├── samples/            Sample IFC models to try out
├── build.rs            Compiles the C++ IfcOpenShell wrapper and generates bindings
└── start.sh            Dev launcher with Wayland/KDE workarounds
```

## Prerequisites

- Rust 1.76+
- Node.js (npm workspaces are used)
- IfcOpenShell C++ libraries
  - On Ubuntu/Debian: `sudo apt install libifcopenshell-dev`
  - Or build IfcOpenShell from source
  - Point the build at the installation prefix with:
    ```bash
    export IFCOPENSHELL_ROOT=/usr   # or your installation prefix
    ```
  - If Boost is not found in `/usr/include/boost`, the build falls back to a stub mode (`IFC_OPENSHELL_STUB`) with limited functionality.
- The `tauri` CLI (`cargo install tauri-cli` or `cargo tauri` via the `cargo-tauri` binary)

## Setup

```bash
npm install          # install front-end dependencies (npm workspaces)
```

## Development

```bash
# Recommended on Linux (KDE/Wayland): sets environment workarounds
./start.sh

# Or directly:
cargo tauri dev
```

`cargo tauri dev` starts the Vite dev server at `http://localhost:5173` automatically (see `beforeDevCommand` in `tauri.conf.json`).

To run only the front-end dev server:

```bash
npm run dev          # runs `vite` inside app/
```

## Building

```bash
cargo tauri build            # debug build -> target/debug/ifctauri
cargo tauri build --release  # release build
```

## Lint & formatting

```bash
cargo clippy   # Rust lint checks
cargo fmt      # format the Rust code
```

## Sample models

The `samples/` directory contains a few IFC files (Ifc2x3, Ifc4, and real-world models) that are convenient for testing: `Ifc2x3_SampleCastle.ifc`, `Ifc4_SampleHouse.ifc`, and others.

## Troubleshooting

- **Blank viewer on KDE/Wayland** — run via `./start.sh`; it sets `GDK_BACKEND=x11`, `GSK_RENDERER=ng`, and `WEBKIT_DISABLE_DMABUF_RENDERER=1` to fix the `BadMatch` protocol error.
- **`cargo` not found** — make sure the rustup toolchain is on your `PATH` (e.g. `~/.cargo/bin`).
