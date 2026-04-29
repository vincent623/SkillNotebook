use crate::domain::common::AppResponse;
use crate::domain::project_root::ProjectRoot;
use crate::services::project_root_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn project_root_create(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<ProjectRoot>, String> {
    match project_root_service::create_project_root(&name) {
        Ok(project_root) => {
            state.set_current_project_root(&project_root.root_path)?;
            Ok(AppResponse::success(project_root))
        }
        Err(error) => Ok(AppResponse::failure("project_root_create_failed", error)),
    }
}

#[tauri::command]
pub async fn project_root_open(
    root_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<ProjectRoot>, String> {
    match project_root_service::open_project_root(&root_path) {
        Ok(project_root) => {
            state.set_current_project_root(&project_root.root_path)?;
            Ok(AppResponse::success(project_root))
        }
        Err(error) => Ok(AppResponse::failure("project_root_open_failed", error)),
    }
}

#[tauri::command]
pub async fn project_root_list_recent(
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<ProjectRoot>>, String> {
    let recent = state.recent_project_roots()?;

    match project_root_service::recent_project_roots(recent) {
        Ok(project_roots) => Ok(AppResponse::success(project_roots)),
        Err(error) => Ok(AppResponse::failure("project_root_list_failed", error)),
    }
}
