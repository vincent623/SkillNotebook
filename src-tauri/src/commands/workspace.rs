use crate::domain::common::AppResponse;
use crate::domain::workspace::Workspace;
use crate::services::workspace_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn workspace_create(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Workspace>, String> {
    match workspace_service::create_workspace(&name) {
        Ok(workspace) => {
            state.set_current_workspace_root(&workspace.root_path)?;
            Ok(AppResponse::success(workspace))
        }
        Err(error) => Ok(AppResponse::failure("workspace_create_failed", error)),
    }
}

#[tauri::command]
pub async fn workspace_open(
    root_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Workspace>, String> {
    match workspace_service::open_workspace(&root_path) {
        Ok(workspace) => {
            state.set_current_workspace_root(&workspace.root_path)?;
            Ok(AppResponse::success(workspace))
        }
        Err(error) => Ok(AppResponse::failure("workspace_open_failed", error)),
    }
}

#[tauri::command]
pub async fn workspace_list_recent(
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<Workspace>>, String> {
    let recent = state.recent_workspace_roots()?;

    match workspace_service::recent_workspaces(recent) {
        Ok(workspaces) => Ok(AppResponse::success(workspaces)),
        Err(error) => Ok(AppResponse::failure("workspace_list_failed", error)),
    }
}
