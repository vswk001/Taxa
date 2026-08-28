**English** | [简体中文](README.zh-CN.md)

<div align="center">

<img src="src-tauri/icons/128x128.png" alt="Taxa" width="96" height="96">

# Taxa

**An AI-powered, local-first notebook for your desktop.**

Capture anything — Taxa auto-categorizes, enriches, and connects it. All data stays on your device.

[![Tauri](https://img.shields.io/badge/Tauri-v2-blue)](https://tauri.app)
[![Vue](https://img.shields.io/badge/Vue-3-42b883)](https://vuejs.org)
[![Rust](https://img.shields.io/badge/Rust-stable-orange)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Win%20%7C%20macOS%20%7C%20Linux-lightgrey)](#build-from-source)

</div>

---

Taxa is a note-taking app with a built-in AI assistant. Write or paste raw text and the AI decides where it belongs — creating a new note or appending to an existing one, with a clean title, folder, and tags. It can also polish and rewrite existing notes. Bring your own LLM (Claude, OpenAI, GLM, DeepSeek, MiniMax, Kimi, or any OpenAI-compatible API), with automatic fallback between providers.

It also ships an **MCP server**, so tools like Claude Code and Codex can search and read your notes as a local knowledge base.

## ✨ Features

- **Local-first storage** — notes live in Markdown files + a SQLite index on your machine. Nothing leaves your device except the LLM calls you make.
- **WYSIWYG editor** — Milkdown-based rich editor that stores plain Markdown.
- **AI assistant with two modes**
  - *Organize* — drop in any text; the AI categorizes, titles, tags, and files it (new note or append to a related one).
  - *Polish* — optimize and rewrite an existing note on instruction.
- **Multi-provider LLM** — configure several providers and let Taxa automatically fall back between them. Drag to set the fallback order. Streaming with reasoning shown.
- **MCP knowledge base** — expose your notes as a knowledge base over the Model Context Protocol, so AI coding assistants can search and read them as context.
- **Full-text search** — FTS5-backed (trigram, CJK-friendly) search across titles, tags, and content with highlighted snippets.
- **Trash** — soft delete with restore; nothing is ever lost by accident.
- **Global quick capture** — press `Ctrl+Alt+N` anywhere to jot something down; the AI files it (or it lands in your Inbox).
- **Image attachments** — paste/drop images straight into notes, stored locally with portable relative paths.
- **Backlinks panel** — see what links to the current note, what it links to, and create unresolved `[[targets]]` in one click.
- **Ask your notes (RAG)** — a Q&A mode that answers from your own notes with cited sources.
- **Note graph** — `[[wikilink]]` notes together and explore the graph.
- **9 languages** — 简体中文, 繁體中文, English, Español, العربية (RTL), Português, 日本語, Français, Deutsch.
- **Themes** — light, dark, or follow system.
- **Secure credentials** — API keys are stored in the OS keyring, not in plain text.

## 📥 Build from source

Pre-built binaries are on the [releases page](https://github.com/vswk001/Taxa/releases). To build from source:

**Prerequisites**

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- Tauri v2 [system dependencies](https://v2.tauri.app/start/prerequisites/) (WebView2 on Windows is preinstalled; Xcode Command Line Tools on macOS; `webkit2gtk` etc. on Linux)

**Steps**

```bash
git clone https://github.com/vswk001/Taxa.git
cd Taxa
npm install

npm run tauri dev      # run in development
npm run tauri build    # produce a production build
```

### 🔌 Enable the MCP knowledge base

The MCP server is a separate binary that exposes your notes to AI coding tools:

```bash
cargo build --release --bin taxa-mcp
```

Then point your client at it (Claude Code / Codex `mcpServers` config):

```jsonc
{ "mcpServers": { "taxa": { "command": "/absolute/path/to/taxa-mcp" } } }
```

See [`docs/mcp-server.md`](docs/mcp-server.md) for the full guide and the list of exposed tools.

## 🤖 Configuring an LLM provider

1. Open **Settings → LLM** (or the gear icon in the AI sidebar).
2. Add a provider: type, API URL, API key, and model.
3. Click **Test Connection**, then **Set as Default**.
4. Add more providers and drag to order them for automatic fallback.

Supported types: Claude (Anthropic), OpenAI, GLM (Zhipu), DeepSeek, MiniMax, Kimi (Moonshot), and any OpenAI-compatible endpoint.

## 🧱 Tech stack

| Layer              | Technology                               |
| ------------------ | ---------------------------------------- |
| Shell / desktop    | Tauri v2                                 |
| Frontend           | Vue 3 + TypeScript + Vite + Pinia        |
| Editor             | Milkdown (ProseMirror)                   |
| Backend            | Rust                                     |
| Storage            | SQLite (rusqlite, FTS5) + Markdown files |
| i18n               | vue-i18n                                 |
| AI transport       | streaming over Tauri events              |
| External interface | MCP over stdio                           |

## 📁 Project structure

```
Taxa/
├─ src/                  # Vue 3 frontend
│  ├─ components/        # editor, AI sidebar, tree, settings
│  ├─ stores/            # Pinia stores
│  └─ i18n/              # locales (9 languages)
├─ src-tauri/            # Tauri desktop app crate
│  └─ src/
│     ├─ notebook/       # notes CRUD, search, folders
│     ├─ ai/             # LLM providers, prompts, organizer engine
│     └─ storage/        # SQLite + Markdown storage
├─ mcp/                  # the MCP server binary (standalone crate)
└─ Cargo.toml            # workspace root
```

## 🌍 Internationalization

The UI is translated into 9 languages. Switch under **Settings → General → Language**; your choice persists across sessions, and Arabic switches the layout to RTL.

## 🤝 Contributing

Issues and pull requests are welcome. For non-trivial changes, please open an issue first to discuss what you'd like to change.

```bash
# frontend type-check + build
npx vue-tsc --noEmit
npx vite build

# backend check, lints, and tests
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same checks on every pull request (see `.github/workflows/ci.yml`).

## 📄 License

[MIT](LICENSE) © Taxa contributors
