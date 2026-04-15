use crate::domain::package::SkillPackage;
use crate::storage::filesystem;

pub fn list_packages(root_path: Option<&str>) -> Result<Vec<SkillPackage>, String> {
    Ok(filesystem::scan_workspace(root_path)?.packages)
}

pub fn get_package(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Option<SkillPackage>, String> {
    let package = filesystem::scan_workspace(root_path)?
        .packages
        .into_iter()
        .find(|item| item.id == package_id);

    Ok(package)
}
