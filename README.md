# Mytory YT-DLP

개인 및 지인에게 배포하는, yt-dlp 기반의 로컬 우선 데스크톱 미디어 다운로드 도구입니다.

## 현재 구현 범위

- Tauri 2 + React/TypeScript + Rust 앱 셸
- 로컬 SQLite 초기 마이그레이션
- 고정된 인수 목록으로 Downloader 프로세스를 실행하는 `DownloaderRunner` 계약
- 실제 네트워크 없이 프로세스 경계를 검증하는 `Downloader Simulator`

Download Queue와 실제 yt-dlp 번들·실행은 다음 수직 슬라이스에서 구현합니다. 자동 업데이트, 원격 분석, 쿠키 저장은 아직 구현하지 않았습니다.

## 개발 환경

- Node.js `^20.19.0 || >=22.12.0`과 npm
- Rust stable toolchain
- 지원 OS별 [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/)

```bash
npm install
npm run tauri dev
```

## 검사

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

`Downloader Simulator` 테스트는 `simulator://success` 시나리오를 독립 실행 파일로 실행해 진행률과 완료 이벤트를 검증합니다. 실제 사이트나 yt-dlp를 실행하지 않습니다.
