use crate::domain::common::AppResponse;
use crate::domain::package::SearchResult;
use crate::services::search_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn package_search(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<SearchResult>>, String> {
    let root_path = state.current_workspace_root()?;

    match search_service::search_packages(&query, Some(root_path.as_str())) {
        Ok(results) => Ok(AppResponse::success(results)),
        Err(error) => Ok(AppResponse::failure("package_search_failed", error)),
    }
}
