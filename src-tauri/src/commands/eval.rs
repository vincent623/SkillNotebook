use crate::domain::common::AppResponse;
use crate::domain::eval::EvalReport;
use crate::services::eval_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn package_run_eval(
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<EvalReport>, String> {
    let root_path = state.current_workspace_root()?;

    match eval_service::run_eval(&package_id, Some(root_path.as_str())) {
        Ok(item) => Ok(AppResponse::success(item)),
        Err(error) => Ok(AppResponse::failure("eval_run_failed", error)),
    }
}
