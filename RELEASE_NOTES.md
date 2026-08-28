# Taxa v0.4.1

**[简体中文](#简体中文) | [English](#english)**

---

## 简体中文

v0.4.0 的补丁版本，修复一个影响较大的界面问题。

### 🐛 修复

- **笔记树右键菜单所有功能失效**：点击菜单项（重命名、移动、删除、导入、导出等）没有任何反应。根因是"点击菜单外部自动关闭"的监听器在捕获阶段先行把菜单从界面移除，导致后续的点击事件无目标可达。现在点击菜单内部会正常执行操作，点击外部和 Esc 关闭行为不变。

### 🔄 升级建议

- **从 v0.3.x 或更早版本升级**：直接下载下方安装包覆盖安装。
- **已安装 v0.4.0**：在应用内 设置 → 关于 → 检查更新，可直接升级到本版本。

---

## English

A patch release for v0.4.0 fixing a significant UI regression.

### 🐛 Fixed

- **All note-tree context menu actions were dead**: clicking menu items (rename, move, delete, import, export, etc.) did nothing. Root cause: the "close the menu when clicking outside" listener ran in the capture phase and removed the menu from the DOM before the item's click event could reach it. Clicks inside the menu now execute normally; clicking outside and pressing Escape still close it.

### 🔄 Upgrading

- **From v0.3.x or earlier**: download an installer below and install over your existing setup.
- **On v0.4.0**: open Settings → About → Check for Updates to upgrade in-app.

---

### 📦 安装 / Downloads

| 平台 / Platform | 资产 / Asset |
|---|---|
| Windows | `Taxa_0.4.1_x64-setup.exe` / `.msi` |
| macOS (Apple Silicon + Intel) | `Taxa_0.4.1_universal.dmg` |
| Linux | `.AppImage` / `.deb` / `.rpm` |
| MCP 服务器 / MCP server | `taxa-mcp-*` |
