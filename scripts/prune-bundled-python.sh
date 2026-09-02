#!/usr/bin/env bash
# Removes Tkinter and its Tcl/Tk runtime, which the Downloader does not use.
set -euo pipefail

PYTHON_DIRECTORY="${1:?usage: scripts/prune-bundled-python.sh <python-directory>}"
[[ -d "$PYTHON_DIRECTORY" ]] || {
  echo "missing Bundled Python directory: $PYTHON_DIRECTORY" >&2
  exit 1
}

shopt -s nullglob
rm -f "$PYTHON_DIRECTORY"/lib/python*/lib-dynload/_tkinter.*.so
rm -f "$PYTHON_DIRECTORY"/lib/libtcl*.so "$PYTHON_DIRECTORY"/lib/libtk*.so
rm -rf \
  "$PYTHON_DIRECTORY/lib/itcl4.3.8" \
  "$PYTHON_DIRECTORY/lib/tcl9" \
  "$PYTHON_DIRECTORY/lib/tcl9.0" \
  "$PYTHON_DIRECTORY/lib/thread3.0.6" \
  "$PYTHON_DIRECTORY/lib/tk9.0"

if find "$PYTHON_DIRECTORY"/lib -type f \( -name '_tkinter.*.so' -o -name 'libtcl*.so' -o -name 'libtk*.so' \) -print -quit | grep -q .; then
  echo "Tkinter files remain in $PYTHON_DIRECTORY" >&2
  exit 1
fi
