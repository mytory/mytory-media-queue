# Third-party notices

Mytory Media Queue bundles the following command-line tools and Python packages in its
application installers. The installation bundle also contains the corresponding license
texts under `resources/licenses/`.

## Downloader: yt-dlp 2026.8.19

- Project: <https://github.com/yt-dlp/yt-dlp>
- Download source: <https://files.pythonhosted.org/packages/69/b2/8cd1613f56eed7ceb64fbd4df3f1c01246bfb098e6f398228bafda22b80b/yt_dlp-2026.8.19-py3-none-any.whl>
- SHA-256: `1d57897e94c6665a0a6f9bc54b34e584284e32c034ffab3a7df25d8f7b24eedf`
- License: Unlicense. The exact license text is included as `yt-dlp-LICENSE`.

## Bundled Extractor: yt-dlp-ejs 0.8.0

- Project: <https://github.com/yt-dlp/ejs>
- Download source: <https://github.com/yt-dlp/ejs/releases/download/0.8.0/yt_dlp_ejs-0.8.0-py3-none-any.whl>
- SHA-256: `79300e5fca7f937a1eeede11f0456862c1b41107ce1d726871e0207424f4bdb4`
- License: Unlicense. The exact license text is included as `yt-dlp-ejs-LICENSE`.

## Bundled Python: CPython 3.13.15 / python-build-standalone 20260825

- Binary provider: <https://github.com/astral-sh/python-build-standalone/releases/tag/20260825>
- Upstream project: <https://www.python.org/>
- License: python-build-standalone is MPL-2.0 and CPython is PSF-2.0. Exact texts are
  included as `python-build-standalone-LICENSE` and `python-LICENSE`.

## Bundled Runtime: Deno 2.3.1

- Project: <https://github.com/denoland/deno>
- Download source: <https://github.com/denoland/deno/releases/tag/v2.3.1>
- License: MIT. The exact text is included as `deno-LICENSE`.

## FFmpeg and FFprobe b6.1.1

- Binary provider: <https://github.com/eugeneware/ffmpeg-static/releases/tag/b6.1.1>
- Upstream project: <https://ffmpeg.org/>
- License: The exact license text supplied by the binary provider is packaged in each
  installer under `resources/licenses/ffmpeg-*-LICENSE`.

The Windows x64 and Linux x64 binaries used by this project are GPL-3.0-or-later.
The macOS binaries used by this project are LGPL-2.1-or-later according to their
provider-supplied license texts. This project itself is GPL-3.0-or-later; see
[`LICENSE`](LICENSE).

## Application dependencies

JavaScript, Rust, and Tauri dependencies are resolved from `package-lock.json` and
`src-tauri/Cargo.lock`. Their respective license terms continue to apply.
