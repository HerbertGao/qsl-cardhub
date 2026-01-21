// 项目管理模块
//
// 提供转卡项目的 CRUD 操作

use crate::db::models::{format_datetime, now_china, Project, ProjectWithStats};
use crate::db::sqlite::get_connection;
use crate::error::AppError;

/// 创建新项目
pub fn create_project(name: String) -> Result<Project, AppError> {
    // 验证名称不为空
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::InvalidParameter("项目名称不能为空".to_string()));
    }

    let conn = get_connection()?;

    // 检查名称是否重复
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE name = ?1)",
            [&name],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Other(format!("查询项目失败: {}", e)))?;

    if exists {
        return Err(AppError::InvalidParameter(
            "项目名称已存在，请使用其他名称".to_string(),
        ));
    }

    // 创建项目
    let project = Project::new(name);

    conn.execute(
        "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [
            &project.id,
            &project.name,
            &project.created_at,
            &project.updated_at,
        ],
    )
    .map_err(|e| AppError::Other(format!("创建项目失败: {}", e)))?;

    log::info!("✅ 创建项目成功: {} ({})", project.name, project.id);
    Ok(project)
}

/// 查询项目列表（带统计信息）
pub fn list_projects() -> Result<Vec<ProjectWithStats>, AppError> {
    let conn = get_connection()?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                p.id,
                p.name,
                p.created_at,
                p.updated_at,
                COALESCE(COUNT(c.id), 0) as card_count
            FROM projects p
            LEFT JOIN cards c ON p.id = c.project_id
            GROUP BY p.id
            ORDER BY p.created_at DESC
            "#,
        )
        .map_err(|e| AppError::Other(format!("准备查询语句失败: {}", e)))?;

    let projects = stmt
        .query_map([], |row| {
            Ok(ProjectWithStats {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                card_count: row.get(4)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询项目列表失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取项目数据失败: {}", e)))?;

    Ok(projects)
}

/// 获取单个项目
pub fn get_project(id: &str) -> Result<Option<Project>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        "SELECT id, name, created_at, updated_at FROM projects WHERE id = ?1",
        [id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    );

    match result {
        Ok(project) => Ok(Some(project)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询项目失败: {}", e))),
    }
}

/// 更新项目名称
pub fn update_project(id: &str, name: String) -> Result<Project, AppError> {
    // 验证名称不为空
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::InvalidParameter("项目名称不能为空".to_string()));
    }

    let conn = get_connection()?;

    // 检查项目是否存在
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Other(format!("查询项目失败: {}", e)))?;

    if !exists {
        return Err(AppError::ProfileNotFound(format!("项目不存在: {}", id)));
    }

    // 检查新名称是否与其他项目重复
    let name_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE name = ?1 AND id != ?2)",
            [&name, id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Other(format!("查询项目失败: {}", e)))?;

    if name_exists {
        return Err(AppError::InvalidParameter(
            "项目名称已存在，请使用其他名称".to_string(),
        ));
    }

    // 更新项目
    let updated_at = format_datetime(&now_china());

    conn.execute(
        "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
        [&name, &updated_at, id],
    )
    .map_err(|e| AppError::Other(format!("更新项目失败: {}", e)))?;

    // 返回更新后的项目
    get_project(id)?.ok_or_else(|| AppError::Other("更新后无法获取项目".to_string()))
}

/// 删除项目
pub fn delete_project(id: &str) -> Result<(), AppError> {
    let conn = get_connection()?;

    // 检查项目是否存在
    let project = get_project(id)?;
    if project.is_none() {
        return Err(AppError::ProfileNotFound(format!("项目不存在: {}", id)));
    }

    // 删除项目（级联删除关联的卡片）
    conn.execute("DELETE FROM projects WHERE id = ?1", [id])
        .map_err(|e| AppError::Other(format!("删除项目失败: {}", e)))?;

    log::info!("✅ 删除项目成功: {}", id);
    Ok(())
}

#[cfg(test)]
mod tests {
    // 测试需要独立的数据库实例，暂时跳过
}
