#!/usr/bin/env bash
# Verifies distribution-facing Tauri configuration.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

node - "$ROOT/src-tauri/tauri.conf.json" <<'NODE'
const fs = require('fs');
const config = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));

if (config.bundle?.windows?.wix?.language !== 'ko-KR') {
  throw new Error('the Windows MSI installer language must be ko-KR');
}
NODE
