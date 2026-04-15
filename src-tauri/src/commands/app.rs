use crate::domain::common::{AppBootstrap, AppResponse};
use crate::services::bootstrap_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn app_bootstrap(
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<AppBootstrap>, String> {
    let root_path = state.current_workspace_root()?;

    match bootstrap_service::build_bootstrap(Some(root_path.as_str())) {
        Ok(data) => Ok(AppResponse::success(data)),
        Err(error) => Ok(AppResponse::failure("bootstrap_failed", error)),
    }
}
