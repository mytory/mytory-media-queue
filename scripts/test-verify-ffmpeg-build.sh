#!/usr/bin/env bash
# Exercises the public contract of verify-ffmpeg-build.sh without compiling FFmpeg.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERIFY="$ROOT/scripts/verify-ffmpeg-build.sh"
TEMP="$(mktemp -d)"
trap 'rm -rf "$TEMP"' EXIT

make_tool() {
  local name="$1" license="$2"
  local program="${name%%-*}"
  cat > "$TEMP/$name" <<EOF
#!/usr/bin/env bash
if [[ "\$1" == "-version" ]]; then
  cat <<'OUTPUT'
$program version 7.1.1
configuration: --disable-gpl --disable-version3 --disable-nonfree --disable-shared --enable-static
OUTPUT
else
  printf '%s\\n' '$license'
fi
EOF
  chmod +x "$TEMP/$name"
}

make_tool ffmpeg-x86_64-unknown-linux-gnu 'GNU Lesser General Public License version 2.1 or later'
make_tool ffprobe-x86_64-unknown-linux-gnu 'GNU Lesser General Public License version 2.1 or later'
"$VERIFY" x86_64-unknown-linux-gnu "$TEMP"

grep -Fq 'FFMPEG_VERSION="7.1.1"' "$ROOT/scripts/build-ffmpeg.sh"
grep -Fq 'FFMPEG_SOURCE_SHA256="733984395e0dbbe5c046abda2dc49a5544e7e0e1e2366bba849222ae9e3a03b1"' "$ROOT/scripts/build-ffmpeg.sh"
if grep -Fq 'eugeneware/ffmpeg-static' "$ROOT/scripts/fetch-tools.sh"; then
  echo 'fetch-tools.sh must not download FFmpeg from ffmpeg-static' >&2
  exit 1
fi

make_tool ffprobe-x86_64-unknown-linux-gnu 'GNU General Public License version 3 or later'
if "$VERIFY" x86_64-unknown-linux-gnu "$TEMP" >/dev/null 2>&1; then
  echo 'expected GPL output to be rejected' >&2
  exit 1
fi
