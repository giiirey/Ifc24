#!/bin/bash
# Start IFC Tauri with Wayland-specific environment variables
# These fix the GDK BadMatch protocol error on KDE/Wayland
export GDK_BACKEND=x11
export GSK_RENDERER=ng
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# Ensure cargo subcommands (cargo-tauri) are on PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Ensure cargo itself is on PATH (rustup toolchains are not on the default PATH)
if ! command -v cargo >/dev/null 2>&1; then
  for d in "$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin" \
           "$HOME"/.rustup/toolchains/*/bin; do
    if [ -x "$d/cargo" ]; then
      export PATH="$d:$PATH"
      break
    fi
  done
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "start.sh: cargo not found (install Rust: https://rustup.rs)" >&2
  exit 1
fi

exec cargo tauri dev "$@"
