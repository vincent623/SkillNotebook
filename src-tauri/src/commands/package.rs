use crate::domain::common::AppResponse;
use crate::domain::package::{
    CreatePackageFromNlRequest, CreatePackageFromNlResponse, SearchResult, SkillPackage,
};
use crate::services::{package_service, skill_create_service};
use crate::state::app_state::AppState;
use crate::utils::errors::{not_found, not_implemented};

#[tauri::command]
pub async fn package_list(
    workspace_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<SkillPackage>>, String> {
    let root_path = state.current_workspace_root()?;

    match package_service::list_packages(Some(root_path.as_str())) {
        Ok(packages) => Ok(AppResponse::success(
            packages
                .into_iter()
                .filter(|item| item.workspace_id == workspace_id)
                .collect::<Vec<_>>(),
        )),
        Err(error) => Ok(AppResponse::failure("package_list_failed", error)),
    }
}

#[tauri::command]
pub async fn package_get(
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<SkillPackage>, String> {
    let root_path = state.current_workspace_root()?;

    match package_service::get_package(&package_id, Some(root_path.as_str())) {
        Ok(Some(item)) => Ok(AppResponse::success(item)),
        Ok(None) => Ok(not_found("package", &package_id)),
        Err(error) => Ok(AppResponse::failure("package_get_failed", error)),
    }
}

#[allow(dead_code)]
fn _search_results(root_path: Option<&str>) -> Vec<SearchResult> {
    match package_service::list_packages(root_path) {
        Ok(packages) => packages
            .into_iter()
            .map(|item| SearchResult {
                package_id: item.id,
                name: item.name,
                description: item.description,
                tags: item.tags,
                updated_at: item.updated_at,
                status: item.status,
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
pub async fn package_create_from_nl(
    req: CreatePackageFromNlRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<CreatePackageFromNlResponse>, String> {
    let root_path = state.current_workspace_root()?;

    match skill_create_service::create_package_from_nl(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_create_failed", error)),
    }
}

#[tauri::command]
pub async fn package_update(
    _package_id: String,
    _payload: serde_json::Value,
) -> Result<AppResponse<SkillPackage>, String> {
    Ok(not_implemented("package_update"))
}
