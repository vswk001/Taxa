# 代码签名指南

> 本文档讲如何让 Taxis 的安装包通过系统签名校验，消除 macOS Gatekeeper / Windows SmartScreen 的"未知开发者"警告。

CI 工作流已做成**条件式签名**：只要把对应密钥填进 GitHub Secrets，下次发版就自动签名；没填则照常出未签名包（不影响构建）。当前已接好 **macOS**，**Windows** 见下方按你选的证书方案配置。

---

## 一、macOS（已接好，填 6 个 secret 即生效）

### 前提
- 拥有 **Apple Developer Program** 会员（$99/年）。

### 1. 创建 Developer ID Application 证书（必须在 Mac 上）
1. 在 Mac 上打开"钥匙串访问" → 菜单 **钥匙串访问 → 证书助理 → 从证书颁发机构请求证书**，生成一个 CSR（保存到磁盘）。
2. 登录 https://developer.apple.com → Certificates, IDs & Profiles → 新建证书，类型选 **Developer ID Application**，上传刚才的 CSR，下载 `.cer`。
3. 双击 `.cer` 导入钥匙串，找到"Developer ID Application: 你的名字 (TeamID)"。
4. 在钥匙串里右键该证书 → **导出**，格式选 **个人信息交换 (.p12)**，设一个导出密码 → 得到 `cert.p12`。

### 2. 申请 App 专用密码（用于公证）
1. 登录 https://appleid.apple.com → 登录-安全 → **App 专用密码** → 生成一个，记下密码。
2. TeamID 见 Apple Developer 账户页（10 位字母数字）。

### 3. 在 GitHub 仓库添加 6 个 Secrets
仓库 → Settings → Secrets and variables → Actions → New repository secret：

| Secret 名 | 值 |
|-----------|----|
| `APPLE_CERTIFICATE` | `cert.p12` 的 base64（mac 上 `base64 -i cert.p12` 的输出） |
| `APPLE_CERTIFICATE_PASSWORD` | 上面导出 .p12 时设的密码 |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: 你的名字 (TeamID)`（可留空让 tauri 自动识别） |
| `APPLE_ID` | 你的 Apple ID 邮箱 |
| `APPLE_PASSWORD` | 上一步的 App 专用密码 |
| `APPLE_TEAM_ID` | 你的 Team ID |

填完后，下次打 tag 发版，macOS 的 `.dmg` / `.app` 会自动签名 + 公证。

---

## 二、Windows（三选一，告诉我选哪个我来接）

2023 年 6 月后，新 OV/EV 证书私钥**必须存在硬件**（HSM/USB token），不能再下载 `.pfx`，所以"把 .pfx 塞进 secret"这条路对新证书走不通了。CI 友好三条路：

### 方案 A：Azure Trusted Signing（推荐，CI 最省心）
- 约 $10/月，云端 HSM，无需 USB token。
- 需要 Azure 账号 + 创建 **Trusted Signing** 资源 + 一个 App Registration（拿到 tenant/client/endpoint/account）。
- CI 里用 `azure/trusted-signing-action` 对构建产物（`.exe`、`.msi`）签名。
- 适合有公司主体或愿意用 Azure 的开发者。

### 方案 B：SignPath Foundation（开源免费）
- Taxis 是开源项目，可向 **SignPath Foundation** 申请免费 Windows 代码签名。
- 审核（需 GitHub 组织、一定活跃度）通过后，在仓库装 SignPath GitHub App，配置签名策略。
- CI 里用 SignPath 的 action 提交签名请求。
- 最省钱，但需要等审核。

### 方案 C：OV/EV 证书 + USB token / 远程签名
- 约 $200+/年（DigiCert、Sectigo 等）。
- 私钥在 USB token，CI 里要用远程签名服务（如 SSL.com eSigner、DigiCert ONE）配合 `signtool` 的 `/csp` / `/kc` 参数。
- 配置最繁琐，且硬件 token 方案在 CI 里体验差。

> 自签名证书**不能用于分发**——终端用户照样看到"未知发布者"，只适合你自己本地测试。本指南默认针对真证书。

**告诉我你选哪个方案（A / B / C，或"我已有 .pfx"），我就在工作流里接好对应的签名步骤。**

---

## 三、（可选）Tauri 自动更新签名

如果你以后启用 Tauri 的应用内自动更新，需要一对更新签名密钥（这和上面的"发布者证书"是两回事，用于校验更新包完整性）：

```bash
# 本地生成
npm run tauri -- signer generate -w ~/.tauri/taxis.key
```

把生成的私钥存为 GitHub Secret `TAURI_PRIVATE_KEY`、密码存 `TAURI_KEY_PASSWORD`，tauri 就会给更新包加签名（`sig`）。当前未启用，按需再加。

---

## 校验签名是否生效

发版后下载安装包：
- **macOS**：`spctl -a -vv /path/to/Taxis.app` 应显示 `accepted`、`source=Notarized Developer ID`。
- **Windows**：右键 `.exe` → 属性 → 数字签名，能看到发布者名称即已签名。
