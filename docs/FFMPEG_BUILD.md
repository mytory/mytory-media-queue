# Bundled Media Toolchain 빌드와 소스 제공

Bundled Media Toolchain은 사용자 시스템의 FFmpeg를 사용하지 않는다. 설치 파일에는 이
저장소가 빌드한 FFmpeg와 FFprobe를 Windows x64, macOS Universal, Linux x64용 sidecar로
포함한다.

## 고정 입력

- FFmpeg: 7.1.1
- 공식 source archive: <https://ffmpeg.org/releases/ffmpeg-7.1.1.tar.xz>
- SHA-256: `733984395e0dbbe5c046abda2dc49a5544e7e0e1e2366bba849222ae9e3a03b1`
- 빌드 스크립트: [`../scripts/build-ffmpeg.sh`](../scripts/build-ffmpeg.sh)

스크립트는 source archive의 SHA-256을 검증한 뒤 GPL, version3, nonfree 옵션과 x86
assembler를 비활성화하고, 외부 codec library를 추가하지 않은 정적 FFmpeg/FFprobe를
빌드한다. 완료 후 `-version`의 `--disable-gpl`과 `-L`의 LGPL 표시를 검증한다.

macOS에서는 arm64와 x86_64를 별도로 빌드하고 `lipo`로 Universal binary를 만든다.
Windows에서는 GitHub Actions의 MSYS2 UCRT64 GCC 환경을 사용하고, Linux에서는 Ubuntu
22.04의 기본 C build toolchain을 사용한다. 실제 컴파일러 버전은 해당 release CI 로그와
`ffmpeg -version` 출력으로 확인한다.

## 릴리스 제공물

태그 기반 Application Release는 설치 파일과 함께 검증된
`ffmpeg-7.1.1.tar.xz`를 Release asset으로 첨부한다. 같은 tag의 공개 저장소에는 그
release를 만든 `scripts/build-ffmpeg.sh`, `scripts/prepare-ffmpeg-source-material.sh`,
`.github/workflows/build.yml`이 있다. 따라서 source archive, build configuration 및 CI
환경 정의를 함께 확인할 수 있다.

설치 파일에는 LGPL-2.1 전문을 Tauri resource-relative path
`binaries/licenses/ffmpeg-LGPL-2.1-or-later.txt`로 포함한다. 관련 고지는
[`../THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md)에 있다.

## 갱신 절차

1. 공식 FFmpeg release의 고정 버전, source URL, SHA-256을 확인한다. `latest` URL은 쓰지
   않는다.
2. `scripts/build-ffmpeg.sh`, `scripts/prepare-ffmpeg-source-material.sh`,
   `scripts/verify-ffmpeg-build.sh`, `THIRD_PARTY_NOTICES.md` 및 이 문서를 같은 변경으로
   갱신한다.
3. 지원 대상별 빌드를 실행하고, `scripts/verify-ffmpeg-build.sh`와 source material 준비를
   통과시킨다.
4. 태그 CI가 source archive를 Application Release asset으로 첨부하는지 확인한다. 태그나
   push는 별도 승인 없이는 수행하지 않는다.
