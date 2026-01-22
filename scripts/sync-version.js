#!/usr/bin/env node

/**
 * 同步 Cargo.toml 的版本号到 web/package.json
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

// 读取 package.json
const packageJsonPath = path.join(__dirname, '../web/package.json');
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));

// 检查是否需要更新
if (packageJson.version === version) {
  console.log('✅ web/package.json 版本号已是最新');
  process.exit(0);
}

// 更新版本号
const oldVersion = packageJson.version;
packageJson.version = version;

// 写回 package.json（保持格式）
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n', 'utf-8');

console.log(`✅ 已更新 web/package.json 版本号: ${oldVersion} → ${version}`);
