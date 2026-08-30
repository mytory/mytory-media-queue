# Mytory YT-DLP

개인 및 지인에게 비공개 배포하는, [yt-dlp](https://github.com/yt-dlp/yt-dlp) 기반의 로컬 우선 데스크톱 미디어 다운로드 도구입니다.

> **개발 중**: Download Queue 영속화·재개·스케줄링, 실제 다운로드 실행, 출력 옵션(썸네일·자막), 쿠키 파일, 실패 복구(3회 재시도·진단 로그), 이력 정리, yt-dlp/FFmpeg/FFprobe sidecar 번들, GitHub Actions 기반 설치 패키지 빌드를 구현했습니다. Deno/`yt-dlp-ejs` 번들(YouTube 자바스크립트 추출기)과 Managed Update는 아직 완성되지 않았습니다.

## 목표

- Windows x64, macOS Universal, Linux x64에서 별도 의존성 설치 없이 동작
- 여러 URL을 하나의 **Download Queue**에 추가하고, 실행 중에도 새 작업을 추가
- 기본 MP4 호환 출력과 안전한 고정 다운로드 인수 사용
- 로컬 SQLite에 작업 이력만 저장하고, 원격 분석과 자동 오류 전송을 하지 않음
- Downloader와 Bundled Extractor만 체크섬 검증 후 함께 교체하는 **Managed Update** 제공

전체 제품 범위와 단계별 계획은 [docs/PLAN.md](docs/PLAN.md)을 참고하세요.

## 현재 구현

- Tauri 2 + React/TypeScript + Rust 앱 셸
- SQLite 마이그레이션과 영속 **Download Queue**
- 다중 URL 등록, MP4 호환 기본 Output Preset(H.264 영상·AAC 오디오 우선), FIFO 순서 보존
- 앱 시작 시 중단된 `running` 작업을 `queued`로 복구
- 1~5 범위의 동시성 설정(기본 3) 및 FIFO 슬롯 할당 기반
- 실행 중인 Downloader 프로세스까지 종료하는 취소·제거, 사용자 재시도
- 썸네일 기본 저장, 선택형 자막(한국어·영어, `.vtt`), 쿠키 파일(`cookies.txt`) 전달
- 일시적 네트워크 오류 자동 재시도(최대 3회), 그 외 실패는 정제된 진단 로그 저장·복사·결과 폴더 열기
- 완료·실패·취소 이력 지우기(파일은 삭제하지 않음)와 90일 이전 완료 이력 정리
- 셸을 만들지 않고 고정 인수 목록으로 Downloader 프로세스를 실행하는 `DownloaderRunner`
- 실제 네트워크 없이 프로세스 경계·진행률·성공·실패·중단·재시도를 재현하는 **Downloader Simulator**

## 개발 환경

- Node.js `^20.19.0 || >=22.12.0`과 npm
- Rust stable toolchain 및 `rustfmt`
- 지원 OS별 [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/)

## 시작하기

```bash
npm install
npm run tauri dev
```

## 검사

```bash
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

테스트는 실제 yt-dlp나 외부 사이트를 호출하지 않습니다. `Downloader Simulator`가 독립 프로세스로 진행률, 성공, 일시 네트워크 실패, 권한 실패, 중단을 결정적으로 출력합니다.

## 설치 파일 빌드 (GitHub Actions)

설치 파일(Windows NSIS/MSI, macOS DMG, Linux deb/AppImage)은 **GitHub Actions**에서 빌드합니다. `mytory-yt-dlp`는 비공개 저장소이며, 릴리스 페이지에서 내려받을 수 있습니다.

- 수동 빌드: Actions 탭 → **Build installers** → Run workflow
- 버전 릴리스: 태그를 푸시하면 설치 파일이 자동으로 GitHub Release에 첨부됩니다

  ```bash
  git tag v0.1.0
  git push origin v0.1.0
  ```

- 다운로드: `https://github.com/mytory/mytory-yt-dlp/releases`

CI는 각 플랫폼용 yt-dlp/FFmpeg/FFprobe를 `scripts/fetch-tools.sh`로 내려받아 sidecar로 번들합니다. 로컬에서 직접 빌드할 때도 같은 스크립트를 먼저 실행해야 합니다 (`bash scripts/fetch-tools.sh <target-triple>`).

## 개인정보 및 보안 원칙

- 원격 분석, 자동 오류 전송을 하지 않습니다.
- Cookie Source는 향후 Downloader 실행에만 전달하며 DB·UI 이벤트·진단 로그에 저장하지 않습니다.
- UI는 임의 yt-dlp 인수를 전달할 수 없습니다.
- Downloader 실행은 셸 문자열이 아닌 절대 경로와 고정 인수 목록을 사용합니다.

## 배포 상태

프로젝트는 비공개 GitLab 저장소에서 관리됩니다. 지원 플랫폼용 무서명 설치 파일, Gatekeeper/SmartScreen 안내, 라이선스 고지는 패키징 단계에서 추가합니다.
