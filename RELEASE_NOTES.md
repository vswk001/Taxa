# Taxa v0.5.0

**[简体中文](#简体中文) | [English](#english)**

---

## 简体中文

重要功能版本：编辑器链接自动补全、AI 笔记库体检、自动备份、快捷键面板、版本发布工具链。

### ✨ 新特性

#### 1. `[[` 链接自动补全
在编辑器中输入 `[[` 即弹出匹配的笔记标题列表：↑↓ 选择，Enter/Tab 成链，Esc 关闭。建立知识关联从未如此顺手——图谱和反向链接的内容质量随之提升。

#### 2. AI 笔记库体检（含撤销）
AI 侧栏新增 🩺 按钮：分析全库后给出**移动 / 合并 / 补标签**建议，逐条确认应用。所有已应用操作自动记录（🕘 按钮查看历史）且**可撤销**——移动还原位置、标签还原原值、合并从回收站恢复源笔记并裁掉目标中追加的段落。

#### 3. 每周自动备份
设置 → 通用 → 数据中开启后，每周自动将数据库快照 + 笔记 + 附件打包为 zip 存入数据目录 `backups/`（保留最近 4 份）。手动备份仍然可用。

#### 4. 窗口状态记忆
记住窗口大小、位置和是否最大化；下次启动在窗口显示前完成定位（无闪烁）。

#### 5. 快捷键说明面板
任意界面按 `?`（或命令面板中搜索"快捷键"）查看全部快捷键，按全局/编辑器/页签分组。

#### 6. README 界面截图
项目首页新增主界面、笔记图谱、快捷捕获、命令面板四张截图。

### 🔧 工程改进
- **版本发布工具链**：`scripts/release.mjs` 一条命令升级全部版本号并生成双语发布说明骨架
- **latest.json 自动修复**：发版工作流新增补丁任务，自动填充更新说明为空的 manifest（修复 v0.4.5 的 API 竞态问题）

### 🔄 升级
- 应用内 设置 → 关于 → 检查更新（可先「查看更新内容」）

---

## English

A major feature release: editor link autocomplete, AI library health check with undo, automatic backups, shortcuts panel, and release tooling.

### ✨ Features

#### 1. `[[` link autocomplete
Typing `[[` in the editor shows matching note titles: arrow keys navigate, Enter/Tab completes the link, Escape dismisses. Building your knowledge graph is now frictionless — the quality of backlinks and the graph view improves naturally.

#### 2. AI library health check (with undo)
A 🩺 button in the AI sidebar analyzes your library and suggests **moves / merges / tag additions**, confirmed individually. All applied operations are logged (🕘 shows history) and **undoable** — move restores the folder, tag restores the tags, merge recovers the source note from the trash and trims the appended block from the target.

#### 3. Weekly auto-backup
Enable in Settings → General → Data. Zips a database snapshot plus notes and attachments into `backups/` weekly, keeping the last 4. Manual backup remains available.

#### 4. Window state persistence
Remembers window size, position, and maximized state; restores before showing on next launch (no flash).

#### 5. Keyboard shortcuts panel
Press `?` anywhere (or search "shortcuts" in the command palette) for a grouped overview of every shortcut.

#### 6. README screenshots
The project page now shows screenshots of the main UI, note graph, quick capture, and command palette.

### 🔧 Engineering
- **Release tooling**: `scripts/release.mjs` bumps all version anchors and scaffolds bilingual release notes in one command
- **latest.json auto-repair**: the release workflow now patches an empty-notes manifest automatically (fixes the v0.4.5 API race)

### 🔄 Upgrading
- Settings → About → Check for Updates (preview What's New first)

---

### 📦 安装 / Downloads

| 平台 / Platform | 资产 / Asset |
|---|---|
| Windows | `Taxa_0.5.0_x64-setup.exe` / `.msi` |
| macOS (Apple Silicon + Intel) | `Taxa_0.5.0_universal.dmg` |
| Linux | `.AppImage` / `.deb` / `.rpm` |
| MCP 服务器 / MCP server | `taxa-mcp-*` |
