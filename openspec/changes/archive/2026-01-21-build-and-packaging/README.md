# 构建和打包系统提案

本变更为 qsl-cardhub 项目添加完整的构建和打包系统。

## 文档

- [proposal.md](./proposal.md) - 提案说明
- [design.md](./design.md) - 设计文档
- [tasks.md](./tasks.md) - 实施任务清单
- [specs/build-and-packaging/spec.md](./specs/build-and-packaging/spec.md) - 功能规范

## 概述

### 目标

实现自动化构建和发布流程，支持 macOS 和 Windows 平台。

### 主要功能

1. **本地构建脚本** - 一键构建应用安装包
2. **GitHub Actions** - 自动化 CI/CD 流程
3. **版本管理** - 统一版本号管理
4. **Release 自动化** - 自动创建 GitHub Release

### 时间估算

15-22 小时（约 2-3 工作日）

## 验证

```bash
openspec-cn validate build-and-packaging --strict
```

验证结果：✅ 通过
