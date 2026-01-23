# 设计文档：自动化数据库迁移机制

## 架构概览

```
migrations/
├── 001_init.sql
├── 002_add_cards.sql
├── 003_add_sf_express.sql
└── 004_xxx.sql (未来添加)
        │
        ▼ 编译时嵌入
┌─────────────────────────────────────┐
│         include_dir! 宏              │
│  (将目录内容嵌入到二进制文件中)        │
└─────────────────────────────────────┘
        │
        ▼ 运行时
┌─────────────────────────────────────┐
│         MigrationManager            │
│  - 解析文件名获取版本号              │
│  - 按版本号排序                      │
│  - 与当前 user_version 比较         │
│  - 执行未运行的迁移                  │
└─────────────────────────────────────┘
```

## 核心实现

### 1. 迁移文件嵌入

使用 `include_dir` crate 在编译时将 `migrations/` 目录嵌入到二进制文件中：

```rust
use include_dir::{include_dir, Dir};

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");
```

### 2. 迁移文件解析

```rust
/// 迁移脚本信息
struct Migration {
    version: i32,
    name: String,
    sql: String,
}

/// 从嵌入的目录中解析迁移脚本
fn parse_migrations() -> Result<Vec<Migration>, AppError> {
    let mut migrations = Vec::new();

    for file in MIGRATIONS_DIR.files() {
        let filename = file.path().file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AppError::Other("无效的文件名".to_string()))?;

        // 解析版本号：001_init.sql -> 1
        if let Some(version) = parse_version(filename) {
            let sql = file.contents_utf8()
                .ok_or_else(|| AppError::Other("无法读取迁移文件".to_string()))?;

            migrations.push(Migration {
                version,
                name: filename.to_string(),
                sql: sql.to_string(),
            });
        }
    }

    // 按版本号排序
    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

/// 从文件名解析版本号
fn parse_version(filename: &str) -> Option<i32> {
    // 匹配格式：001_xxx.sql, 002_xxx.sql, ...
    if !filename.ends_with(".sql") {
        return None;
    }

    let parts: Vec<&str> = filename.split('_').collect();
    if parts.is_empty() {
        return None;
    }

    parts[0].parse::<i32>().ok()
}
```

### 3. 迁移执行逻辑

```rust
/// 执行数据库迁移
fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    let migrations = parse_migrations()?;
    let current_version = get_db_version(conn)?;

    for migration in migrations {
        if migration.version > current_version {
            log::info!("📦 执行迁移: {} (v{})", migration.name, migration.version);

            conn.execute_batch(&migration.sql).map_err(|e| {
                AppError::Other(format!(
                    "迁移 {} 执行失败: {}",
                    migration.name, e
                ))
            })?;

            set_db_version(conn, migration.version)?;
            log::info!("✅ 迁移完成: {}", migration.name);
        }
    }

    Ok(())
}
```

## 文件命名规范

| 格式 | 示例 | 说明 |
|------|------|------|
| `{NNN}_{description}.sql` | `001_init.sql` | 版本号使用 3 位数字 |
| 版本号 | 001, 002, 003... | 必须连续，不能跳跃 |
| 描述 | `add_cards`, `add_sf_express` | 使用下划线分隔的小写字母 |

## 错误处理

1. **文件名格式错误**：跳过不符合格式的文件，记录警告日志
2. **SQL 执行失败**：中止迁移，保持数据库在上一个稳定版本
3. **版本号冲突**：编译时检测（通过测试）

## 向后兼容

- 现有的迁移文件无需修改
- 数据库 `user_version` 机制保持不变
- 已执行的迁移不会重复执行

## 依赖添加

```toml
# Cargo.toml
[dependencies]
include_dir = "0.7"
```

## 测试策略

1. **单元测试**：测试文件名解析逻辑
2. **集成测试**：测试完整迁移流程
3. **边界测试**：测试空目录、无效文件名等情况
