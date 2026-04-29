use crate::domain::common::AppResponse;
use crate::domain::package::{
    CommitPackagePreviewRequest, CreatePackageFromNlRequest, CreatePackageFromNlResponse,
    CreatePackageFromSourcesRequest, CreatePackageFromUrlRequest, CreatePackagePreviewResponse,
    DiscardPackagePreviewRequest, PackageExportArtifact, PackageFileContent, PackageFileEntry,
    PackageUpdateRequest, SearchResult, SkillPackage,
};
use crate::services::{export_service, package_service, skill_create_service};
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
pub async fn package_create_from_nl(
    req: CreatePackageFromNlRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<CreatePackageFromNlResponse>, String> {
    let root_path = state.current_project_root()?;

    match skill_create_service::create_package_from_nl(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_create_failed", error)),
    }
}

#[tauri::command]
pub async fn package_generate_preview_from_nl(
    req: CreatePackageFromNlRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<CreatePackagePreviewResponse>, String> {
    let root_path = state.current_project_root()?;

    match skill_create_service::generate_package_preview_from_nl(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_preview_failed", error)),
    }
}

#[tauri::command]
pub async fn package_generate_preview_from_sources(
    req: CreatePackageFromSourcesRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<CreatePackagePreviewResponse>, String> {
    let root_path = state.current_project_root()?;

    match skill_create_service::generate_package_preview_from_sources(
        &req,
        Some(root_path.as_str()),
    ) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_source_preview_failed", error)),
    }
}

#[tauri::command]
pub async fn package_generate_preview_from_url(
    req: CreatePackageFromUrlRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<CreatePackagePreviewResponse>, String> {
    let root_path = state.current_project_root()?;

    match skill_create_service::generate_package_preview_from_url(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_url_preview_failed", error)),
    }
}

#[tauri::command]
pub async fn package_commit_preview(
    req: CommitPackagePreviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<CreatePackageFromNlResponse>, String> {
    let root_path = state.current_project_root()?;

    match skill_create_service::commit_package_preview(&req, Some(root_path.as_str())) {
        Ok(response) => Ok(AppResponse::success(response)),
        Err(error) => Ok(AppResponse::failure("package_preview_commit_failed", error)),
    }
}

#[tauri::command]
pub async fn package_discard_preview(
    req: DiscardPackagePreviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<bool>, String> {
    let root_path = state.current_project_root()?;

    match skill_create_service::discard_package_preview(&req, Some(root_path.as_str())) {
        Ok(discarded) => Ok(AppResponse::success(discarded)),
        Err(error) => Ok(AppResponse::failure(
            "package_preview_discard_failed",
            error,
        )),
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
