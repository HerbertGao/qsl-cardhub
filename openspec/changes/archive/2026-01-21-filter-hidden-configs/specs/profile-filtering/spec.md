# 规范：配置文件过滤

**功能**: 配置文件过滤
**模块**: config/profile_manager
**状态**: 草稿

---

## 新增需求

### 需求：ProfileManager.get_all 必须过滤隐藏配置文件

`ProfileManager::get_all()` 方法在扫描 profiles 目录时，必须自动过滤掉文件名以 `.` 开头的配置文件。文件名判断基于 `file_name()` 返回的 OsStr，转换为字符串后检查首字符。仅过滤文件名，不涉及路径的其他部分。文件扩展名检查（`.toml`）在隐藏文件过滤之前进行。过滤逻辑应用于所有通过 `read_dir()` 获取的条目。

---

#### 场景：加载正常配置文件

**前置条件**:
- profiles 目录存在
- 目录中存在文件 `my-profile.toml`（非隐藏）
- 文件内容为有效的 Profile TOML

**操作**:
1. 调用 `ProfileManager::get_all()`

**预期结果**:
- 返回的 Vec 包含从 `my-profile.toml` 加载的 Profile
- 该 Profile 的字段正确反映文件内容

---

#### 场景：忽略隐藏配置文件

**前置条件**:
- profiles 目录存在
- 目录中存在文件 `.example.toml`（隐藏）
- 目录中存在文件 `my-profile.toml`（正常）
- 两个文件内容均为有效的 Profile TOML

**操作**:
1. 调用 `ProfileManager::get_all()`

**预期结果**:
- 返回的 Vec **不包含** 从 `.example.toml` 加载的 Profile
- 返回的 Vec **包含** 从 `my-profile.toml` 加载的 Profile
- Vec 的长度为 1

---

#### 场景：空目录

**前置条件**:
- profiles 目录存在
- 目录中无任何 `.toml` 文件

**操作**:
1. 调用 `ProfileManager::get_all()`

**预期结果**:
- 返回空的 Vec
- 不抛出错误

---

#### 场景：仅包含隐藏文件

**前置条件**:
- profiles 目录存在
- 目录中仅存在 `.example.toml`（隐藏）
- 无其他 `.toml` 文件

**操作**:
1. 调用 `ProfileManager::get_all()`

**预期结果**:
- 返回空的 Vec（因为隐藏文件被过滤）
- 不抛出错误

---

#### 场景：混合正常和隐藏文件

**前置条件**:
- profiles 目录存在
- 目录中存在以下文件：
  - `.example.toml`（隐藏）
  - `.backup.toml`（隐藏）
  - `profile-1.toml`（正常）
  - `profile-2.toml`（正常）
- 所有文件内容均为有效的 Profile TOML

**操作**:
1. 调用 `ProfileManager::get_all()`

**预期结果**:
- 返回的 Vec **不包含** `.example.toml` 和 `.backup.toml`
- 返回的 Vec **包含** `profile-1.toml` 和 `profile-2.toml`
- Vec 的长度为 2

---

## 修改需求

### 需求：示例配置文件必须为隐藏文件

位于 `config/profiles/` 目录的示例配置文件必须从 `example.toml` 重命名为 `.example.toml`，以避免出现在用户的配置列表中。文件内容保持不变。文件路径变更：`config/profiles/example.toml` → `config/profiles/.example.toml`。使用 `git mv` 保留文件历史。

---

#### 场景：示例文件可作为模板使用

**前置条件**:
- `.example.toml` 存在于 `config/profiles/` 目录
- 文件包含有效的示例 Profile 配置

**操作**:
1. 用户手动复制 `.example.toml` 为 `my-profile.toml`
2. 用户编辑 `my-profile.toml` 中的字段
3. 调用 `ProfileManager::get_all()`

**预期结果**:
- 用户的 `my-profile.toml` 出现在配置列表中
- `.example.toml` 不出现在配置列表中
- 用户可以基于示例创建自己的配置

---

## 移除需求

无

---

## 重命名需求

无

---

## 参考

- Unix/Linux 隐藏文件约定：以 `.` 开头的文件为隐藏文件
- Rust `std::fs::read_dir` 文档：https://doc.rust-lang.org/std/fs/fn.read_dir.html
- `ProfileManager` 源码：`src/config/profile_manager.rs`
