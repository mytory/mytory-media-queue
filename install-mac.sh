#!/bin/bash
set -euo pipefail

REPOSITORY="mytory/mytory-media-queue"
API_URL="https://api.github.com/repos/${REPOSITORY}/releases/latest"
APP_NAME="Mytory Media Queue.app"
INSTALL_DIR="${HOME}/Applications"
TARGET_APP="${INSTALL_DIR}/${APP_NAME}"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "이 설치 스크립트는 macOS에서만 실행할 수 있습니다." >&2
  exit 1
fi

for command_name in curl hdiutil ditto python3; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "필요한 명령을 찾을 수 없습니다: $command_name" >&2
    exit 1
  fi
done

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/mytory-media-queue.XXXXXX")"
MOUNT_DIR="${WORK_DIR}/mount"
STAGE_DIR=""
BACKUP_APP=""
MOUNTED=0
mkdir "$MOUNT_DIR"

cleanup() {
  if [[ "$MOUNTED" -eq 1 ]]; then
    hdiutil detach "$MOUNT_DIR" -quiet || true
  fi
  if [[ -n "$BACKUP_APP" && -d "$BACKUP_APP" && ! -e "$TARGET_APP" ]]; then
    mv "$BACKUP_APP" "$TARGET_APP" || true
  fi
  if [[ -n "$STAGE_DIR" ]]; then
    rm -rf "$STAGE_DIR"
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

echo "최신 macOS 설치 파일 정보를 가져옵니다..."
curl --fail --location --silent --show-error "$API_URL" -o "${WORK_DIR}/release.json"
DMG_URL="$(python3 - "${WORK_DIR}/release.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as release_file:
    release = json.load(release_file)

assets = [asset["browser_download_url"] for asset in release.get("assets", [])
          if asset.get("name", "").lower().endswith(".dmg")]
if len(assets) != 1:
    raise SystemExit(f"최신 릴리스에서 DMG 파일을 하나 찾을 수 없습니다 (발견: {len(assets)}개).")
print(assets[0])
PY
)"

echo "DMG를 다운로드합니다..."
curl --fail --location --silent --show-error "$DMG_URL" -o "${WORK_DIR}/app.dmg"
hdiutil attach -quiet -nobrowse -readonly -mountpoint "$MOUNT_DIR" "${WORK_DIR}/app.dmg"
MOUNTED=1

SOURCE_APP="$(find "$MOUNT_DIR" -type d -name "$APP_NAME" -print -quit)"
if [[ -z "$SOURCE_APP" ]]; then
  echo "DMG에서 ${APP_NAME}을(를) 찾을 수 없습니다." >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
STAGE_DIR="$(mktemp -d "${INSTALL_DIR}/.mytory-media-queue.XXXXXX")"
ditto "$SOURCE_APP" "${STAGE_DIR}/${APP_NAME}"

if [[ -e "$TARGET_APP" ]]; then
  BACKUP_APP="${STAGE_DIR}/previous.app"
  mv "$TARGET_APP" "$BACKUP_APP"
fi

if ! mv "${STAGE_DIR}/${APP_NAME}" "$TARGET_APP"; then
  echo "앱 설치에 실패했습니다." >&2
  exit 1
fi
BACKUP_APP=""

echo "설치했습니다: ${TARGET_APP}"
echo "Finder에서 응용 프로그램 폴더를 열어 Mytory Media Queue를 실행하세요."
