# Taxa v0.4.2

**[简体中文](#简体中文) | [English](#english)**

---

## 简体中文

功能更新版本：五项体验增强。

### ✨ 新特性

#### 1. 划词 AI 操作
在笔记中**选中一段文字**，选区下方会浮出操作菜单：**润色 / 翻译 / 解释 / 扩写**。AI 结果预览后可一键**替换原选区**或**插入到其后**。翻译方向自动判断（中文↔其他语言）。

#### 2. 图谱视图可导航
- **点击节点**直接打开对应笔记（与拖拽自动区分）
- **滚轮缩放**（以光标为中心，20%–300%），工具栏提供放大/缩小/重置
- **拖拽空白区域平移画布**，缩放较小时标签自动淡出

#### 3. 页签会话持久化 + 未保存标记
- 重启应用自动恢复上次的页签（含固定状态）和活动页签；笔记已删除的页签自动清理
- 有未保存修改时，活动页签显示**圆点标记**

#### 4. 每日笔记
目录树新增 📅 按钮：一键打开今天的笔记，首次点击自动在 `Daily` 文件夹创建；同一天反复进入同一篇，内容自然累积。

#### 5. 文件夹导出携带附件
导出文件夹时自动打包笔记中引用的图片（`attachments/`），导出目录自包含，换机器图片不丢失。

### 🔄 升级

- 应用内 设置 → 关于 → 检查更新，可直接升级到本版本。

---

## English

A feature release with five experience upgrades.

### ✨ Features

#### 1. Selection AI
**Select text** in a note to reveal a floating menu: **Polish / Translate / Explain / Expand**. Preview the result, then **replace** the selection or **insert after** it. Translation direction is detected automatically (Chinese ↔ other languages).

#### 2. Navigable graph view
- **Click a node** to open its note (distinguished from dragging automatically)
- **Wheel zoom** centered on the cursor (20%–300%) plus toolbar zoom in/out/reset
- **Drag empty space to pan**; labels fade out at low zoom levels

#### 3. Tab session persistence + unsaved indicator
- Tabs (with pin state) and the active tab are **restored on restart**; tabs whose notes no longer exist are dropped
- The active tab shows a **dot** while it has unsaved changes

#### 4. Daily notes
A 📅 button in the tree header opens today's note, creating it in the `Daily` folder on first use; same-day visits accumulate into the same note.

#### 5. Folder export includes attachments
Exporting a folder now packages the images its notes reference (`attachments/`), so the exported folder is self-contained.

### 🔄 Upgrading

- Open Settings → About → Check for Updates to upgrade in-app.

---

### 📦 安装 / Downloads

| 平台 / Platform | 资产 / Asset |
|---|---|
| Windows | `Taxa_0.4.2_x64-setup.exe` / `.msi` |
| macOS (Apple Silicon + Intel) | `Taxa_0.4.2_universal.dmg` |
| Linux | `.AppImage` / `.deb` / `.rpm` |
| MCP 服务器 / MCP server | `taxa-mcp-*` |
