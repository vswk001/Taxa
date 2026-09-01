# Taxa v0.4.3

**[简体中文](#简体中文) | [English](#english)**

---

## 简体中文

功能更新版本：标签管理、命令面板、备份/恢复、托盘常驻，以及更新前的版本说明预览。

### ✨ 新特性

#### 1. 查看版本更新内容
应用内检测到新版本时，新增**「查看更新内容」**按钮：弹窗展示该版本的完整更新说明（新特性、修复清单），看完可直接在弹窗里安装。本版起升级前即可知道改了什么。

#### 2. 标签管理
- 标签输入**自动补全**：输入时从已有标签中匹配建议，点击即用
- **标签面板**（目录树 🏷 按钮）：按使用量排序、显示计数，点击标签直接跳转该标签的搜索结果
- **全局重命名**：重命名一个标签会应用到所有使用它的笔记；与已有标签同名时自动合并

#### 3. 命令面板（Ctrl+P）
任何界面（包括编辑器内）按 `Ctrl+P` 呼出：统一入口搜索笔记、文件夹，或执行动作（新建笔记、每日笔记、搜索、图谱、标签、回收站、设置），支持键盘上下选择 + 回车执行。

#### 4. 备份与恢复（设置 → 通用 → 数据）
- **立即备份**：将数据库一致性快照 + 全部笔记、附件、回收站打包为一个 zip
- **从备份恢复**：选择备份文件，确认后自动重启完成替换；恢复在数据库打开前执行，避免运行中替换导致损坏

#### 5. 托盘常驻
- 关闭主窗口默认**最小化到托盘**（设置中可关闭该行为），托盘菜单提供 打开 / 快捷捕获 / 退出
- 常驻期间**全局快捷捕获热键持续可用**——"随时随地捕获"从此名副其实
- 点击托盘图标恢复主窗口

### 🔧 质量改进
- 新增自动化回归测试套件（21 项），覆盖历史上所有已修复的关键缺陷（数据丢失、菜单失效、页签关闭等），每次提交自动运行

### 🔄 升级
- 应用内 设置 → 关于 → 检查更新，升级前可先「查看更新内容」

---

## English

A feature release: tag management, command palette, backup/restore, tray residency, and release-notes preview before updating.

### ✨ Features

#### 1. View release notes on update
When an update is detected, a **What's New** button opens the full release notes (features and fixes) in-app, with an Install button right in the dialog — from this version on you always know what changed before upgrading.

#### 2. Tag management
- **Autocomplete** while typing tags, sourced from existing tags
- **Tag panel** (tree header 🏷): usage-sorted with counts; clicking a tag jumps to a scoped search
- **Global rename**: renames the tag across every note that uses it, merging into an existing tag on name collision

#### 3. Command palette (Ctrl+P)
Summon from anywhere (editor included): search notes and folders or run actions (new note, daily note, search, graph, tags, trash, settings) with full keyboard navigation.

#### 4. Backup & restore (Settings → General → Data)
- **Back Up Now**: one zip containing a consistent database snapshot plus all notes, attachments, and the trash
- **Restore from Backup**: pick a zip, confirm, and the app relaunches to complete the swap — restores are applied before the database opens, avoiding live-file corruption

#### 5. Tray residency
- Closing the main window hides to tray by default (toggleable); the tray menu offers Open / Quick Capture / Quit
- The **global quick-capture hotkey keeps working** while the app sits in the tray
- Click the tray icon to restore the window

### 🔧 Quality
- Added an automated regression suite (21 tests) covering every historically fixed critical bug (data loss, dead menus, tab closing, …), run on every push

### 🔄 Upgrading
- Settings → About → Check for Updates — and preview What's New first

---

### 📦 安装 / Downloads

| 平台 / Platform | 资产 / Asset |
|---|---|
| Windows | `Taxa_0.4.3_x64-setup.exe` / `.msi` |
| macOS (Apple Silicon + Intel) | `Taxa_0.4.3_universal.dmg` |
| Linux | `.AppImage` / `.deb` / `.rpm` |
| MCP 服务器 / MCP server | `taxa-mcp-*` |
