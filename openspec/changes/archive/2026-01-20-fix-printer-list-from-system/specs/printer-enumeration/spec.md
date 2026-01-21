# 规范：打印机枚举

## 概述

定义打印机列表获取的行为规范，确保用户界面显示系统真实打印机和 PDF 预览打印机。

## 新增需求

### 需求：系统打印机枚举

打印机列表**必须**包含系统中实际安装的打印机。

#### 场景：macOS/Linux 系统枚举打印机

**前提**：系统通过 CUPS 安装了打印机 "HP_LaserJet" 和 "Canon_Printer"

**当**：用户打开配置管理页面

**那么**：打印机下拉列表显示：
- HP_LaserJet
- Canon_Printer
- PDF 测试打印机

#### 场景：Windows 系统枚举打印机

**前提**：系统安装了打印机 "Microsoft Print to PDF" 和 "Deli DL-888C"

**当**：用户打开配置管理页面

**那么**：打印机下拉列表显示：
- Microsoft Print to PDF
- Deli DL-888C
- PDF 测试打印机

#### 场景：系统没有安装打印机

**前提**：系统没有安装任何打印机

**当**：用户打开配置管理页面

**那么**：打印机下拉列表显示：
- PDF 测试打印机

### 需求：PdfBackend 提供预览打印机

PdfBackend **必须**在打印机列表中提供 "PDF 测试打印机" 选项。

#### 场景：PdfBackend 返回 PDF 打印机

**当**：调用 `PdfBackend::list_printers()`

**那么**：返回 `["PDF 测试打印机"]`

### 需求：PdfBackend 预览功能

PdfBackend **必须**能够将 TSPL 数据渲染为 PNG 文件。

#### 场景：PdfBackend 生成预览

**前提**：有一个 `PdfBackend` 实例

**当**：调用 `send_raw("PDF 测试打印机", tspl_data)`

**那么**：TSPL 数据被渲染为 PNG 文件

**并且**：文件保存到 Downloads 目录

## 移除需求

### 需求：MockBackend 虚拟打印机

MockBackend **必须**被移除，**禁止**在打印机列表中显示 Mock 打印机。

#### 场景：不再支持 Mock 打印

**当**：用户查看打印机列表

**那么**：不会看到 "Mock Printer" 或 "Mock Printer 2"

## 修改需求

### 需求：PrinterManager 打印机聚合

PrinterManager **必须**聚合系统打印机和 PDF 预览打印机，**禁止**包含 Mock 打印机。

#### 场景：聚合后的打印机列表

**前提**：
- CupsBackend 返回 ["HP_LaserJet"]
- PdfBackend 返回 ["PDF 测试打印机"]

**当**：调用 `PrinterManager::list_printers()`

**那么**：返回 ["HP_LaserJet", "PDF 测试打印机"]

**并且**：列表中不包含 "Mock Printer"
