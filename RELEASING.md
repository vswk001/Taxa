**English** | [简体中文](RELEASING.zh-CN.md)

# Releasing

Cross-platform builds are produced by the **Release** workflow
(`.github/workflows/release.yml`), which builds natively on Windows,
macOS, and Linux runners and uploads the installers to a GitHub release.

## Why CI, not local builds

Tauri apps bundle a native webview (WebView2 on Windows, WKWebView on
macOS, webkit2gtk on Linux). Each platform's binary must be built on that
OS — reliable cross-compilation from a single machine isn't practical.
GitHub Actions runs all three in parallel.

## Trigger a release

Either push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

…or go to **Actions → Release → Run workflow** and enter a tag name.

The workflow creates a **draft** release named `Taxis <tag>` with the
platform installers attached:

| Platform | Assets                                                                  |
| -------- | ----------------------------------------------------------------------- |
| Windows  | `.msi` / `.exe` installer + `taxis-mcp-windows-x86_64.exe`              |
| macOS    | `.dmg` (universal, Apple Silicon + Intel) + `taxis-mcp-macos-universal` |
| Linux    | `.deb` / `.AppImage` + `taxis-mcp-linux-x86_64`                         |

Review the draft release on GitHub, edit the notes if you like, then
**publish** it.

## Signing (optional)

Builds are currently unsigned. Windows and macOS will warn "unidentified
developer." To silence that, configure signing:

- **macOS:** an Apple Developer ID certificate + notarization
  (`APPLE_CERTIFICATE`, `APPLE_ID`, `APPLE_PASSWORD` secrets passed to
  `tauri-action`).
- **Windows:** a code-signing certificate
  (`WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`).

See the [`tauri-action` docs](https://github.com/tauri-apps/tauri-action)
for the secret names.

## Local verification

You can build your own platform locally to sanity-check before tagging:

```bash
npm install
npm run tauri build          # produces your platform's installer
cd src-tauri && cargo build --release --bin taxis-mcp
```
