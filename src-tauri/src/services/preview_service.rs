use crate::domain::preview::PreviewModel;
use crate::storage::filesystem;
use crate::utils::errors::AppError;

pub fn get_preview(package_id: &str) -> Result<Option<PreviewModel>, AppError> {
    let preview = filesystem::scan_project_root(None)?
        .previews
        .into_iter()
        .find(|item| item.package_id == package_id);

    Ok(preview)
}
