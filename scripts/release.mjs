#!/usr/bin/env node
// Release helper: bump the version everywhere it lives, optionally writing
// a fresh RELEASE_NOTES.md skeleton.
//
//   node scripts/release.mjs 0.4.6
//   node scripts/release.mjs 0.4.6 --notes
import { readFileSync, writeFileSync } from 'node:fs';
import process from 'node:process';

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  console.error('usage: node scripts/release.mjs <x.y.z> [--notes]');
  process.exit(1);
}
const wantNotes = process.argv.includes('--notes');

function patch(path, fn) {
  const before = readFileSync(path, 'utf8');
  const after = fn(before);
  if (after === before) {
    console.error(`✗ ${path}: no change (anchor not found)`);
    process.exit(1);
  }
  writeFileSync(path, after);
  console.log(`✓ ${path}`);
}

// Workspace manifest (single source for crate versions).
patch('Cargo.toml', s =>
  s.replace(/^version = ".*"$/m, `version = "${version}"`));

patch('package.json', s => {
  const d = JSON.parse(s);
  d.version = version;
  return JSON.stringify(d, null, 2) + '\n';
});

patch('package-lock.json', s => {
  const d = JSON.parse(s);
  d.version = version;
  if (d.packages && d.packages['']) d.packages[''].version = version;
  return JSON.stringify(d, null, 2) + '\n';
});

patch('src-tauri/tauri.conf.json', s => {
  const d = JSON.parse(s);
  d.version = version;
  return JSON.stringify(d, null, 2) + '\n';
});

patch('src/components/settings/SettingsDialog.vue', s =>
  s.replace(/appVersion\.value = '[\d.]+';/, `appVersion.value = '${version}';`));

if (wantNotes) {
  writeFileSync('RELEASE_NOTES.md', `# Taxa v${version}

**[简体中文](#简体中文) | [English](#english)**

---

## 简体中文

### ✨ 新特性

### 🐛 修复

### 🔄 升级
- 应用内 设置 → 关于 → 检查更新

---

## English

### ✨ Features

### 🐛 Fixed

### 🔄 Upgrading
- Settings → About → Check for Updates

---

### 📦 安装 / Downloads

| 平台 / Platform | 资产 / Asset |
|---|---|
| Windows | \`Taxa_${version}_x64-setup.exe\` / \`.msi\` |
| macOS (Apple Silicon + Intel) | \`Taxa_${version}_universal.dmg\` |
| Linux | \`.AppImage\` / \`.deb\` / \`.rpm\` |
| MCP 服务器 / MCP server | \`taxa-mcp-*\` |
`);
  console.log('✓ RELEASE_NOTES.md (skeleton — fill in the details)');
}

console.log(`
Next steps:
  1. cargo check --workspace   # refresh Cargo.lock
  2. edit RELEASE_NOTES.md     # ${wantNotes ? 'fill the skeleton' : 'update for this release'}
  3. verify: cargo clippy/test, npm test, vue-tsc, vite build
  4. git add -A && git commit -m "chore(release): v${version}" && git push
  5. git tag v${version} && git push origin v${version}`);
