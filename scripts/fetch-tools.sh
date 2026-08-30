#!/usr/bin/env bash
# Downloads the bundled media toolchain (yt-dlp, ffmpeg, ffprobe) into
# src-tauri/binaries/ with Tauri sidecar naming for the given target triple.
#
# Usage: scripts/fetch-tools.sh <target-triple>
#   universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin \
#   | x86_64-pc-windows-msvc | x86_64-unknown-linux-gnu
set -euo pipefail

TARGET="${1:?usage: scripts/fetch-tools.sh <target-triple>}"
OUT="src-tauri/binaries"
FFMPEG_RELEASE="eugeneware/ffmpeg-static"
mkdir -p "$OUT"

case "$TARGET" in
  universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin)
    YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    ;;
  x86_64-pc-windows-msvc)
    YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    ;;
  x86_64-unknown-linux-gnu)
    YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    ;;
  *)
    echo "unsupported target: $TARGET" >&2
    exit 1
    ;;
esac

EXT=""
case "$TARGET" in
  x86_64-pc-windows-msvc) EXT=".exe" ;;
esac

echo "fetching yt-dlp for $TARGET"
curl -fL --retry 3 -o "$OUT/yt-dlp-$TARGET$EXT" "$YTDLP_URL"

if [ "$TARGET" = "universal-apple-darwin" ]; then
  echo "merging universal ffmpeg/ffprobe with lipo"
  curl -fL --retry 3 -o /tmp/ffmpeg-x64 "https://github.com/$FFMPEG_RELEASE/releases/latest/download/ffmpeg-darwin-x64"
  curl -fL --retry 3 -o /tmp/ffmpeg-arm64 "https://github.com/$FFMPEG_RELEASE/releases/latest/download/ffmpeg-darwin-arm64"
  curl -fL --retry 3 -o /tmp/ffprobe-x64 "https://github.com/$FFMPEG_RELEASE/releases/latest/download/ffprobe-darwin-x64"
  curl -fL --retry 3 -o /tmp/ffprobe-arm64 "https://github.com/$FFMPEG_RELEASE/releases/latest/download/ffprobe-darwin-arm64"
  lipo -create /tmp/ffmpeg-x64 /tmp/ffmpeg-arm64 -output "$OUT/ffmpeg-$TARGET"
  lipo -create /tmp/ffprobe-x64 /tmp/ffprobe-arm64 -output "$OUT/ffprobe-$TARGET"
  chmod +x "$OUT/ffmpeg-$TARGET" "$OUT/ffprobe-$TARGET"
else
  echo "fetching ffmpeg/ffprobe for $TARGET"
  case "$TARGET" in
    aarch64-apple-darwin) FF="darwin-arm64" ;;
    x86_64-apple-darwin) FF="darwin-x64" ;;
    x86_64-pc-windows-msvc) FF="win32-x64" ;;
    x86_64-unknown-linux-gnu) FF="linux-x64" ;;
  esac
  curl -fL --retry 3 -o "$OUT/ffmpeg-$TARGET$EXT" \
    "https://github.com/$FFMPEG_RELEASE/releases/latest/download/ffmpeg-$FF"
  curl -fL --retry 3 -o "$OUT/ffprobe-$TARGET$EXT" \
    "https://github.com/$FFMPEG_RELEASE/releases/latest/download/ffprobe-$FF"
  if [ -z "$EXT" ]; then
    chmod +x "$OUT/yt-dlp-$TARGET" "$OUT/ffmpeg-$TARGET" "$OUT/ffprobe-$TARGET"
  fi
fi

ls -la "$OUT"
