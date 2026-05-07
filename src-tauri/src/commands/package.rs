use crate::domain::common::AppResponse;
use crate::domain::package::{
    PackageExportArtifact, PackageFileContent, PackageFileEntry, PackageImportRequest,
    PackageImportResponse, PackageReferenceResponse, PackageUpdateRequest, SearchResult,
    SkillPackage,
};
use crate::services::{export_service, package_service};
use crate::state::app_state::AppState;
use crate::utils::errors::not_found;

#[tauri::command]
pub async fn package_list(
    project_root_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<SkillPackage>>, String> {
    let root_path = state.current_project_root()?;

    match package_service::list_packages(Some(root_path.as_str())) {
        Ok(packages) => Ok(AppResponse::success(
            packages
                .into_iter()
                .filter(|item| item.project_root_id == project_root_id)
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
    let root_path = state.current_project_root()?;

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
pub async fn package_update(
    package_id: String,
    payload: PackageUpdateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<SkillPackage>, String> {
    let root_path = state.current_project_root()?;

    match package_service::update_package(&package_id, &payload, Some(root_path.as_str())) {
        Ok(package) => Ok(AppResponse::success(package)),
        Err(error) => Ok(AppResponse::failure("package_update_failed", error)),
    }
}

#[tauri::command]
pub async fn package_export_zip(
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageExportArtifact>, String> {
    let root_path = state.current_project_root()?;

    match export_service::export_package_zip(&package_id, Some(root_path.as_str())) {
        Ok(artifact) => Ok(AppResponse::success(artifact)),
        Err(error) => Ok(AppResponse::failure("package_export_failed", error)),
    }
}

#[tauri::command]
pub async fn package_reference(
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageReferenceResponse>, String> {
    let root_path = state.current_project_root()?;

    match package_service::reference_package(&package_id, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_reference_failed", error)),
    }
}

#[tauri::command]
pub async fn package_import(
    req: PackageImportRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageImportResponse>, String> {
    let root_path = state.current_project_root()?;

    match package_service::import_package(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_import_failed", error)),
    }
}

#[tauri::command]
pub async fn package_file_tree(
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<Vec<PackageFileEntry>>, String> {
    let root_path = state.current_project_root()?;

    match package_service::list_package_files(&package_id, Some(root_path.as_str())) {
        Ok(entries) => Ok(AppResponse::success(entries)),
        Err(error) => Ok(AppResponse::failure("package_file_tree_failed", error)),
    }
}

#[tauri::command]
pub async fn package_file_read(
    package_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageFileContent>, String> {
    let root_path = state.current_project_root()?;

    match package_service::read_package_file(&package_id, &path, Some(root_path.as_str())) {
        Ok(file) => Ok(AppResponse::success(file)),
        Err(error) => Ok(AppResponse::failure("package_file_read_failed", error)),
    }
}

#[tauri::command]
pub async fn package_file_write(
    package_id: String,
    path: String,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageFileContent>, String> {
    let root_path = state.current_project_root()?;

    match package_service::write_package_file(
        &package_id,
        &path,
        &content,
        Some(root_path.as_str()),
    ) {
        Ok(file) => Ok(AppResponse::success(file)),
        Err(error) => Ok(AppResponse::failure("package_file_write_failed", error)),
    }
}
