# 代码签名说明

> 结论：Taxis 的安装包**当前未签名**。可信代码签名都要付费，本项目选择不弄；唯一免费途径见下方 SignPath。

## 为什么不签名

要消除 macOS Gatekeeper / Windows SmartScreen 的"未知开发者"警告，需要**受信任的代码签名证书**，这些都要钱：

| 平台 | 证书 | 费用 |
|------|------|------|
| macOS | Apple Developer ID（必须公证） | $99/年，且创建证书需 Mac |
| Windows | OV/EV 证书 | $200+/年（2023 年后私钥强制存硬件，CI 难用） |
| Windows | Azure Trusted Signing | ~$10/月 |

> 自签名证书**对分发无效**——终端用户照样看到"未知发布者"，只适合本地测试，所以也不算解决方案。

本项目当前**选择不付费签名**，安装包保持未签名。

## 对用户的影响

首次打开未签名的安装包会有警告，绕过方式：
- **Windows**：SmartScreen 提示"已保护你的电脑" → 点"更多信息" → "仍要运行"。
- **macOS**：双击 .dmg 后，.app 拖到 Applications，右键 → 打开 → 确认。或终端 `xattr -dr com.apple.quarantine /Applications/Taxis.app`。

对早期开源项目这通常可接受。

## 唯一免费途径：SignPath Foundation（仅 Windows）

[SignPath Foundation](https://signpath.org/foundation) 为**开源项目提供免费的 Windows 代码签名**（基于 SignPath 的云端签名）。macOS 无免费途径。

申请条件（大致）：GitHub 上的开源项目、有一定活跃度、非商业。审核通过后：

1. 在 signpath.org 注册项目，配置**签名策略**（哪些产物可签）。
2. 在仓库安装 **SignPath GitHub App**。
3. CI 里用 `signpath/github-action` 把构建产物（`.exe`、`.msi`）提交给 SignPath 签名。

**如果你想走这条路**：先去 signpath.org/foundation 申请，通过后把 SignPath 项目信息（API token、project slug）给我，我把签名步骤接进 `.github/workflows/release.yml`。没申请前接了也不可用，所以默认不接。

## 以后改主意

若将来愿意付费，最快的是：
- **macOS**：买 Apple Developer Program，按 [Tauri macOS 签名文档](https://v2.tauri.app/distribute/sign/macos/) 创建 Developer ID 证书，把 6 个 `APPLE_*` secret 填进仓库即可（工作流结构已支持，届时加回 env 引用）。
- **Windows**：参考 [Tauri Windows 签名文档](https://v2.tauri.app/distribute/sign/windows/)。
