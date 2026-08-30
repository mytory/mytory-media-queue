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
if [ "$TARGET" = "universal-apple-darwin" ]; then
  curl -fL --retry 3 -o /tmp/yt-dlp-macos "$YTDLP_URL"
else
  curl -fL --retry 3 -o "$OUT/yt-dlp-$TARGET$EXT" "$YTDLP_URL"
fi

if [ "$TARGET" = "universal-apple-darwin" ]; then
  echo "preparing per-arch and universal sidecars for macOS"
  # yt-dlp_macos is already a universal binary; publish it under every
  # name tauri looks up (per-arch cargo builds and the universal bundle).
  for name in aarch64-apple-darwin x86_64-apple-darwin universal-apple-darwin; do
    cp /tmp/yt-dlp-macos "$OUT/yt-dlp-$name"
  done
  for tool in ffmpeg ffprobe; do
    curl -fL --retry 3 -o /tmp/$tool-x64 "https://github.com/$FFMPEG_RELEASE/releases/latest/download/$tool-darwin-x64"
    curl -fL --retry 3 -o /tmp/$tool-arm64 "https://github.com/$FFMPEG_RELEASE/releases/latest/download/$tool-darwin-arm64"
    cp /tmp/$tool-arm64 "$OUT/$tool-aarch64-apple-darwin"
    cp /tmp/$tool-x64 "$OUT/$tool-x86_64-apple-darwin"
    lipo -create /tmp/$tool-x64 /tmp/$tool-arm64 -output "$OUT/$tool-universal-apple-darwin"
  done
  chmod +x "$OUT"/yt-dlp-* "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
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
