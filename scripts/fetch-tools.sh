#!/usr/bin/env bash
# Downloads checksum-verified Downloader and Bundled Media Toolchain files into
# src-tauri/binaries/ using the Tauri sidecar naming required by TARGET.
#
# Usage: scripts/fetch-tools.sh <target-triple>
#   universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin \
#   | x86_64-pc-windows-msvc | x86_64-unknown-linux-gnu
set -euo pipefail

TARGET="${1:?usage: scripts/fetch-tools.sh <target-triple>}"
OUT="src-tauri/binaries"
LICENSE_OUT="$OUT/licenses"
YTDLP_VERSION="2026.08.19"
FFMPEG_VERSION="b6.1.1"
YTDLP_BASE_URL="https://github.com/yt-dlp/yt-dlp/releases/download/$YTDLP_VERSION"
YTDLP_LICENSE_URL="https://raw.githubusercontent.com/yt-dlp/yt-dlp/$YTDLP_VERSION/LICENSE"
FFMPEG_BASE_URL="https://github.com/eugeneware/ffmpeg-static/releases/download/$FFMPEG_VERSION"

mkdir -p "$OUT" "$LICENSE_OUT"

fetch() {
  local url="$1"
  local destination="$2"
  local checksum="$3"

  curl -fL --retry 3 -o "$destination" "$url"
  printf '%s  %s\n' "$checksum" "$destination" | shasum -a 256 -c -
}

fetch_ffmpeg_license() {
  local source_name="$1"
  local target_name="$2"
  local checksum="$3"

  fetch "$FFMPEG_BASE_URL/$source_name.LICENSE" \
    "$LICENSE_OUT/ffmpeg-$target_name-LICENSE" "$checksum"
}

fetch "$YTDLP_LICENSE_URL" "$LICENSE_OUT/yt-dlp-LICENSE" \
  7e12e5df4bae12cb21581ba157ced20e1986a0508dd10d0e8a4ab9a4cf94e85c

case "$TARGET" in
  universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin)
    fetch "$YTDLP_BASE_URL/yt-dlp_macos" /tmp/yt-dlp-macos \
      0f192b7ec147ab6288885d6351d9ab67367640029b4377576ef46dd79cf7b202

    for name in aarch64-apple-darwin x86_64-apple-darwin universal-apple-darwin; do
      cp /tmp/yt-dlp-macos "$OUT/yt-dlp-$name"
    done

    fetch "$FFMPEG_BASE_URL/ffmpeg-darwin-x64" /tmp/ffmpeg-x64 \
      ebdddc936f61e14049a2d4b549a412b8a40deeff6540e58a9f2a2da9e6b18894
    fetch "$FFMPEG_BASE_URL/ffprobe-darwin-x64" /tmp/ffprobe-x64 \
      fa3add0ce901f7241abe0dfc0155d958fc834aca3f8ce61f87cc712ae669c1e0
    fetch "$FFMPEG_BASE_URL/ffmpeg-darwin-arm64" /tmp/ffmpeg-arm64 \
      a90e3db6a3fd35f6074b013f948b1aa45b31c6375489d39e572bea3f18336584
    fetch "$FFMPEG_BASE_URL/ffprobe-darwin-arm64" /tmp/ffprobe-arm64 \
      bb2db6f5d8cef919da12fbf592119a987202a8c060a886f3cab091f9cab90b64

    lipo -create /tmp/ffmpeg-x64 /tmp/ffmpeg-arm64 -output "$OUT/ffmpeg-universal-apple-darwin"
    lipo -create /tmp/ffprobe-x64 /tmp/ffprobe-arm64 -output "$OUT/ffprobe-universal-apple-darwin"
    cp /tmp/ffmpeg-arm64 "$OUT/ffmpeg-aarch64-apple-darwin"
    cp /tmp/ffmpeg-x64 "$OUT/ffmpeg-x86_64-apple-darwin"
    cp /tmp/ffprobe-arm64 "$OUT/ffprobe-aarch64-apple-darwin"
    cp /tmp/ffprobe-x64 "$OUT/ffprobe-x86_64-apple-darwin"

    fetch_ffmpeg_license darwin-x64 universal-apple-darwin \
      2e1d16c72fd74e12063776371da757322f8b77589386532f4fd8634bde7de1af
    fetch_ffmpeg_license darwin-arm64 universal-apple-darwin-arm64 \
      cb48bf09a11f5fb576cddb0431c8f5ed0a60157a9ec942adffc13907cbe083f2
    chmod +x "$OUT"/yt-dlp-* "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
    ;;
  x86_64-pc-windows-msvc)
    fetch "$YTDLP_BASE_URL/yt-dlp.exe" "$OUT/yt-dlp-$TARGET.exe" \
      66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a
    fetch "$FFMPEG_BASE_URL/ffmpeg-win32-x64" "$OUT/ffmpeg-$TARGET.exe" \
      04e1307997530f9cf2fe35cba2ca7e8875ca91da02f89d6c7243df819c94ad00
    fetch "$FFMPEG_BASE_URL/ffprobe-win32-x64" "$OUT/ffprobe-$TARGET.exe" \
      3a7e2dc003dc2cd1472827e4c7c4f056ae1ae0ae7c5bbc580c99b49827351ba4
    fetch_ffmpeg_license win32-x64 "$TARGET" \
      8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903
    ;;
  x86_64-unknown-linux-gnu)
    fetch "$YTDLP_BASE_URL/yt-dlp" "$OUT/yt-dlp-$TARGET" \
      1fa6733c37ea6fb51c99ad8fe785e7b7e5f3246c9b980230329d4fb72ed8d4d6
    fetch "$FFMPEG_BASE_URL/ffmpeg-linux-x64" "$OUT/ffmpeg-$TARGET" \
      e7e7fb30477f717e6f55f9180a70386c62677ef8a4d4d1a5d948f4098aa3eb99
    fetch "$FFMPEG_BASE_URL/ffprobe-linux-x64" "$OUT/ffprobe-$TARGET" \
      4f231a1960d83e403d08f7971e271707bec278a9ae18e21b8b5b03186668450d
    fetch_ffmpeg_license linux-x64 "$TARGET" \
      8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903
    chmod +x "$OUT"/yt-dlp-* "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
    ;;
  *)
    echo "unsupported target: $TARGET" >&2
    exit 1
    ;;
esac

cp THIRD_PARTY_NOTICES.md "$LICENSE_OUT/THIRD_PARTY_NOTICES.md"
