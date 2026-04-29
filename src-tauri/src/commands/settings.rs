use crate::domain::common::AppResponse;
use crate::services::project_root_service;
use crate::services::skill_create_service;
use crate::state::app_state::AppState;
use crate::storage::filesystem;
use serde_json::json;

#[tauri::command]
pub async fn settings_get(
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<serde_json::Value>, String> {
    let current_project_root = state.current_project_root()?;
    let recent_project_roots =
        project_root_service::recent_project_roots(state.recent_project_roots()?)?;

    Ok(AppResponse::success(json!({
        "platform": "macOS",
        "shell": ["zsh", "bash"],
        "formalVersionCap": 10,
        "projectRootModel": "local_directory",
        "skillRootName": ".skills",
        "defaultProjectRoot": filesystem::default_project_root().to_string_lossy().to_string(),
        "currentProjectRoot": current_project_root,
        "recentProjectRoots": recent_project_roots,
        "creationBridge": skill_create_service::creator_bridge_status()
    })))
}

#[tauri::command]
pub async fn settings_update(
    payload: serde_json::Value,
) -> Result<AppResponse<serde_json::Value>, String> {
    Ok(AppResponse::success(payload))
}
