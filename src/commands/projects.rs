// 项目管理 Tauri 命令
//
// 提供前端调用的项目管理 API

use crate::db::{self, Project, ProjectWithStats};

/// 创建新项目
#[tauri::command]
pub async fn create_project_cmd(name: String) -> Result<Project, String> {
    tokio::task::spawn_blocking(move || db::create_project(name).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 查询项目列表
#[tauri::command]
pub async fn list_projects_cmd() -> Result<Vec<ProjectWithStats>, String> {
    tokio::task::spawn_blocking(|| db::list_projects().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 获取单个项目
#[tauri::command]
pub async fn get_project_cmd(id: String) -> Result<Option<Project>, String> {
    tokio::task::spawn_blocking(move || db::get_project(&id).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 更新项目名称
#[tauri::command]
pub async fn update_project_cmd(id: String, name: String) -> Result<Project, String> {
    tokio::task::spawn_blocking(move || db::update_project(&id, name).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 删除项目
#[tauri::command]
pub async fn delete_project_cmd(id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || db::delete_project(&id).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}
