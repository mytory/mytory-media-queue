#!/usr/bin/env bash
# Downloads checksum-verified Bundled Python, Bundled Runtime, Downloader wheels,
# and Bundled Media Toolchain files into src-tauri/binaries/.
#
# Usage: scripts/fetch-tools.sh <target-triple>
#   universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin \
#   | x86_64-pc-windows-msvc | x86_64-unknown-linux-gnu
set -euo pipefail

TARGET="${1:?usage: scripts/fetch-tools.sh <target-triple>}"
OUT="src-tauri/binaries"
LICENSE_OUT="$OUT/licenses"
WHEEL_OUT="$OUT/wheels"
PYTHON_OUT="$OUT/python"
YTDLP_VERSION="2026.8.19"
EJS_VERSION="0.8.0"
PYTHON_BUILD_VERSION="20260825"
PYTHON_VERSION="3.13.15"
DENO_VERSION="2.3.1"
FFMPEG_VERSION="b6.1.1"
YTDLP_WHEEL_URL="https://files.pythonhosted.org/packages/69/b2/8cd1613f56eed7ceb64fbd4df3f1c01246bfb098e6f398228bafda22b80b/yt_dlp-2026.8.19-py3-none-any.whl"
EJS_WHEEL_URL="https://github.com/yt-dlp/ejs/releases/download/0.8.0/yt_dlp_ejs-0.8.0-py3-none-any.whl"
FFMPEG_BASE_URL="https://github.com/eugeneware/ffmpeg-static/releases/download/$FFMPEG_VERSION"

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

fetch_ffmpeg_license() {
  local source_name="$1" target_name="$2" checksum="$3"
  fetch "$FFMPEG_BASE_URL/$source_name.LICENSE" "$LICENSE_OUT/ffmpeg-$target_name-LICENSE" "$checksum"
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

    fetch "$FFMPEG_BASE_URL/ffmpeg-darwin-x64" /tmp/ffmpeg-x64 ebdddc936f61e14049a2d4b549a412b8a40deeff6540e58a9f2a2da9e6b18894
    fetch "$FFMPEG_BASE_URL/ffprobe-darwin-x64" /tmp/ffprobe-x64 fa3add0ce901f7241abe0dfc0155d958fc834aca3f8ce61f87cc712ae669c1e0
    fetch "$FFMPEG_BASE_URL/ffmpeg-darwin-arm64" /tmp/ffmpeg-arm64 a90e3db6a3fd35f6074b013f948b1aa45b31c6375489d39e572bea3f18336584
    fetch "$FFMPEG_BASE_URL/ffprobe-darwin-arm64" /tmp/ffprobe-arm64 bb2db6f5d8cef919da12fbf592119a987202a8c060a886f3cab091f9cab90b64
    lipo -create /tmp/ffmpeg-x64 /tmp/ffmpeg-arm64 -output "$OUT/ffmpeg-universal-apple-darwin"
    lipo -create /tmp/ffprobe-x64 /tmp/ffprobe-arm64 -output "$OUT/ffprobe-universal-apple-darwin"
    cp /tmp/ffmpeg-arm64 "$OUT/ffmpeg-aarch64-apple-darwin"; cp /tmp/ffmpeg-x64 "$OUT/ffmpeg-x86_64-apple-darwin"
    cp /tmp/ffprobe-arm64 "$OUT/ffprobe-aarch64-apple-darwin"; cp /tmp/ffprobe-x64 "$OUT/ffprobe-x86_64-apple-darwin"
    fetch_ffmpeg_license darwin-x64 universal-apple-darwin 2e1d16c72fd74e12063776371da757322f8b77589386532f4fd8634bde7de1af
    fetch_ffmpeg_license darwin-arm64 universal-apple-darwin-arm64 cb48bf09a11f5fb576cddb0431c8f5ed0a60157a9ec942adffc13907cbe083f2
    chmod +x "$OUT"/deno-* "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
    ;;
  x86_64-pc-windows-msvc)
    fetch_python x86_64-pc-windows-msvc c1dc1e267f2a81493ce6e94837263f648f1eb6d0df73a1492469c1fed025ce8f
    fetch_deno x86_64-pc-windows-msvc deno-x86_64-pc-windows-msvc.zip 1b968541d115420ba04f7a5fbb5d0f8d620d9d87d492b66da5c97ca07e269b9b "$OUT/deno-$TARGET.exe"
    fetch "$FFMPEG_BASE_URL/ffmpeg-win32-x64" "$OUT/ffmpeg-$TARGET.exe" 04e1307997530f9cf2fe35cba2ca7e8875ca91da02f89d6c7243df819c94ad00
    fetch "$FFMPEG_BASE_URL/ffprobe-win32-x64" "$OUT/ffprobe-$TARGET.exe" 3a7e2dc003dc2cd1472827e4c7c4f056ae1ae0ae7c5bbc580c99b49827351ba4
    fetch_ffmpeg_license win32-x64 "$TARGET" 8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903
    ;;
  x86_64-unknown-linux-gnu)
    fetch_python x86_64-unknown-linux-gnu 8af9a8214c71b2dd698005e39fab87aad02a994330508857da4e6d1ba7e6ddb6
    fetch_deno x86_64-unknown-linux-gnu deno-x86_64-unknown-linux-gnu.zip b2920265e633215959b09a32b67f46c93362842bbfd27c96e8acc2d24b66f563 "$OUT/deno-$TARGET"
    fetch "$FFMPEG_BASE_URL/ffmpeg-linux-x64" "$OUT/ffmpeg-$TARGET" e7e7fb30477f717e6f55f9180a70386c62677ef8a4d4d1a5d948f4098aa3eb99
    fetch "$FFMPEG_BASE_URL/ffprobe-linux-x64" "$OUT/ffprobe-$TARGET" 4f231a1960d83e403d08f7971e271707bec278a9ae18e21b8b5b03186668450d
    fetch_ffmpeg_license linux-x64 "$TARGET" 8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903
    chmod +x "$OUT"/deno-* "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
    ;;
  *) echo "unsupported target: $TARGET" >&2; exit 1 ;;
esac

cp THIRD_PARTY_NOTICES.md "$LICENSE_OUT/THIRD_PARTY_NOTICES.md"
