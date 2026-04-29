use crate::domain::common::AppResponse;
use crate::domain::package::SkillPackage;
use crate::domain::version::{PackageVersion, PackageVersionDiff};
use crate::services::version_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn package_list_versions(
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<PackageVersion>>, String> {
    let root_path = state.current_project_root()?;

    match version_service::list_versions(&package_id, Some(root_path.as_str())) {
        Ok(versions) => Ok(AppResponse::success(versions)),
        Err(error) => Ok(AppResponse::failure("version_list_failed", error)),
    }
}

#[tauri::command]
pub async fn package_save_version(
    package_id: String,
    note: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageVersion>, String> {
    let root_path = state.current_project_root()?;

    match version_service::save_version(&package_id, note, Some(root_path.as_str())) {
        Ok(version) => Ok(AppResponse::success(version)),
        Err(error) => Ok(AppResponse::failure("version_save_failed", error)),
    }
}

#[tauri::command]
pub async fn package_restore_version(
    version_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<SkillPackage>, String> {
    let root_path = state.current_project_root()?;

    match version_service::restore_version(&version_id, Some(root_path.as_str())) {
        Ok(package) => Ok(AppResponse::success(package)),
        Err(error) => Ok(AppResponse::failure("version_restore_failed", error)),
    }
}

#[tauri::command]
pub async fn package_diff_version(
    version_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageVersionDiff>, String> {
    let root_path = state.current_project_root()?;

    match version_service::diff_version(&version_id, Some(root_path.as_str())) {
        Ok(diff) => Ok(AppResponse::success(diff)),
        Err(error) => Ok(AppResponse::failure("version_diff_failed", error)),
    }
}
