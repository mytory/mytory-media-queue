#!/usr/bin/env bash
# Extracts the FFmpeg LGPL text from the verified source archive retained for release.
set -euo pipefail

SOURCE_ARCHIVE="${1:?usage: scripts/prepare-ffmpeg-source-material.sh <source-archive> <license-output>}"
LICENSE_OUTPUT="${2:?usage: scripts/prepare-ffmpeg-source-material.sh <source-archive> <license-output>}"
FFMPEG_VERSION="7.1.1"
FFMPEG_SOURCE_SHA256="733984395e0dbbe5c046abda2dc49a5544e7e0e1e2366bba849222ae9e3a03b1"

printf '%s  %s\n' "$FFMPEG_SOURCE_SHA256" "$SOURCE_ARCHIVE" | shasum -a 256 -c -
mkdir -p "$(dirname "$LICENSE_OUTPUT")"
tar -xJOf "$SOURCE_ARCHIVE" "ffmpeg-$FFMPEG_VERSION/COPYING.LGPLv2.1" > "$LICENSE_OUTPUT"
grep -Fq 'GNU LESSER GENERAL PUBLIC LICENSE' "$LICENSE_OUTPUT"
