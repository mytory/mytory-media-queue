#!/usr/bin/env bash
# Exercises wheel notice extraction and deterministic Bundled Python inventories.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREPARE="$ROOT/scripts/prepare-bundled-notices.sh"
TEMP="$(mktemp -d)"
trap 'rm -rf "$TEMP"' EXIT

mkdir -p "$TEMP/wheel/yt_dlp_ejs/yt/solver" "$TEMP/python/vendor" "$TEMP/licenses"
cat > "$TEMP/wheel/yt_dlp_ejs/yt/solver/lib.min.js" <<'EOF'
/*
 * Bundled Dependencies:
 * Name: meriyah
 * License: ISC
 * Copyright (c) 2019 and later, KFlash and others.
 * Name: astring
 * License: MIT
 */
minified_source();
EOF
(
  cd "$TEMP/wheel"
  zip -q "$TEMP/yt-dlp-ejs.whl" yt_dlp_ejs/yt/solver/lib.min.js
)
printf 'root license\n' > "$TEMP/python/LICENSE.txt"
printf 'vendor notice\n' > "$TEMP/python/vendor/NOTICE"
printf 'vendor copying\n' > "$TEMP/python/vendor/COPYING"
printf 'not a notice\n' > "$TEMP/python/vendor/README.md"

"$PREPARE" "$TEMP/yt-dlp-ejs.whl" "$TEMP/python" test-target "$TEMP/licenses"
cp "$TEMP/licenses/python-test-target-NOTICE-INVENTORY.txt" "$TEMP/first-inventory.txt"
"$PREPARE" "$TEMP/yt-dlp-ejs.whl" "$TEMP/python" test-target "$TEMP/licenses"
cmp "$TEMP/first-inventory.txt" "$TEMP/licenses/python-test-target-NOTICE-INVENTORY.txt"

grep -Fq 'Name: meriyah' "$TEMP/licenses/yt-dlp-ejs-BUNDLED-NOTICES.txt"
grep -Fq 'Name: astring' "$TEMP/licenses/yt-dlp-ejs-BUNDLED-NOTICES.txt"
grep -Fq 'LICENSE.txt' "$TEMP/licenses/python-test-target-NOTICE-INVENTORY.txt"
grep -Fq 'vendor/NOTICE' "$TEMP/licenses/python-test-target-NOTICE-INVENTORY.txt"
grep -Fq 'vendor/COPYING' "$TEMP/licenses/python-test-target-NOTICE-INVENTORY.txt"
if grep -Fq 'README.md' "$TEMP/licenses/python-test-target-NOTICE-INVENTORY.txt"; then
  echo 'non-notice file leaked into the inventory' >&2
  exit 1
fi
