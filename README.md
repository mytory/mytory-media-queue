# Mytory Media Queue

Mytory Media Queue는 [yt-dlp](https://github.com/yt-dlp/yt-dlp)와 FFmpeg을 번들로 사용하는 로컬 우선 데스크톱 미디어 다운로드 도구입니다. 제3자 소프트웨어의 라이선스와 고지는 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)를 참고하세요.

> **v1.0.1**: 첫 안정 릴리스의 설치 파일 빌드 호환성을 보완한 patch 릴리스입니다. 설치 패키지는 Bundled Python, Downloader(`yt-dlp` wheel), Bundled Extractor(`yt-dlp-ejs` wheel), Bundled Runtime(Deno), FFmpeg/FFprobe를 고정 버전·SHA-256으로 포함합니다. Managed Update는 고정 GitHub Release manifest를 하루 한 번 또는 수동으로 확인하고, 활성 다운로드가 없을 때만 Downloader와 Bundled Extractor를 함께 교체합니다.

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
- 셸을 만들지 않고 Bundled Python의 `python -m yt_dlp`, 고정 `--js-runtimes deno:<절대 경로>`, 고정 `PYTHONPATH`와 인수 목록으로 Downloader 프로세스를 실행하는 `DownloaderRunner`
- 최초 실행 시 검증된 Downloader와 Bundled Extractor wheel을 앱 데이터 디렉터리에 함께 초기화하고, 이후 원자 교체 시 이전 세트를 보존하는 `ToolManager`
- `managed-tools-v1` 고정 Release asset manifest를 하루 한 번 또는 수동으로 확인하고, 진행 중인 Download Queue가 있으면 적용을 지연하는 **Managed Update**
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

## 설치 파일 빌드

설치 파일(Windows NSIS/MSI, macOS DMG, Linux deb/AppImage)은 GitHub Actions에서 빌드합니다.

- 수동 빌드: Actions 탭 → **Build installers** → Run workflow
- 버전 릴리스: 태그를 푸시하면 설치 파일이 GitHub Release에 첨부됩니다.
- CI와 로컬 빌드는 `scripts/fetch-tools.sh <target-triple>`로 고정 버전의 Bundled Python, Downloader·Bundled Extractor wheel, Bundled Runtime을 내려받고 SHA-256을 검증합니다. Bundled Media Toolchain은 고정 FFmpeg source archive의 SHA-256을 검증한 뒤 자체 빌드합니다.

## 라이선스 및 제3자 소프트웨어

이 프로젝트는 [GPL-3.0-or-later](LICENSE)로 배포됩니다.

Managed Update manifest의 배포·갱신 절차는 [docs/MANAGED_UPDATE.md](docs/MANAGED_UPDATE.md)를 참고하세요.

이 애플리케이션은 [yt-dlp](https://github.com/yt-dlp/yt-dlp),
[yt-dlp-ejs](https://github.com/yt-dlp/ejs), CPython, Deno, FFmpeg 및 FFprobe를
설치 파일에 번들로 포함합니다. 고정한 버전, 원본 배포처 및 해당 라이선스 고지는
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)에 기록합니다. 설치 파일은
대상 플랫폼의 도구 라이선스 원문을 함께 포함합니다. FFmpeg source archive, 빌드 설정 및
릴리스 제공 방식은 [docs/FFMPEG_BUILD.md](docs/FFMPEG_BUILD.md)를 참고하세요.

Mytory Media Queue는 yt-dlp 프로젝트, YouTube 또는 Google과 제휴, 후원 또는 보증
관계가 아닙니다. 콘텐츠 다운로드 전에는 해당 콘텐츠의 이용약관, 저작권 및 기타
적용 법령에 따른 권한이 있는지 확인해야 합니다.

## 개인정보 및 보안 원칙

- 원격 분석, 자동 오류 전송을 하지 않습니다.
- Cookie Source는 Downloader 실행에만 전달하며 DB·UI 이벤트·진단 로그에 저장하지 않습니다.
- UI는 임의 yt-dlp 인수를 전달할 수 없습니다.
- Downloader 실행은 셸 문자열이 아닌 절대 경로와 고정 인수 목록을 사용합니다.
