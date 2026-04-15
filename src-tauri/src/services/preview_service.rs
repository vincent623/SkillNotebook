use crate::domain::preview::PreviewModel;
use crate::storage::filesystem;

pub fn get_preview(package_id: &str) -> Result<Option<PreviewModel>, String> {
    let preview = filesystem::scan_workspace(None)?
        .previews
        .into_iter()
        .find(|item| item.package_id == package_id);

    Ok(preview)
}
