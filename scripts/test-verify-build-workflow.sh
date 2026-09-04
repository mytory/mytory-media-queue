#!/usr/bin/env bash
# Verifies that CI publishes only the installer bundles from each platform.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/build.yml"

if grep -Eq '\*\*/\*\.(app|exe|msi|AppImage|deb|dmg)' "$WORKFLOW"; then
  echo 'build workflow must not recursively glob release assets' >&2
  exit 1
fi

for bundle_file in \
  'src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg' \
  'src-tauri/target/release/bundle/nsis/*.exe' \
  'src-tauri/target/release/bundle/msi/*.msi' \
  'src-tauri/target/release/bundle/appimage/*.AppImage' \
  'src-tauri/target/release/bundle/deb/*.deb'; do
  grep -Fqx "              $bundle_file" "$WORKFLOW"
done

[[ "$(grep -Fc '          path: ${{ matrix.bundle_files }}' "$WORKFLOW")" == 1 ]]
[[ "$(grep -Fc '          files: ${{ matrix.bundle_files }}' "$WORKFLOW")" == 1 ]]
