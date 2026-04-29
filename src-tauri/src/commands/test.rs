use crate::domain::common::AppResponse;
use crate::domain::test::PackageTestReport;
use crate::services::test_service;
use crate::state::app_state::AppState;

#[tauri::command]
pub async fn package_run_test(
    package_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AppResponse<PackageTestReport>, String> {
    let root_path = state.current_project_root()?;

    match test_service::run_package_test(&package_id, Some(root_path.as_str())) {
        Ok(report) => Ok(AppResponse::success(report)),
        Err(error) => Ok(AppResponse::failure("package_test_failed", error)),
    }
}
