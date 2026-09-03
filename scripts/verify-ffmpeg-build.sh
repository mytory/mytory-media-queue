#!/usr/bin/env bash
# Validates the bundled FFmpeg/FFprobe artifact contract for one Tauri target.
set -euo pipefail

TARGET="${1:?usage: scripts/verify-ffmpeg-build.sh <target-triple> <output-directory>}"
OUT="${2:?usage: scripts/verify-ffmpeg-build.sh <target-triple> <output-directory>}"
FFMPEG_VERSION="7.1.1"
EXTENSION=""
[[ "$TARGET" == *windows* ]] && EXTENSION=".exe"

verify_program() {
  local program="$1"
  local binary="$OUT/$program-$TARGET$EXTENSION"
  local version license

  [[ -x "$binary" ]] || {
    echo "missing executable $binary" >&2
    return 1
  }

  version="$($binary -version 2>&1)"
  grep -Fq "$program version $FFMPEG_VERSION" <<<"$version" || {
    echo "$binary is not $program $FFMPEG_VERSION" >&2
    return 1
  }
  grep -Fq -- '--disable-gpl' <<<"$version" || {
    echo "$binary was not configured with --disable-gpl" >&2
    return 1
  }
  if grep -Fq -- '--enable-gpl' <<<"$version"; then
    echo "$binary was configured with --enable-gpl" >&2
    return 1
  fi

  license="$($binary -L 2>&1)"
  license="${license//$'\n'/ }"
  grep -Eq 'GNU Lesser General Public License|LGPL version 2\.1 or later' <<<"$license" || {
    echo "$binary does not report an LGPL license" >&2
    return 1
  }
}

verify_program ffmpeg
verify_program ffprobe
