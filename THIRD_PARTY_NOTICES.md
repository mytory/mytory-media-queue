# Third-party notices

Mytory YT-DLP bundles the following command-line tools in its application installers.
The installation bundle also contains the FFmpeg license text for its target platform
under `resources/licenses/`.

## yt-dlp 2026.08.19

- Project: <https://github.com/yt-dlp/yt-dlp>
- Download source: <https://github.com/yt-dlp/yt-dlp/releases/tag/2026.08.19>
- License: The yt-dlp source project is released under the Unlicense. The exact
  executable release and its dependencies may carry additional notices; refer to the
  upstream release and its source distribution for them.

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
