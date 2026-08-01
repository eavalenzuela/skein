#!/usr/bin/env bash
# Locate the GTK/WebKit development headers Tauri needs.
#
# Distro -dev packages are the normal source. When they are absent but a
# GNOME flatpak SDK is installed, its pkgconfig tree provides the same
# files, which is enough to compile and test the Rust crate on a machine
# where installing system packages isn't an option.
if pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
  return 0 2>/dev/null || exit 0
fi
sdk=$(ls -d "$HOME"/.local/share/flatpak/runtime/org.gnome.Sdk/*/*/*/files 2>/dev/null | head -1)
if [ -n "$sdk" ] && [ -d "$sdk/lib/x86_64-linux-gnu/pkgconfig" ]; then
  # PKG_CONFIG_PATH for the headers, -L via RUSTFLAGS so the linker finds
  # the matching .so files. Deliberately NOT LD_LIBRARY_PATH — that would
  # shadow the system glibc and break every tool in the shell.
  export PKG_CONFIG_PATH="$sdk/lib/x86_64-linux-gnu/pkgconfig:$sdk/lib/pkgconfig:$sdk/share/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  export RUSTFLAGS="-L $sdk/lib/x86_64-linux-gnu${RUSTFLAGS:+ $RUSTFLAGS}"
fi
