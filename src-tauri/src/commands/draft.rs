use crate::domain::common::AppResponse;
use crate::domain::draft::{
    DraftDiscardRequest, DraftImportRequest, DraftImportResponse, DraftStartRequest, DraftWorkspace,
};
use crate::services::draft_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn draft_start(
    req: DraftStartRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<DraftWorkspace>, String> {
    let root_path = state.current_project_root()?;

    match draft_service::start_draft(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("draft_start_failed", error)),
    }
}

#[tauri::command]
pub async fn draft_list(
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<DraftWorkspace>>, String> {
    let root_path = state.current_project_root()?;

    match draft_service::list_drafts(Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("draft_list_failed", error)),
    }
}

#[tauri::command]
pub async fn draft_discard(
    req: DraftDiscardRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<bool>, String> {
    let root_path = state.current_project_root()?;

    match draft_service::discard_draft(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("draft_discard_failed", error)),
    }
}

#[tauri::command]
pub async fn draft_import(
    req: DraftImportRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<DraftImportResponse>, String> {
    let root_path = state.current_project_root()?;

    match draft_service::import_draft(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("draft_import_failed", error)),
    }
}
