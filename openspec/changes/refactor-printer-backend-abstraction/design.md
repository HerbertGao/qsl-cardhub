# 设计文档：打印机后端抽象层重构

## 上下文

QSL-CardHub 应用支持多种打印场景：
- QSL 卡片打印（基于模板系统）
- 地址标签打印（基于模板系统）
- 顺丰面单打印（基于 PDF 渲染）

同时支持两类打印机：
- 系统打印机（通过 CUPS/Windows API）
- PDF 测试打印机（保存为文件用于预览）

当前实现中，每个打印命令都需要手动判断打印机类型，导致代码重复和维护困难。

## 目标 / 非目标

### 目标
- 提供统一的图像打印接口，屏蔽后端差异
- 消除魔法字符串，使用常量定义打印机名称
- 减少代码重复，提高可维护性
- 支持未来扩展新的打印机后端

### 非目标
- 不改变模板系统的 `RenderResult` 打印流程（这是另一种抽象）
- 不改变现有 API 的外部接口

## 决策

### 1. 扩展 PrinterBackend trait 而非创建新 trait

**选择**：在现有 `PrinterBackend` trait 中添加新方法

**原因**：
- 保持单一 trait 定义打印机能力
- 避免 trait 膨胀和组合复杂性
- 所有后端都需要支持这些方法

**替代方案**：
- 创建 `ImagePrinter` trait 独立于 `PrinterBackend` — 拒绝，因为会增加 trait 组合复杂性

### 2. 使用 owns_printer() 而非打印机类型枚举

**选择**：让后端声明它是否拥有某个打印机

**原因**：
- 简单直观，后端自己知道它管理哪些打印机
- 支持动态打印机列表（如系统打印机随时可能增减）
- 不需要维护全局的打印机类型枚举

**替代方案**：
- 使用 `PrinterType` 枚举 — 拒绝，因为需要在每个打印命令中维护类型映射
- 使用打印机名称前缀约定 — 拒绝，因为系统打印机名称不可控

### 3. print_image() 接受 GrayImage 而非 RenderResult

**选择**：新方法接受 `GrayImage` 作为输入

**原因**：
- `GrayImage` 是更通用的图像格式，适用于各种来源（PDF 渲染、模板渲染等）
- `RenderResult` 包含混合模式信息（原生条码），不是所有场景都需要
- 保持职责分离：图像生成与图像打印分开

**替代方案**：
- 接受 `RenderResult` — 拒绝，因为会限制使用场景

### 4. 在 PrinterState 中添加路由方法

**选择**：在 `PrinterState` 中添加 `print_image_to_printer()` 方法

**原因**：
- `PrinterState` 已经持有所有后端的引用
- 集中管理打印机路由逻辑
- 调用方无需了解后端细节

**替代方案**：
- 创建独立的 `PrinterRouter` 结构 — 拒绝，因为增加不必要的复杂性

## 架构图

```
                          ┌──────────────────────┐
                          │    PrinterState      │
                          │                      │
                          │ print_image_to_      │
                          │ printer()            │
                          └──────────┬───────────┘
                                     │
                    owns_printer()?  │
                          ┌──────────┴───────────┐
                          │                      │
                    ┌─────▼─────┐          ┌─────▼─────┐
                    │ PdfBackend │          │ System    │
                    │            │          │ Backend   │
                    └─────┬─────┘          └─────┬─────┘
                          │                      │
                    ┌─────▼─────┐          ┌─────▼─────┐
                    │ 保存 PNG   │          │ 生成 TSPL  │
                    │ 到下载目录  │          │ 发送到打印机│
                    └───────────┘          └───────────┘
```

## 接口定义

```rust
/// 图像打印配置
pub struct ImagePrintConfig {
    pub width_mm: f32,
    pub height_mm: f32,
    pub dpi: u32,
}

/// 打印机后端 trait（扩展）
pub trait PrinterBackend: Send + Sync {
    // ... 现有方法 ...

    /// 检查此后端是否拥有指定的打印机
    fn owns_printer(&self, printer_name: &str) -> bool;

    /// 打印灰度图像
    fn print_image(
        &self,
        printer_name: &str,
        image: &GrayImage,
        config: &ImagePrintConfig,
    ) -> Result<PrintResult>;
}
```

## 风险 / 权衡

| 风险 | 缓解措施 |
|------|----------|
| 新方法增加 trait 复杂性 | 方法职责明确，文档清晰 |
| owns_printer() 可能有歧义 | 使用简单的名称匹配，PDF 后端只匹配固定名称 |
| 系统后端转 TSPL 增加延迟 | TSPL 生成很快，可接受 |

## 迁移计划

1. 添加新的 trait 方法和实现（向后兼容）
2. 更新 `sf_print_waybill` 使用新接口
3. 更新 `print_qsl` 和 `print_address` 使用常量
4. 验证所有打印功能正常

回滚：由于是增量变更，可以简单回退代码。

## 待决问题

- [ ] 是否需要为模板系统的 `RenderResult` 也创建统一的打印接口？（当前保持现状，因为模板系统有自己的 PDF/TSPL 分支逻辑）
