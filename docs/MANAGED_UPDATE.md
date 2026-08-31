# Managed Update manifest

앱은 `https://github.com/mytory/mytory-media-queue/releases/download/managed-tools-v1/manifest.json`만 Managed Update manifest trust root로 사용한다. GitHub의 `latest` URL이나 임의 URL은 사용하지 않는다.

`docs/managed-tools-manifest.json`을 다음의 공개 Release asset으로 업로드한다.

- Release tag: `managed-tools-v1`
- Asset name: `manifest.json`

이 Release는 저장소 관리자만 변경할 수 있으며, manifest에는 공식 `files.pythonhosted.org` 또는 `github.com/yt-dlp` HTTPS 배포물과 SHA-256만 넣는다. Downloader와 Bundled Extractor는 반드시 하나의 호환 세트로 함께 변경한다.

## 갱신 절차

1. 두 공식 배포물의 고정 버전·URL·SHA-256과 라이선스를 검증한다.
2. `scripts/fetch-tools.sh`와 `THIRD_PARTY_NOTICES.md`를 앱 릴리스에 맞게 갱신한다.
3. `docs/managed-tools-manifest.json`의 version, URL, SHA-256을 함께 갱신한다.
4. `managed-tools-v1` Release asset `manifest.json`을 교체한다. 앱은 다음 하루 1회 확인 또는 사용자의 수동 확인에서 이를 읽는다.
5. 로컬 HTTP fixture 테스트와 전체 Rust 테스트를 실행한다.

검증·다운로드·스테이징 중 하나라도 실패하면 기존 `current` 세트는 유지된다. 실행 중인 Download Queue가 있으면 적용하지 않고 지연한다.
