[English](RELEASING.md) | **简体中文**

# 发版

跨平台构建由 **Release** 工作流（`.github/workflows/release.yml`）完成：在 Windows、macOS、Linux 三种 runner 上原生构建，并把安装包上传到 GitHub release。

## 为什么用 CI，而不是本地构建

Tauri 应用捆绑各平台原生 webview（Windows 的 WebView2、macOS 的 WKWebView、Linux 的 webkit2gtk）。每个平台的二进制必须在该平台上构建——从单台机器可靠地交叉编译并不现实。GitHub Actions 在三平台并行构建。

## 触发一次发布

推送一个 tag：

```bash
git tag v0.1.0
git push origin v0.1.0
```

……或进入 **Actions → Release → Run workflow**，填入 tag 名。

工作流会创建一个名为 `Taxis <tag>` 的**草稿 release**，并附上各平台安装包：

| 平台 | 产物 |
|------|------|
| Windows | `.msi` / `.exe` 安装包 + `taxis-mcp-windows-x86_64.exe` |
| macOS | `.dmg`（universal，Apple Silicon + Intel）+ `taxis-mcp-macos-universal` |
| Linux | `.deb` / `.AppImage` + `taxis-mcp-linux-x86_64` |

到 GitHub 上查看草稿 release，按需编辑说明，然后点击**发布**。

## 代码签名（可选）

当前构建未签名。Windows 和 macOS 会提示「未知开发者」。要消除提示，请配置签名：

- **macOS：** Apple Developer ID 证书 + 公证（将 `APPLE_CERTIFICATE`、`APPLE_ID`、`APPLE_PASSWORD` 等 secrets 传给 `tauri-action`）。
- **Windows：** 代码签名证书（`WINDOWS_CERTIFICATE`、`WINDOWS_CERTIFICATE_PASSWORD`）。

密钥名详见 [`tauri-action` 文档](https://github.com/tauri-apps/tauri-action)。

## 本地预检

打 tag 之前，可以在本机构建自己的平台以做一次验证：

```bash
npm install
npm run tauri build          # 产出当前平台的安装包
cd src-tauri && cargo build --release --bin taxis-mcp
```
