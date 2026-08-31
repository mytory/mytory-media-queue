# Mytory Media Queue

공개 소스 저장소에서 개발하는, yt-dlp 기반의 데스크톱 미디어 다운로드 도구이다.

## 언어

**Downloader**:
미디어 메타데이터를 조회하고 다운로드 작업을 수행하는 내장 yt-dlp 실행 파일.
_피할 것_: 다운로더 엔진, yt-dlp 프로그램

**Bundled Runtime**:
애플리케이션 릴리스와 함께 배포되며 별도로 자동 갱신하지 않는 내장 Deno 실행 환경.
_피할 것_: 자동 업데이트 런타임

**Bundled Media Toolchain**:
Downloader가 고화질 영상과 오디오를 병합하거나 변환할 수 있도록 애플리케이션 릴리스와 함께 제공하는 FFmpeg 및 FFprobe 실행 파일.
_피할 것_: 선택 설치 코덱, 시스템 FFmpeg

**Bundled Tool Manifest**:
Bundled Python, Downloader, Bundled Extractor, Bundled Runtime 및 Bundled Media Toolchain의 고정 버전, 원본 배포처, SHA-256 및 라이선스 고지를 함께 관리하는 배포 규칙. 설치 파일에는 대상 플랫폼의 도구 라이선스 원문을 포함한다.
_피할 것_: `latest` URL, 검증 없는 바이너리, 고지 없는 재배포

**Bundled Extractor**:
YouTube의 JavaScript 추출을 지원하는 내장 `yt-dlp-ejs` 구성 요소. Downloader와 호환되는 버전 쌍으로 Managed Update에서 함께 갱신되며, Bundled Runtime으로 실행된다.
_피할 것_: Deno, 독립 업데이트 구성 요소

**Managed Update**:
앱이 공식 안정 릴리스에서 Downloader와 Bundled Extractor를 내려받아 체크섬 검증 후 한 번에 교체하는 과정. 앱 시작 시 하루 1회 확인하며, 진행 중인 작업이 없을 때만 교체한다. 실패하면 기존 구성 요소를 유지하고 사용자는 수동으로 확인할 수 있다.
_피할 것_: nightly 업데이트, 전체 앱 업데이트, 강제 업데이트

**Supported Platform**:
초기 릴리스를 제공하는 Windows x64, macOS Universal(Apple Silicon 및 Intel), Linux x64 환경.
_피할 것_: 모든 데스크톱 환경, ARM 지원

**Unsigned Distribution**:
서명·공증 없이 배포하는 플랫폼별 설치 파일. 설치 안내는 Windows SmartScreen과 macOS Gatekeeper의 최초 실행 절차를 포함하며, Application Release는 자동 교체하지 않는다.
_피할 것_: 공개 앱 스토어 배포, 서명된 릴리스

**Download Queue**:
여러 URL과 재생목록에서 확장된 항목을 순서대로 처리하는 작업 목록. 실행 중에도 새 URL을 추가할 수 있으며, 앱 종료로 중단된 작업은 다음 실행 때 재개할 수 있다.
_피할 것_: 단일 다운로드, 일괄 실행

**Quit Confirmation**:
진행 중인 작업이 있을 때 앱을 종료하려는 사용자에게 표시하는 확인 절차. 앱은 트레이로 숨겨지지 않는다.
_피할 것_: 트레이 최소화, 무확인 종료

**Download Destination**:
URL을 대기열에 추가할 때 선택하는 파일 저장 위치. 기본값은 운영체제의 다운로드 폴더이며 사용자가 변경할 수 있다.
_피할 것_: 고정 저장 경로, 전역 출력 폴더

**Output Preset**:
사용자가 Download Queue에 추가할 때 고르는 출력 형식 정책. 기본값은 MP4 호환 우선이며, 최고 품질 영상·원본 품질 오디오·MP3 320kbps를 추가로 제공한다.
_피할 것_: 직접 yt-dlp 인자, 사용자 정의 포맷 문자열

**Supplementary Asset**:
콘텐츠와 함께 저장하는 별도 파일. 썸네일은 기본 저장하며, 자막은 사용자가 켠 경우 한국어·영어 순으로 제공 자막만 `.vtt` 파일로 저장한다. 영상 내부 삽입과 자동 생성 자막은 제공하지 않는다.
_피할 것_: 내장 자막, 자동 생성 자막, 설명 파일

**Output Naming**:
단일 항목은 제목으로 저장하고 동명 파일 충돌 시 영상 ID를 붙이는 규칙. 재생목록 항목은 선택한 Download Destination 아래 재생목록 제목 폴더에 저장한다.
_피할 것_: 덮어쓰기, 평면 재생목록 저장

**Download Concurrency**:
Download Queue에서 동시에 실행할 수 있는 작업 수. 기본값은 3이며 사용자가 1~5 사이에서 설정한다.
_피할 것_: 항목별 병렬 수, 무제한 병렬 다운로드

**Download History**:
대기·진행·실패 작업과 최근 90일의 완료 작업에 대한 로컬 이력. 사용자는 이력만 지울 수 있고, 실제 내려받은 파일은 삭제하지 않는다.
_피할 것_: 파일 관리, 영구 감사 로그

**Failure Recovery**:
일시적 네트워크 오류를 최대 3회 자동 재시도하고, 그 밖의 오류는 사용자의 항목별 재시도·취소로 해결하는 정책. 각 작업은 정제된 진단 로그를 제공한다.
_피할 것_: 무한 재시도, 원시 쿠키 로그

**Cookie Source**:
로그인 또는 접근 제한 콘텐츠를 위해 사용자가 선택하는 브라우저 프로필 또는 `cookies.txt` 파일. 앱은 이를 Downloader 실행 시에만 전달하고 저장·표시·로그하지 않는다.
_피할 것_: 저장된 계정, 내보낸 쿠키

**Local-Only Diagnostics**:
Download History와 Failure Recovery의 정보는 기기에만 저장하고, 원격 분석·자동 오류 전송은 하지 않는 정책. 사용자는 정제된 로그를 직접 복사해 공유할 수 있다.
_피할 것_: telemetry, 자동 오류 보고

**Downloader Simulator**:
Downloader와 같은 프로세스 경계를 통해 결정적 진행·성공·실패·중단 결과를 내보내는 테스트 전용 실행 파일. TDD에서는 실제 yt-dlp 대신 이것을 사용한다.
_피할 것_: 실제 네트워크 다운로드, 내부 협력자 mock
