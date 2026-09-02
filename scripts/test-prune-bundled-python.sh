#!/usr/bin/env bash
# Exercises removal of unused Tkinter runtime files from Bundled Python.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PRUNE="$ROOT/scripts/prune-bundled-python.sh"
TEMP="$(mktemp -d)"
trap 'rm -rf "$TEMP"' EXIT

PYTHON="$TEMP/python"
mkdir -p \
  "$PYTHON/lib/python3.13/lib-dynload" \
  "$PYTHON/lib/tcl9.0" \
  "$PYTHON/lib/tcl9" \
  "$PYTHON/lib/tk9.0" \
  "$PYTHON/lib/thread3.0.6" \
  "$PYTHON/lib/thread-other" \
  "$PYTHON/lib/itcl4.3.8" \
  "$PYTHON/bin"
printf 'unused tkinter extension\n' > "$PYTHON/lib/python3.13/lib-dynload/_tkinter.cpython-313-x86_64-linux-gnu.so"
printf 'tcl\n' > "$PYTHON/lib/libtcl9.0.so"
printf 'tk\n' > "$PYTHON/lib/libtcl9tk9.0.so"
printf 'thread\n' > "$PYTHON/lib/thread3.0.6/libtcl9thread3.0.6.so"
printf 'itcl\n' > "$PYTHON/lib/itcl4.3.8/libtcl9itcl4.3.8.so"
printf 'other thread component\n' > "$PYTHON/lib/thread-other/keep.txt"
printf 'license\n' > "$PYTHON/LICENSE.txt"
printf 'python\n' > "$PYTHON/bin/python3"

"$PRUNE" "$PYTHON"

grep -Fq 'fetch_python x86_64-unknown-linux-gnu 8af9a8214c71b2dd698005e39fab87aad02a994330508857da4e6d1ba7e6ddb6' "$ROOT/scripts/fetch-tools.sh"
grep -Fq 'prune-bundled-python.sh" "$PYTHON_OUT/x86_64-unknown-linux-gnu"' "$ROOT/scripts/fetch-tools.sh"

[[ ! -e "$PYTHON/lib/python3.13/lib-dynload/_tkinter.cpython-313-x86_64-linux-gnu.so" ]]
[[ ! -e "$PYTHON/lib/libtcl9.0.so" ]]
[[ ! -e "$PYTHON/lib/libtcl9tk9.0.so" ]]
[[ ! -e "$PYTHON/lib/thread3.0.6" ]]
[[ ! -e "$PYTHON/lib/itcl4.3.8" ]]
[[ ! -e "$PYTHON/lib/tcl9" ]]
[[ ! -e "$PYTHON/lib/tcl9.0" ]]
[[ ! -e "$PYTHON/lib/tk9.0" ]]
[[ -f "$PYTHON/lib/thread-other/keep.txt" ]]
[[ -f "$PYTHON/LICENSE.txt" ]]
[[ -f "$PYTHON/bin/python3" ]]
