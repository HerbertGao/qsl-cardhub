#!/usr/bin/env node

/**
 * 同步 Cargo.toml 的版本号到 web/package.json 和 tauri.conf.json
 */

const fs = require('fs');
const path = require('path');

// 读取 Cargo.toml
const cargoTomlPath = path.join(__dirname, '../Cargo.toml');
const cargoToml = fs.readFileSync(cargoTomlPath, 'utf-8');

// 提取版本号
const versionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
if (!versionMatch) {
  console.error('❌ 无法从 Cargo.toml 中提取版本号');
  process.exit(1);
}

const version = versionMatch[1];
console.log(`📦 检测到版本号: ${version}`);

let updated = false;

// 需要同步版本号的 package.json 列表
const packageJsonFiles = [
  '../web/package.json',
  '../web_query_service/package.json',
];

for (const relPath of packageJsonFiles) {
  const pkgPath = path.join(__dirname, relPath);
  const label = relPath.replace('../', '');

  if (!fs.existsSync(pkgPath)) {
    console.log(`⏭️  ${label} 不存在，跳过`);
    continue;
  }

  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));

  if (pkg.version !== version) {
    const oldVersion = pkg.version;
    pkg.version = version;
    fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf-8');
    console.log(`✅ 已更新 ${label} 版本号: ${oldVersion} → ${version}`);
    updated = true;
  } else {
    console.log(`✅ ${label} 版本号已是最新`);
  }
}

// 更新 tauri.conf.json
const tauriConfPath = path.join(__dirname, '../tauri.conf.json');
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf-8'));

if (tauriConf.version !== version) {
  const oldVersion = tauriConf.version || '未设置';
  tauriConf.version = version;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf-8');
  console.log(`✅ 已更新 tauri.conf.json 版本号: ${oldVersion} → ${version}`);
  updated = true;
} else {
  console.log('✅ tauri.conf.json 版本号已是最新');
}

if (!updated) {
  console.log('📝 所有版本号已同步');
}
