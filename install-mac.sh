#!/bin/bash
set -euo pipefail

REPOSITORY_URL="https://github.com/mytory/mytory-media-queue.git"
APP_NAME="Mytory Media Queue.app"
INSTALL_DIR="${HOME}/Applications"
TARGET_APP="${INSTALL_DIR}/${APP_NAME}"
SOURCE_DIR=""
WORK_DIR=""
STAGE_DIR=""
BACKUP_APP=""

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "이 설치 스크립트는 macOS에서만 실행할 수 있습니다." >&2
  exit 1
fi

for command_name in git curl node npm rustup cargo make clang lipo shasum unzip tar; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "필요한 명령을 찾을 수 없습니다: $command_name" >&2
    echo "Xcode Command Line Tools, Node.js, Rust를 설치한 뒤 다시 실행하세요." >&2
    exit 1
  fi
done

if ! xcode-select -p >/dev/null 2>&1; then
  echo "Xcode Command Line Tools가 필요합니다. xcode-select --install을 실행하세요." >&2
  exit 1
fi

if [[ "$#" -gt 1 ]]; then
  echo "사용법: install-mac.sh [저장소 경로]" >&2
  exit 1
fi

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/mytory-media-queue.XXXXXX")"

cleanup() {
  if [[ -n "$BACKUP_APP" && -d "$BACKUP_APP" && ! -e "$TARGET_APP" ]]; then
    mv "$BACKUP_APP" "$TARGET_APP" || true
  fi
  if [[ -n "$STAGE_DIR" ]]; then
    rm -rf "$STAGE_DIR"
  fi
  rm -rf "$WORK_DIR"
}
trap cleanup EXIT

if [[ "$#" -eq 1 ]]; then
  if [[ ! -d "$1" ]]; then
    echo "저장소 경로가 없습니다: $1" >&2
    exit 1
  fi
  SOURCE_DIR="$(cd "$1" && pwd)"
  if [[ ! -f "${SOURCE_DIR}/package.json" || ! -f "${SOURCE_DIR}/src-tauri/tauri.conf.json" ]]; then
    echo "유효한 Mytory Media Queue 저장소 경로가 아닙니다: $SOURCE_DIR" >&2
    exit 1
  fi
else
  SOURCE_DIR="${WORK_DIR}/source"
  echo "소스 코드를 내려받습니다..."
  git clone --depth 1 "$REPOSITORY_URL" "$SOURCE_DIR"
fi

cd "$SOURCE_DIR"
echo "Universal macOS용 빌드 도구를 준비합니다..."
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm ci
bash scripts/fetch-tools.sh universal-apple-darwin

echo "앱을 빌드합니다. 내장 FFmpeg 컴파일 때문에 시간이 걸릴 수 있습니다..."
npm run tauri build -- --bundles app --target universal-apple-darwin

SOURCE_APP="${SOURCE_DIR}/src-tauri/target/universal-apple-darwin/release/bundle/macos/${APP_NAME}"
if [[ ! -d "$SOURCE_APP" ]]; then
  echo "빌드 결과에서 ${APP_NAME}을(를) 찾을 수 없습니다: $SOURCE_APP" >&2
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
