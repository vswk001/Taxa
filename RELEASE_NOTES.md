# Taxa v0.4.4

**[简体中文](#简体中文) | [English](#english)**

---

## 简体中文

稳定性补丁：修复"打开即消失"的启动问题，并杜绝其根源。

### 🐛 修复

#### 启动无窗口（表现为闪退）
根因是一个连锁反应：应用内更新完成后自动重启时，可能留下一个**无窗口的后台残留进程**；之后再次更新时，该残留进程锁定程序文件导致**安装中途失败**，并损坏 WebView 数据目录——此后每次打开，进程启动但窗口永不出现，看起来就是闪退，且每次点击都会积累更多残留进程。

本版加入**单实例保护**（Tauri 官方插件）：再次启动时不再产生竞争进程，而是唤起并聚焦已有窗口。残留进程从此无法累积，更新时也不会再被旧进程锁死文件。

#### 💡 如果你的应用现在正打不开（自救步骤）
1. 打开任务管理器，结束所有 `taxa.exe` 进程（可能有好几个）
2. 删除文件夹 `%LOCALAPPDATA%\com.taxa.desktop`（只是浏览器缓存，笔记数据不受影响）
3. 重新打开 Taxa——应恢复正常
4. 建议立即在 设置 → 关于 → 检查更新，升级到本版本以获得根治

### 🔄 升级
- 应用内 设置 → 关于 → 检查更新（可先「查看更新内容」）

---

## English

A stability patch: fixes the "launches then vanishes" startup failure and eliminates its root cause.

### 🐛 Fixed

#### App opens with no window (perceived as an instant crash)
Root cause was a chain reaction: after an in-app update, the automatic relaunch could leave a **headless background process** behind; a later update then **failed midway** because that zombie locked the executable, corrupting the WebView data folder — after which every launch started a process whose window never appeared (looking like a crash), with each attempt piling up more zombies.

This version adds a **single-instance guard** (official Tauri plugin): launching again now shows and focuses the existing window instead of spawning a competing process. Zombies can no longer accumulate, and updates can no longer be blocked by a stale process holding the executable.

#### 💡 If your app currently won't open (recovery steps)
1. Open Task Manager and end every `taxa.exe` process (there may be several)
2. Delete the folder `%LOCALAPPDATA%\com.taxa.desktop` (browser cache only — your notes are unaffected)
3. Launch Taxa again — it should come back
4. Then update to this version (Settings → About → Check for Updates) for the permanent fix

### 🔄 Upgrading
- Settings → About → Check for Updates (preview What's New first if you like)

---

### 📦 安装 / Downloads

| 平台 / Platform | 资产 / Asset |
|---|---|
| Windows | `Taxa_0.4.4_x64-setup.exe` / `.msi` |
| macOS (Apple Silicon + Intel) | `Taxa_0.4.4_universal.dmg` |
| Linux | `.AppImage` / `.deb` / `.rpm` |
| MCP 服务器 / MCP server | `taxa-mcp-*` |
