use crate::domain::common::AppResponse;

#[tauri::command]
pub async fn package_run_test(package_id: String) -> Result<AppResponse<String>, String> {
    let output = format!(
        "CLI test bridge is scaffolded for package `{}`. Real macOS shell execution comes next.",
        package_id
    );

    Ok(AppResponse::success(output))
}
