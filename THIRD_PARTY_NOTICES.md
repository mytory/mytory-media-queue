# Third-party notices

Mytory Media Queue bundles the following command-line tools and Python packages in its
application installers. The installation bundle contains the corresponding license texts
at the Tauri resource-relative path `binaries/licenses/`.

## Downloader: yt-dlp 2026.8.19

- Project: <https://github.com/yt-dlp/yt-dlp>
- Download source: <https://files.pythonhosted.org/packages/69/b2/8cd1613f56eed7ceb64fbd4df3f1c01246bfb098e6f398228bafda22b80b/yt_dlp-2026.8.19-py3-none-any.whl>
- SHA-256: `1d57897e94c6665a0a6f9bc54b34e584284e32c034ffab3a7df25d8f7b24eedf`
- License: Unlicense. The exact license text is included as `yt-dlp-LICENSE`.

## Bundled Extractor: yt-dlp-ejs 0.8.0

- Project: <https://github.com/yt-dlp/ejs>
- Download source: <https://github.com/yt-dlp/ejs/releases/download/0.8.0/yt_dlp_ejs-0.8.0-py3-none-any.whl>
- SHA-256: `79300e5fca7f937a1eeede11f0456862c1b41107ce1d726871e0207424f4bdb4`
- License expression: Unlicense AND MIT AND ISC. The yt-dlp-ejs Unlicense text is included
  as `yt-dlp-ejs-LICENSE`. Its bundled meriyah 6.1.4 (ISC; Copyright (c) 2019 and later,
  KFlash and others) and astring 1.9.0 (MIT; Copyright (c) 2015, David Bonnet) notices
  are preserved verbatim in `yt-dlp-ejs-BUNDLED-NOTICES.txt`.

## Bundled Python: CPython 3.13.15 / python-build-standalone 20260825

- Binary provider: <https://github.com/astral-sh/python-build-standalone/releases/tag/20260825>
- Upstream project: <https://www.python.org/>
- License: python-build-standalone is MPL-2.0 and CPython is PSF-2.0. Exact texts are
  included as `python-build-standalone-LICENSE` and `python-LICENSE`. License and notice
  files delivered in each target's Python tree remain at `binaries/python/<target>/`; the
  exact target-specific file list and SHA-256 values are included as
  `python-<target>-NOTICE-INVENTORY.txt`.

## Bundled Runtime: Deno 2.3.1

- Project: <https://github.com/denoland/deno>
- Download source: <https://github.com/denoland/deno/releases/tag/v2.3.1>
- License: MIT. The exact text is included as `deno-LICENSE`.

## Bundled Media Toolchain: FFmpeg and FFprobe 7.1.1

- Upstream project and source archive: <https://ffmpeg.org/releases/ffmpeg-7.1.1.tar.xz>
- Source archive SHA-256: `733984395e0dbbe5c046abda2dc49a5544e7e0e1e2366bba849222ae9e3a03b1`
- Build script: [`scripts/build-ffmpeg.sh`](scripts/build-ffmpeg.sh) at this Application
  Release's Git tag. It builds without GPL, nonfree, or external codec options.
- License: LGPL-2.1-or-later. The exact LGPL-2.1 text is installed as
  `binaries/licenses/ffmpeg-LGPL-2.1-or-later.txt`.
- Corresponding Source: the same versioned Application Release includes the verified
  `ffmpeg-7.1.1.tar.xz` source archive as a Release asset. The release tag's source tree
  contains the exact build script and CI configuration used for that asset.

The Bundled Media Toolchain is built from this source by this project for Windows x64,
macOS Universal, and Linux x64. This project itself is GPL-3.0-or-later; see
[`LICENSE`](LICENSE).

## Application dependencies

JavaScript, Rust, and Tauri dependencies are resolved from `package-lock.json` and
`src-tauri/Cargo.lock`. Their respective license terms continue to apply.
