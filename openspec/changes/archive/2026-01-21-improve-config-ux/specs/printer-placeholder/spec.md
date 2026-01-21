# 规范：打印机名称选择框提示文本

**功能**: 打印机名称选择框提示文本
**模块**: web/ConfigView
**状态**: 草稿

---

## 新增需求

无

---

## 修改需求

### 需求：新建配置对话框的打印机选择框必须显示 placeholder

`ConfigView` 组件新建配置对话框中的打印机名称选择框（`el-select` 组件）必须添加 `placeholder` 属性，值为 `"请选择打印机"`。该 placeholder 必须在选择框为空且未聚焦时显示。当打印机列表为空时，placeholder 必须仍然可见，提示用户需要选择打印机。当用户选择了打印机后，placeholder 必须被隐藏。该修改对应 ConfigView.vue 第166行的 `el-select` 组件。

---

#### 场景：打开新建配置对话框时显示 placeholder

**前置条件**:
- 用户在 ConfigView 页面
- 打印机列表至少包含一个打印机
- 用户点击"新建"按钮

**操作**:
1. 新建配置对话框打开
2. 查看打印机名称选择框

**预期结果**:
- 选择框显示 placeholder 文本 `"请选择打印机"`（如果未自动选择打印机）
- 或显示第一个可用打印机名称（如果自动选择了打印机）

---

#### 场景：打印机列表为空时显示 placeholder

**前置条件**:
- 用户在 ConfigView 页面
- `availablePrinters` 数组为空
- 用户点击"新建"按钮

**操作**:
1. 新建配置对话框打开
2. 查看打印机名称选择框

**预期结果**:
- 选择框显示 placeholder 文本 `"请选择打印机"`
- 下拉列表为空（无可选项）
- placeholder 提示用户需要配置或刷新打印机

---

#### 场景：用户选择打印机后 placeholder 消失

**前置条件**:
- 新建配置对话框已打开
- 打印机选择框显示 placeholder

**操作**:
1. 用户点击打印机选择框
2. 从下拉列表选择一个打印机

**预期结果**:
- 选择框显示所选打印机的名称
- placeholder 文本被隐藏
- `newConfigForm.printerName` 被更新为所选打印机名称

---

### 需求：配置详情页的打印机选择框必须显示 placeholder

`ConfigView` 组件配置详情区域中的打印机名称选择框（`el-select` 组件）必须添加 `placeholder` 属性，值为 `"请选择打印机"`。该 placeholder 必须在选择框为空且未聚焦时显示。当用户选择了打印机后，placeholder 必须被隐藏。该修改对应 ConfigView.vue 第103行的 `el-select` 组件。placeholder 的行为必须与新建配置对话框中的选择框保持一致。

---

#### 场景：查看配置详情时显示 placeholder

**前置条件**:
- 用户在 ConfigView 页面
- 配置列表中至少有一个配置
- 用户选择了一个配置

**操作**:
1. 配置详情在右侧面板显示
2. 查看打印机名称选择框

**预期结果**:
- 选择框显示当前配置的打印机名称（通常不为空）
- 如果打印机名称为空或 `null`，显示 placeholder `"请选择打印机"`

---

#### 场景：编辑配置时 placeholder 行为正常

**前置条件**:
- 用户在配置详情页面
- 当前配置的打印机名称已设置

**操作**:
1. 用户点击打印机选择框
2. 从下拉列表选择另一个打印机
3. 点击"保存修改"按钮

**预期结果**:
- 选择框更新为新选择的打印机名称
- placeholder 保持隐藏
- 配置保存成功

---

#### 场景：打印机名称被清空后显示 placeholder

**前置条件**:
- 用户在配置详情页面
- 当前配置的打印机名称已设置

**操作**:
1. 用户通过某种方式清空打印机名称（例如选择了空值）
2. 选择框失去焦点

**预期结果**:
- 选择框显示 placeholder `"请选择打印机"`
- `selectedConfig.printer.name` 为空或 `null`

---

## 移除需求

无

---

## 重命名需求

无

---

## 参考

- [Element Plus Select 组件文档](https://element-plus.org/zh-CN/component/select.html)
- ConfigView 源码：`web/src/views/ConfigView.vue`（第103行和第166行）
