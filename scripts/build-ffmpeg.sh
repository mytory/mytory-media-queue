#!/usr/bin/env bash
# Builds checksum-verified, LGPL-only FFmpeg and FFprobe binaries for Tauri.
#
# Usage: scripts/build-ffmpeg.sh <target-triple>
#   universal-apple-darwin | aarch64-apple-darwin | x86_64-apple-darwin
#   | x86_64-pc-windows-msvc | x86_64-unknown-linux-gnu
set -euo pipefail

TARGET="${1:?usage: scripts/build-ffmpeg.sh <target-triple>}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/src-tauri/binaries"
FFMPEG_VERSION="7.1.1"
FFMPEG_SOURCE_URL="https://ffmpeg.org/releases/ffmpeg-$FFMPEG_VERSION.tar.xz"
FFMPEG_SOURCE_SHA256="733984395e0dbbe5c046abda2dc49a5544e7e0e1e2366bba849222ae9e3a03b1"
JOBS="${FFMPEG_BUILD_JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.ncpu)}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/mytory-ffmpeg.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

SOURCE_OUT="$OUT/ffmpeg-source"
mkdir -p "$OUT" "$SOURCE_OUT"
ARCHIVE="$WORK/ffmpeg-$FFMPEG_VERSION.tar.xz"
SOURCE_ARCHIVE="$SOURCE_OUT/ffmpeg-$FFMPEG_VERSION.tar.xz"
curl -fL --retry 3 -o "$ARCHIVE" "$FFMPEG_SOURCE_URL"
printf '%s  %s\n' "$FFMPEG_SOURCE_SHA256" "$ARCHIVE" | shasum -a 256 -c -
cp "$ARCHIVE" "$SOURCE_ARCHIVE"

build_one() {
  local arch="$1" output_suffix="$2" extension="$3"
  local source="$WORK/source-$arch"
  shift 3

  mkdir "$source"
  tar -xJf "$ARCHIVE" -C "$source" --strip-components=1
  (
    cd "$source"
    ./configure \
      --prefix=/usr \
      --disable-debug \
      --disable-doc \
      --disable-gpl \
      --disable-nonfree \
      --disable-shared \
      --disable-version3 \
      --disable-x86asm \
      --enable-static \
      --enable-ffmpeg \
      --enable-ffprobe \
      "$@"
    make -j "$JOBS" ffmpeg ffprobe
  )
  cp "$source/ffmpeg$extension" "$OUT/ffmpeg-$output_suffix$extension"
  cp "$source/ffprobe$extension" "$OUT/ffprobe-$output_suffix$extension"
}

case "$TARGET" in
  universal-apple-darwin)
    build_one arm64 aarch64-apple-darwin '' --arch=arm64 --cc=clang --extra-cflags='-arch arm64' --extra-ldflags='-arch arm64'
    build_one x86_64 x86_64-apple-darwin '' --arch=x86_64 --cc=clang --extra-cflags='-arch x86_64' --extra-ldflags='-arch x86_64'
    lipo -create \
      "$OUT/ffmpeg-aarch64-apple-darwin" \
      "$OUT/ffmpeg-x86_64-apple-darwin" \
      -output "$OUT/ffmpeg-$TARGET"
    lipo -create \
      "$OUT/ffprobe-aarch64-apple-darwin" \
      "$OUT/ffprobe-x86_64-apple-darwin" \
      -output "$OUT/ffprobe-$TARGET"
    ;;
  aarch64-apple-darwin)
    build_one arm64 "$TARGET" '' --arch=arm64 --cc=clang --extra-cflags='-arch arm64' --extra-ldflags='-arch arm64'
    ;;
  x86_64-apple-darwin)
    build_one x86_64 "$TARGET" '' --arch=x86_64 --cc=clang --extra-cflags='-arch x86_64' --extra-ldflags='-arch x86_64'
    ;;
  x86_64-pc-windows-msvc)
    build_one x86_64 "$TARGET" .exe \
      --arch=x86_64 \
      --target-os=mingw32 \
      --cc=gcc \
      --extra-ldflags=-static
    ;;
  x86_64-unknown-linux-gnu)
    build_one x86_64 "$TARGET" '' --arch=x86_64
    ;;
  *)
    echo "unsupported target: $TARGET" >&2
    exit 1
    ;;
esac

chmod +x "$OUT"/ffmpeg-* "$OUT"/ffprobe-*
"$ROOT/scripts/verify-ffmpeg-build.sh" "$TARGET" "$OUT"
