## 新增需求

### 需求：图像打印统一接入全局 TSPL 参数
系统必须在图像打印路径中统一接入全局 TSPL 参数（GAP、DIRECTION）。

#### 场景：系统打印机打印图像
- **当** 调用系统后端打印图像
- **那么** 后端必须使用全局配置中的 GAP 与 DIRECTION 生成 TSPL 指令
- **并且** 不得在后端内部使用固定硬编码覆盖该配置

#### 场景：配置非法时回退
- **当** 全局 TSPL 配置非法
- **那么** 系统必须回退到默认值 `GAP 2mm,0mm` 与 `DIRECTION 1,0`
- **并且** 输出可读日志用于诊断

## 修改需求

### 需求：TSPL 图像生成
TSPLGenerator 必须支持直接从灰度图像生成完整的 TSPL 打印指令，并支持外部传入 GAP 与 DIRECTION 参数。

#### 场景：从图像生成 TSPL
- **当** 调用图像 TSPL 生成接口并提供 GAP 与 DIRECTION
- **那么** 生成的 TSPL 头必须按传入参数生成 `GAP` 与 `DIRECTION`
- **并且** 指令仍然包含 SIZE、CLS、BITMAP、PRINT 等必要字段

## 移除需求

