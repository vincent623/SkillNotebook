use crate::domain::common::AppResponse;
use crate::services::skill_create_service;
use crate::services::workspace_service;
use crate::state::app_state::AppState;
use crate::storage::filesystem;
use serde_json::json;

#[tauri::command]
pub async fn settings_get(
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<serde_json::Value>, String> {
    let current_workspace_root = state.current_workspace_root()?;
    let recent_workspaces = workspace_service::recent_workspaces(state.recent_workspace_roots()?)?;

    Ok(AppResponse::success(json!({
        "platform": "macOS",
        "shell": ["zsh", "bash"],
        "formalVersionCap": 10,
        "workspaceModel": "local_directory",
        "defaultWorkspaceRoot": filesystem::default_workspace_root().to_string_lossy().to_string(),
        "currentWorkspaceRoot": current_workspace_root,
        "recentWorkspaces": recent_workspaces,
        "creationBridge": skill_create_service::creator_bridge_status()
    })))
}

#[tauri::command]
pub async fn settings_update(
    payload: serde_json::Value,
) -> Result<AppResponse<serde_json::Value>, String> {
    Ok(AppResponse::success(payload))
}
