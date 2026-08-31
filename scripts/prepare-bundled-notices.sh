#!/usr/bin/env bash
# Preserves notices embedded in yt-dlp-ejs and inventories legal files in Bundled Python.
set -euo pipefail

EJS_WHEEL="${1:?usage: scripts/prepare-bundled-notices.sh <ejs-wheel> <python-directory> <target> <output-directory>}"
PYTHON_DIRECTORY="${2:?usage: scripts/prepare-bundled-notices.sh <ejs-wheel> <python-directory> <target> <output-directory>}"
TARGET="${3:?usage: scripts/prepare-bundled-notices.sh <ejs-wheel> <python-directory> <target> <output-directory>}"
OUT="${4:?usage: scripts/prepare-bundled-notices.sh <ejs-wheel> <python-directory> <target> <output-directory>}"

[[ -f "$EJS_WHEEL" ]] || {
  echo "missing yt-dlp-ejs wheel: $EJS_WHEEL" >&2
  exit 1
}
[[ -d "$PYTHON_DIRECTORY" ]] || {
  echo "missing Bundled Python directory: $PYTHON_DIRECTORY" >&2
  exit 1
}
mkdir -p "$OUT"

EJS_NOTICE="$OUT/yt-dlp-ejs-BUNDLED-NOTICES.txt"
unzip -p "$EJS_WHEEL" 'yt_dlp_ejs/yt/solver/lib.min.js' | sed -n '1,/\*\//p' > "$EJS_NOTICE"
grep -Fq 'Name: meriyah' "$EJS_NOTICE"
grep -Fq 'Name: astring' "$EJS_NOTICE"

INVENTORY="$OUT/python-$TARGET-NOTICE-INVENTORY.txt"
{
  printf '# Bundled Python notice inventory for %s\n' "$TARGET"
  printf '# SHA-256  relative path\n'
  while IFS= read -r file; do
    relative_path="${file#"$PYTHON_DIRECTORY"/}"
    checksum="$(shasum -a 256 "$file" | awk '{print $1}')"
    printf '%s  %s\n' "$checksum" "$relative_path"
  done < <(
    find "$PYTHON_DIRECTORY" -type f \( \
      -iname 'license*' -o \
      -iname 'notice*' -o \
      -iname 'copying*' -o \
      -iname 'license.terms' \
    \) -print | LC_ALL=C sort
  )
} > "$INVENTORY"

grep -Eq '^[[:xdigit:]]{64}  ' "$INVENTORY" || {
  echo "no license or notice files found in $PYTHON_DIRECTORY" >&2
  exit 1
}
