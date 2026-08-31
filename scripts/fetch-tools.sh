#!/usr/bin/env bash
# Downloads checksum-verified Bundled Python, Bundled Runtime, Downloader wheels,
# and Bundled Media Toolchain files into src-tauri/binaries/.
#
# Usage: scripts/fetch-tools.sh <target-triple>
#   universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin \
#   | x86_64-pc-windows-msvc | x86_64-unknown-linux-gnu
set -euo pipefail

TARGET="${1:?usage: scripts/fetch-tools.sh <target-triple>}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="src-tauri/binaries"
LICENSE_OUT="$OUT/licenses"
WHEEL_OUT="$OUT/wheels"
PYTHON_OUT="$OUT/python"
YTDLP_VERSION="2026.8.19"
EJS_VERSION="0.8.0"
PYTHON_BUILD_VERSION="20260825"
PYTHON_VERSION="3.13.15"
DENO_VERSION="2.3.1"
YTDLP_WHEEL_URL="https://files.pythonhosted.org/packages/69/b2/8cd1613f56eed7ceb64fbd4df3f1c01246bfb098e6f398228bafda22b80b/yt_dlp-2026.8.19-py3-none-any.whl"
EJS_WHEEL_URL="https://github.com/yt-dlp/ejs/releases/download/0.8.0/yt_dlp_ejs-0.8.0-py3-none-any.whl"
FFMPEG_VERSION="7.1.1"
FFMPEG_SOURCE_ARCHIVE="$OUT/ffmpeg-source/ffmpeg-$FFMPEG_VERSION.tar.xz"

mkdir -p "$OUT" "$LICENSE_OUT" "$WHEEL_OUT" "$PYTHON_OUT"

fetch() {
  local url="$1" destination="$2" checksum="$3"
  curl -fL --retry 3 -o "$destination" "$url"
  printf '%s  %s\n' "$checksum" "$destination" | shasum -a 256 -c -
}

extract_wheel_license() {
  local wheel="$1" destination="$2" checksum="$3"
  unzip -p "$wheel" '*/licenses/LICENSE' > "$destination"
  printf '%s  %s\n' "$checksum" "$destination" | shasum -a 256 -c -
}

fetch_python() {
  local triple="$1" checksum="$2"
  local archive="/tmp/cpython-$triple.tar.gz"
  local encoded_name="cpython-$PYTHON_VERSION%2B$PYTHON_BUILD_VERSION-$triple-install_only_stripped.tar.gz"
  fetch "https://github.com/astral-sh/python-build-standalone/releases/download/$PYTHON_BUILD_VERSION/$encoded_name" "$archive" "$checksum"
  rm -rf "$PYTHON_OUT/$triple"
  mkdir -p "$PYTHON_OUT/$triple"
  tar -xzf "$archive" -C "$PYTHON_OUT/$triple" --strip-components=1
}

fetch_deno() {
  local triple="$1" asset="$2" checksum="$3" output="$4"
  local archive="/tmp/$asset" member="deno"
  [[ "$asset" == *windows* ]] && member="deno.exe"
  fetch "https://github.com/denoland/deno/releases/download/v$DENO_VERSION/$asset" "$archive" "$checksum"
  unzip -p "$archive" "$member" > "$output"
  chmod +x "$output"
}

# Wheels are copied to the app data directory during first startup, not executed
# from the read-only application resource directory.
fetch "$YTDLP_WHEEL_URL" "$WHEEL_OUT/yt-dlp.whl" \
  1d57897e94c6665a0a6f9bc54b34e584284e32c034ffab3a7df25d8f7b24eedf
fetch "$EJS_WHEEL_URL" "$WHEEL_OUT/yt-dlp-ejs.whl" \
  79300e5fca7f937a1eeede11f0456862c1b41107ce1d726871e0207424f4bdb4
extract_wheel_license "$WHEEL_OUT/yt-dlp.whl" "$LICENSE_OUT/yt-dlp-LICENSE" \
  7e12e5df4bae12cb21581ba157ced20e1986a0508dd10d0e8a4ab9a4cf94e85c
extract_wheel_license "$WHEEL_OUT/yt-dlp-ejs.whl" "$LICENSE_OUT/yt-dlp-ejs-LICENSE" \
  b5065838cbac452dfc855ba6e6e031481ad2c68406f70d21ead9321374653e6c
fetch "https://raw.githubusercontent.com/astral-sh/python-build-standalone/$PYTHON_BUILD_VERSION/LICENSE" "$LICENSE_OUT/python-build-standalone-LICENSE" \
  1f256ecad192880510e84ad60474eab7589218784b9a50bc7ceee34c2b91f1d5
fetch "https://raw.githubusercontent.com/python/cpython/v$PYTHON_VERSION/LICENSE" "$LICENSE_OUT/python-LICENSE" \
  78b12c3a81360b357002334f0e70ea0e92eebf7a9b358805c03c48484945f3bb
fetch "https://raw.githubusercontent.com/denoland/deno/v$DENO_VERSION/LICENSE.md" "$LICENSE_OUT/deno-LICENSE" \
  ee79dd206fb5aa60a7485104ea78ba2a78935d6586cbb83c616db2579543f756

case "$TARGET" in
  universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin)
    fetch_python aarch64-apple-darwin 149038dd0c194c25d4616d7e42a35f67f2edee96412788f74115819b6a4c8548
    fetch_python x86_64-apple-darwin d33d61f7f4982c94216e14a43599c75657b7d0839277fc72bc6dbac53e8229bc
    fetch_deno aarch64-apple-darwin deno-aarch64-apple-darwin.zip e3d3d7b21ce89105d96c316e9370b1f05aa6e87687f40faf37a39a613a477014 /tmp/deno-arm64
    fetch_deno x86_64-apple-darwin deno-x86_64-apple-darwin.zip ba34eb6ec164a0f89f5431fc1989a31f7896f76d074415f64ea70509de39fc56 /tmp/deno-x64
    lipo -create /tmp/deno-arm64 /tmp/deno-x64 -output "$OUT/deno-universal-apple-darwin"
    cp /tmp/deno-arm64 "$OUT/deno-aarch64-apple-darwin"
    cp /tmp/deno-x64 "$OUT/deno-x86_64-apple-darwin"

    bash "$ROOT/scripts/build-ffmpeg.sh" universal-apple-darwin
    chmod +x "$OUT"/deno-* "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
    ;;
  x86_64-pc-windows-msvc)
    fetch_python x86_64-pc-windows-msvc c1dc1e267f2a81493ce6e94837263f648f1eb6d0df73a1492469c1fed025ce8f
    fetch_deno x86_64-pc-windows-msvc deno-x86_64-pc-windows-msvc.zip 1b968541d115420ba04f7a5fbb5d0f8d620d9d87d492b66da5c97ca07e269b9b "$OUT/deno-$TARGET.exe"
    bash "$ROOT/scripts/build-ffmpeg.sh" "$TARGET"
    ;;
  x86_64-unknown-linux-gnu)
    fetch_python x86_64-unknown-linux-gnu 8af9a8214c71b2dd698005e39fab87aad02a994330508857da4e6d1ba7e6ddb6
    fetch_deno x86_64-unknown-linux-gnu deno-x86_64-unknown-linux-gnu.zip b2920265e633215959b09a32b67f46c93362842bbfd27c96e8acc2d24b66f563 "$OUT/deno-$TARGET"
    bash "$ROOT/scripts/build-ffmpeg.sh" "$TARGET"
    chmod +x "$OUT"/deno-* "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
    ;;
  *) echo "unsupported target: $TARGET" >&2; exit 1 ;;
esac

bash "$ROOT/scripts/prepare-ffmpeg-source-material.sh" \
  "$FFMPEG_SOURCE_ARCHIVE" \
  "$LICENSE_OUT/ffmpeg-LGPL-2.1-or-later.txt"
cp THIRD_PARTY_NOTICES.md "$LICENSE_OUT/THIRD_PARTY_NOTICES.md"
