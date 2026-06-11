#!/bin/bash
# Start IFC Tauri with Wayland-specific environment variables
# These fix the GDK BadMatch protocol error on KDE/Wayland
export GDK_BACKEND=x11
export GSK_RENDERER=ng
export WEBKIT_DISABLE_DMABUF_RENDERER=1

exec cargo tauri dev "$@"
