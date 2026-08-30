# Tauri 2와 번들 sidecar를 사용한다

Windows·macOS·Linux용 설치형 앱은 React/TypeScript UI와 Rust 백엔드를 가진 Tauri 2로 만든다. Electron보다 설치 용량과 상주 자원 부담을 낮추면서, yt-dlp·Deno·FFmpeg 계열을 플랫폼별 sidecar로 번들하고 Rust에서 장시간 프로세스를 제어할 수 있기 때문이다.

공개 배포하는 sidecar는 고정 버전과 SHA-256으로 검증한다. `scripts/fetch-tools.sh`는 `latest` URL을 사용하지 않으며, `THIRD_PARTY_NOTICES.md`와 대상 플랫폼용 라이선스 원문을 설치 파일에 함께 포함한다.
