# QSL 分卡助手 (QSL-CardHub)

[![Release][ico-actions]][link-actions]
[![Releases][ico-releases]][link-releases]
[![Stars][ico-stars]][link-stars]
[![License][ico-license]][link-license]

[ico-actions]: https://github.com/HerbertGao/qsl-cardhub/actions/workflows/release.yml/badge.svg
[ico-releases]: https://img.shields.io/github/v/release/HerbertGao/qsl-cardhub
[ico-stars]: https://img.shields.io/github/stars/HerbertGao/qsl-cardhub
[ico-license]: https://img.shields.io/github/license/HerbertGao/qsl-cardhub

[link-actions]: https://github.com/HerbertGao/qsl-cardhub/actions/workflows/release.yml
[link-releases]: https://github.com/HerbertGao/qsl-cardhub/releases
[link-stars]: https://github.com/HerbertGao/qsl-cardhub/stargazers
[link-license]: LICENSE

> 业余无线电 QSL 卡片管理工具

一款专为业余无线电爱好者（HAM）设计的 QSL 卡片管理工具，支持卡片录入、分发、打印标签、集成顺丰等功能。

## 功能特性

**卡片管理**
- 卡片录入与编辑，支持呼号、序列号等信息管理
- 支持 QRZ.com、QRZ.cn 等平台数据查询
- 本地数据库存储，支持云端同步

**辅助功能**
- 标签打印：支持 TSPL 热敏打印机（如 Deli DL-888C）
- 顺丰快递：面单打印与订单管理
- 多配置管理，支持导入导出

**其他**
- 跨平台支持：Windows、macOS
- 应用内自动更新（支持阿里云 CDN 加速，国内用户可获得更快的下载速度）

## 快速开始

### 下载安装

前往 [Releases](https://github.com/HerbertGao/qsl-cardhub/releases) 下载对应平台的安装包：

| 平台 | 文件 |
|------|------|
| macOS (Apple Silicon) | `qsl-cardhub-vX.X.X-macos-arm64.dmg` |
| macOS (Intel) | `qsl-cardhub-vX.X.X-macos-x64.dmg` |
| Windows x64 | `qsl-cardhub-vX.X.X-windows-x64-setup.exe` |
| Windows ARM64 | `qsl-cardhub-vX.X.X-windows-arm64-setup.exe` |

### 开发环境

```bash
# 前置要求：Rust 1.70+、Node.js 18+、pnpm

# 安装依赖
cargo build && cd web && pnpm install && cd ..

# 启动开发服务器
cargo tauri dev

# 生产构建
./scripts/build.sh  # macOS/Linux
.\scripts\build.ps1 # Windows
```

## 技术栈

| 类别 | 技术 |
|------|------|
| 后端 | Rust, Tauri 2 |
| 前端 | Vue 3, Element Plus, TypeScript |
| 打印 | TSPL 指令, Win32 API / CUPS |

## 许可证

[MIT License](LICENSE)

## 相关链接

- [项目主页](https://github.com/HerbertGao/qsl-cardhub)
- [问题反馈](https://github.com/HerbertGao/qsl-cardhub/issues)
