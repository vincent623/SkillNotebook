use crate::config::app_config;
use crate::domain::common::AppResponse;
use crate::services::project_root_service;
use crate::state::app_state::AppState;
use crate::storage::filesystem;
use serde_json::json;

#[tauri::command]
pub async fn settings_get(
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<serde_json::Value>, String> {
    match build_settings_payload(&state) {
        Ok(payload) => Ok(AppResponse::success(payload)),
        Err(error) => Ok(AppResponse::failure("settings_get_failed", error)),
    }
}

#[tauri::command]
pub async fn settings_update(
    payload: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<serde_json::Value>, String> {
    if let Err(error) = app_config::update_handoff_from_payload(&payload) {
        return Ok(AppResponse::failure("settings_update_failed", error));
    }

    match build_settings_payload(&state) {
        Ok(payload) => Ok(AppResponse::success(payload)),
        Err(error) => Ok(AppResponse::failure("settings_get_failed", error)),
    }
}

fn build_settings_payload(state: &AppState) -> Result<serde_json::Value, String> {
    let current_project_root = state.current_project_root()?;
    let recent_project_roots =
        project_root_service::recent_project_roots(state.recent_project_roots()?)?;
    let app_config = app_config::load_app_config();

    Ok(json!({
        "platform": "macOS",
        "shell": ["zsh", "bash"],
        "formalVersionCap": 10,
        "projectRootModel": "local_directory",
        "skillRootName": ".skills",
        "defaultProjectRoot": filesystem::default_project_root().to_string_lossy().to_string(),
        "currentProjectRoot": current_project_root,
        "recentProjectRoots": recent_project_roots,
        "settingsPath": app_config::app_settings_path().map(|path| path.to_string_lossy().to_string()),
        "handoff": app_config.handoff
    }))
}
