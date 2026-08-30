#!/usr/bin/env bash
# Downloads the bundled media toolchain (yt-dlp, ffmpeg, ffprobe) into
# src-tauri/binaries/ with Tauri sidecar naming for the given target triple.
#
# Usage: scripts/fetch-tools.sh <target-triple>
#   aarch64-apple-darwin | x86_64-apple-darwin | x86_64-pc-windows-msvc | x86_64-unknown-linux-gnu
set -euo pipefail

TARGET="${1:?usage: scripts/fetch-tools.sh <target-triple>}"
OUT="src-tauri/binaries"
FFMPEG_RELEASE="eugeneware/ffmpeg-static"
mkdir -p "$OUT"

case "$TARGET" in
  aarch64-apple-darwin)
    YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    FFMPEG_ASSET="ffmpeg-darwin-arm64"
    FFPROBE_ASSET="ffprobe-darwin-arm64"
    EXT=""
    ;;
  x86_64-apple-darwin)
    YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    FFMPEG_ASSET="ffmpeg-darwin-x64"
    FFPROBE_ASSET="ffprobe-darwin-x64"
    EXT=""
    ;;
  x86_64-pc-windows-msvc)
    YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    FFMPEG_ASSET="ffmpeg-win32-x64"
    FFPROBE_ASSET="ffprobe-win32-x64"
    EXT=".exe"
    ;;
  x86_64-unknown-linux-gnu)
    YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    FFMPEG_ASSET="ffmpeg-linux-x64"
    FFPROBE_ASSET="ffprobe-linux-x64"
    EXT=""
    ;;
  *)
    echo "unsupported target: $TARGET" >&2
    exit 1
    ;;
esac

echo "fetching yt-dlp for $TARGET"
curl -fL --retry 3 -o "$OUT/yt-dlp-$TARGET$EXT" "$YTDLP_URL"

echo "fetching ffmpeg/ffprobe for $TARGET"
curl -fL --retry 3 -o "$OUT/ffmpeg-$TARGET$EXT" \
  "https://github.com/$FFMPEG_RELEASE/releases/latest/download/$FFMPEG_ASSET"
curl -fL --retry 3 -o "$OUT/ffprobe-$TARGET$EXT" \
  "https://github.com/$FFMPEG_RELEASE/releases/latest/download/$FFPROBE_ASSET"

if [ -z "$EXT" ]; then
  chmod +x "$OUT/yt-dlp-$TARGET" "$OUT/ffmpeg-$TARGET" "$OUT/ffprobe-$TARGET"
fi

ls -la "$OUT"
