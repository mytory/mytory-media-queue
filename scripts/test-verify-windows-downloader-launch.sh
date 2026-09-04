#!/usr/bin/env bash
# Verifies that Windows Downloader processes do not create a visible console window.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="$ROOT/src-tauri/src/downloader.rs"

grep -Fq '#[cfg(windows)]' "$SOURCE"
grep -Fq 'const CREATE_NO_WINDOW: u32 = 0x08000000;' "$SOURCE"
grep -Fq 'use std::os::windows::process::CommandExt;' "$SOURCE"
grep -Fq 'command.creation_flags(CREATE_NO_WINDOW);' "$SOURCE"
