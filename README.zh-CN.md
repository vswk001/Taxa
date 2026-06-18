[English](README.md) | **简体中文**

<div align="center">

<img src="src-tauri/icons/128x128.png" alt="Taxis" width="96" height="96">

# Taxis

**AI 驱动的本地优先笔记本。**

随手记录任何内容——Taxis 自动归类、完善并关联。所有数据都留在你的设备上。

[![Tauri](https://img.shields.io/badge/Tauri-v2-blue)](https://tauri.app)
[![Vue](https://img.shields.io/badge/Vue-3-42b883)](https://vuejs.org)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![平台](https://img.shields.io/badge/platform-Win%20%7C%20macOS%20%7C%20Linux-lightgrey)](#从源码构建)

</div>

---

Taxis 是一款内置 AI 助手的笔记应用。写下或粘贴任意文字，AI 会判断它属于哪里——新建一篇笔记，或追加到已有相关笔记，并给出整洁的标题、目录和标签。它还能按指令润色、改写已有笔记。自带 LLM（Claude、OpenAI、GLM、DeepSeek、MiniMax、Kimi，或任何 OpenAI 兼容接口），并支持多 provider 自动 fallback。

它还附带一个**只读 MCP 服务器**，让 Claude Code、Codex 等工具能搜索并读取你的笔记，作为本地知识库。

## ✨ 功能特性

- **本地优先存储**——笔记以 Markdown 文件 + SQLite 索引保存在你的设备上。除你主动发起的 LLM 调用外，无任何数据外传。
- **所见即所得编辑器**——基于 Milkdown 的富文本编辑器，底层存储纯 Markdown。
- **双模式 AI 助手**
  - *整理*——丢入任意文字，AI 自动归类、拟标题、打标签、归档（新建或追加到相关笔记）。
  - *润色*——按指令优化、改写已有笔记。
- **多 provider LLM**——可配置多个 provider，自动在它们之间 fallback；拖拽即可调整备用顺序。支持流式输出与思考过程展示。
- **全文搜索**——基于 FTS5，覆盖标题、标签、内容。
- **笔记图谱**——用 `[[双链]]` 关联笔记，可视化探索。
- **9 种语言**——简体中文、繁體中文、English、Español、العربية（RTL）、Português、日本語、Français、Deutsch。
- **主题**——亮色、暗色、跟随系统。
- **凭证安全**——API Key 存入系统钥匙串，不明文落盘。

## 📥 从源码构建

目前尚未发布预编译二进制。可自行构建运行：

**前置要求**

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install)（stable 工具链）
- Tauri v2 [系统依赖](https://v2.tauri.app/start/prerequisites/)（Windows 自带 WebView2；macOS 需 Xcode 命令行工具；Linux 需 `webkit2gtk` 等）

**步骤**

```bash
git clone https://github.com/vswk001/Taxis.git
cd Taxis
npm install

npm run tauri dev      # 开发模式运行
npm run tauri build    # 构建生产版本
```

### 🔌 启用 MCP 知识库

MCP 服务器是一个独立二进制，把你的笔记暴露给 AI 编程工具：

```bash
cd src-tauri
cargo build --release --bin taxis-mcp
```

然后在客户端（Claude Code / Codex 的 `mcpServers` 配置）中指向它：

```jsonc
{ "mcpServers": { "taxis": { "command": "/taxis-mcp 的绝对路径" } } }
```

完整指南与可用工具列表见 [`docs/mcp-server.md`](docs/mcp-server.md)。

## 🤖 配置 LLM provider

1. 打开 **设置 → LLM**（或 AI 侧边栏的齿轮图标）。
2. 添加 provider：类型、API 地址、API Key、模型。
3. 点击 **测试连接**，再 **设为默认**。
4. 添加更多 provider 并拖拽排序，用于自动 fallback。

支持的类型：Claude (Anthropic)、OpenAI、智谱 GLM、DeepSeek、MiniMax、Kimi (Moonshot)，以及任何 OpenAI 兼容接口。

## 🧱 技术栈

| 层 | 技术 |
|----|------|
| 桌面外壳 | Tauri v2 |
| 前端 | Vue 3 + TypeScript + Vite + Pinia |
| 编辑器 | Milkdown (ProseMirror) |
| 后端 | Rust |
| 存储 | SQLite (rusqlite, FTS5) + Markdown 文件 |
| 国际化 | vue-i18n |
| AI 传输 | 基于 Tauri 事件的流式 |
| 外部接口 | 基于 stdio 的 MCP |

## 📁 项目结构

```
Taxis/
├─ src/                  # Vue 3 前端
│  ├─ components/        # 编辑器、AI 侧边栏、目录树、设置
│  ├─ stores/            # Pinia stores
│  └─ i18n/              # 9 种语言文案
└─ src-tauri/
   ├─ src/
   │  ├─ notebook/       # 笔记增删改查、搜索、目录
   │  ├─ ai/             # LLM provider、prompt、整理引擎
   │  ├─ storage/        # SQLite + Markdown 存储
   │  └─ bin/mcp/        # 只读 MCP 服务器二进制
   └─ tauri.conf.json
```

## 🌍 国际化

界面已翻译为 9 种语言。在 **设置 → 通用 → 语言** 中切换；选择会跨会话保存，阿拉伯语会自动切换为 RTL 布局。

## 🤝 参与贡献

欢迎提 Issue 和 Pull Request。较大改动请先开 Issue 讨论你想修改的内容。

```bash
# 前端类型检查
npx vue-tsc --noEmit

# 后端检查
cd src-tauri && cargo check --lib
```

## 📄 许可证

[MIT](LICENSE) © Taxis contributors
