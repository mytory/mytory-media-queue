# AGENTS

`yt-dlp` 기반의 로컬 우선 Tauri 데스크톱 미디어 다운로드 도구입니다. 프런트엔드는 React/TypeScript/Vite, 백엔드는 Rust/Tauri입니다.

## 커밋 규칙

- 푸시는 절대 스스로 하지 않습니다.
- 태그를 임의로 생성·이동하지 않습니다. 사용자가 명시적으로 지시할 때만 처리합니다.

## 대화 및 수정 규칙

- 대화와 생각은 모두 한국어로 합니다.
- 코드·파일 생성·수정·삭제 전에는 수정 계획(무엇을, 왜, 어떻게)을 제시하고 사용자 승인을 받습니다.
- 사용자가 명시적으로 수정 의사를 밝히지 않으면 기본값은 설명만입니다.
- 관련 파일과 기존 패턴을 먼저 확인하고, 요청을 완전히 해결하는 최소 변경만 합니다.
- 추측하지 말고 코드·테스트·문서로 가정을 확인합니다.
- 변경 후 관련 검사를 실행하고, 변경 때문에 발생한 실패는 수정합니다.
- 반복 진단·수정 루프는 5회를 넘기지 않습니다. 더 필요하면 사용자에게 묻습니다.

## 공개 저장소와 번들 도구

- 다운로드한 sidecar, 설치 파일, 사용자 데이터, 비밀값은 Git에 추적하지 않습니다.
- Downloader 또는 Bundled Media Toolchain을 갱신할 때는 `latest` URL을 사용하지 않습니다. 고정 버전·SHA-256·`THIRD_PARTY_NOTICES.md`의 원본 배포처 및 라이선스 고지를 함께 갱신합니다.
- README, 패키지 메타데이터, 설치 파일의 라이선스 및 번들 도구 고지는 실제 배포물과 일치하게 유지합니다.

## 도메인 용어와 안전 원칙

용어와 제품 경계는 `CONTEXT.md`를 기준으로 합니다.

- **Downloader**: 메타데이터 조회와 다운로드를 수행하는 내장 `yt-dlp` 실행 파일입니다.
- **Bundled Runtime**: 앱과 함께 배포되고 자동 갱신하지 않는 내장 Deno 실행 환경입니다.
- **Bundled Media Toolchain**: Downloader가 영상·오디오 병합과 변환에 사용하는 내장 FFmpeg/FFprobe입니다.
- **Bundled Extractor**: YouTube JavaScript 추출을 지원하며 Managed Update로 Downloader와 함께 갱신되는 구성 요소입니다.
- **Managed Update**: 공식 안정 릴리스에서 Downloader와 Bundled Extractor를 내려받아 체크섬 검증 뒤 함께 교체합니다. 작업 중에는 교체하지 않으며, 실패 시 기존 구성 요소를 유지합니다.
- **Download Queue**: 여러 URL·재생목록 항목을 순서대로 처리하며 앱 재시작 후에도 재개되는 작업 목록입니다.
- **Downloader Simulator**: 실제 네트워크 없이 프로세스 경계를 통해 결정적 결과를 내는 테스트 전용 실행 파일입니다. TDD에서는 실제 `yt-dlp` 대신 사용합니다.

다음 원칙을 지킵니다.

- 원격 분석·자동 오류 전송을 추가하지 않습니다. 진단 정보는 로컬 전용입니다.
- Cookie Source는 Downloader 실행에만 전달하며 DB, UI 이벤트, 로그에 저장하거나 표시하지 않습니다.
- UI가 임의 `yt-dlp` 인수나 셸 명령을 전달하도록 만들지 않습니다. Downloader는 절대 경로와 고정 인수 목록으로 실행합니다.
- 지원 대상은 Windows x64, macOS Universal, Linux x64입니다.
- Download Queue의 기본 동시성은 3이며 사용자가 1~5 범위에서 설정합니다.
- 일시적 네트워크 오류만 최대 3회 자동 재시도합니다. 무한 재시도를 추가하지 않습니다.

## 테스트와 검사

TDD를 따릅니다. 먼저 실패하는 테스트를 작성하고, 최소 구현으로 통과시킨 뒤 리팩터링합니다. 실제 외부 사이트나 실제 `yt-dlp` 네트워크 다운로드에 의존하지 말고 Downloader Simulator를 사용합니다.

기본 검사는 다음과 같습니다.

```bash
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
```

변경 범위에 맞는 검사만 최소한으로 선택하되, Rust 변경에는 `cargo fmt --check`와 관련 테스트를 포함합니다.

## 출시

- 설치 파일은 GitHub Actions에서 빌드합니다.
- 로컬 패키지 빌드 전에는 필요하면 `bash scripts/fetch-tools.sh <target-triple>`로 sidecar를 준비합니다.
- 출시 준비는 다음 절차를 따릅니다.
  1. 현재 `package.json` 버전을 확인하고 변경 사항을 검토합니다.
  2. semver에 따라 다음 버전을 결정합니다.
     - patch: 버그 수정
     - minor: 기능 추가
     - major: 호환성을 깨는 변경
  3. 결정한 버전과 이유를 사용자에게 보고하고 승인을 받습니다.
  4. 승인 후 `package.json` 버전을 업데이트합니다.
  5. `README.md`를 해당 릴리스 내용에 맞게 업데이트합니다.
  6. 변경 사항을 커밋합니다.
  7. 사용자가 승인하면 `git tag -a vX.Y.Z -m "Release vX.Y.Z"`로 주석 태그를 생성합니다.
  8. 직접 푸시하지 않고, 사용자에게 `git push origin <branch> --tags` 등 필요한 브랜치·태그 푸시 명령을 안내합니다.
- 릴리스 태그·푸시는 사용자가 명시적으로 승인한 경우에만 실행합니다.
