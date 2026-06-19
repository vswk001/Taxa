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

## Signing

The workflow signs **conditionally**: it signs automatically once the relevant
GitHub Secrets exist, and produces an unsigned build otherwise (no failure).

- **macOS** is already wired — add the 6 `APPLE_*` secrets (Apple Developer ID
  certificate + notarization) and the next release signs + notarizes.
- **Windows** depends on the certificate source (Azure Trusted Signing,
  SignPath, or a PFX/USB-token cert); each is configured differently.

Full steps (obtaining certs, the exact secret names, verifying the signature)
are in [`docs/code-signing.zh-CN.md`](docs/code-signing.zh-CN.md).

## Local verification

You can build your own platform locally to sanity-check before tagging:

```bash
npm install
npm run tauri build          # produces your platform's installer
cargo build --release --bin taxis-mcp
```
