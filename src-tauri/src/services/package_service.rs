use std::path::PathBuf;

use crate::domain::package::{
    PackageFileContent, PackageFileEntry, PackageUpdateRequest, SkillPackage,
};
use crate::storage::filesystem;
use crate::utils::time::now_iso;

pub fn list_packages(root_path: Option<&str>) -> Result<Vec<SkillPackage>, String> {
    Ok(filesystem::scan_project_root(root_path)?.packages)
}

pub fn get_package(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Option<SkillPackage>, String> {
    let package = filesystem::scan_project_root(root_path)?
        .packages
        .into_iter()
        .find(|item| item.id == package_id);

    Ok(package)
}

pub fn list_package_files(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Vec<PackageFileEntry>, String> {
    let package_root = package_root(package_id, root_path)?;
    filesystem::list_package_file_tree(&package_root)
}

pub fn read_package_file(
    package_id: &str,
    path: &str,
    root_path: Option<&str>,
) -> Result<PackageFileContent, String> {
    let package_root = package_root(package_id, root_path)?;
    filesystem::read_package_text_file(&package_root, path)
}

pub fn write_package_file(
    package_id: &str,
    path: &str,
    content: &str,
    root_path: Option<&str>,
) -> Result<PackageFileContent, String> {
    let package_root = package_root(package_id, root_path)?;
    filesystem::write_package_text_file(&package_root, path, content)
}

pub fn update_package(
    package_id: &str,
    payload: &PackageUpdateRequest,
    root_path: Option<&str>,
) -> Result<SkillPackage, String> {
    let package = get_package(package_id, root_path)?
        .ok_or_else(|| format!("package not found: {}", package_id))?;
    let package_root = PathBuf::from(&package.root_path);
    let mut notebook = filesystem::load_package_notebook(&package_root)?;

    if let Some(name) = payload.name.as_deref() {
        let next_name = name.trim();
        if next_name.is_empty() {
            return Err("package name cannot be empty".to_string());
        }
        notebook.name = next_name.to_string();
    }
    if let Some(description) = payload.description.as_deref() {
        notebook.description = description.trim().to_string();
    }
    if let Some(tags) = &payload.tags {
        notebook.tags = normalize_string_list(tags, 12);
    }
    if let Some(status) = &payload.status {
        notebook.status = status.clone();
    }
    if let Some(related_skills) = &payload.related_skills {
        notebook.related_skills = normalize_string_list(related_skills, 24);
    }
    if let Some(bundle_candidates) = &payload.bundle_candidates {
        notebook.bundle_candidates = normalize_string_list(bundle_candidates, 24);
    }

    notebook.updated_at = now_iso();
    filesystem::save_package_notebook(&package_root, &notebook)?;

    get_package(package_id, root_path)?.ok_or_else(|| {
        format!(
            "package {} was updated but could not be reloaded from {}",
            package_id,
            package_root.display()
        )
    })
}

fn package_root(package_id: &str, root_path: Option<&str>) -> Result<PathBuf, String> {
    let package = get_package(package_id, root_path)?
        .ok_or_else(|| format!("package not found: {}", package_id))?;
    Ok(PathBuf::from(package.root_path))
}

fn normalize_string_list(values: &[String], limit: usize) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let item = value.trim().to_string();
        if item.is_empty() || normalized.contains(&item) {
            continue;
        }
        normalized.push(item);
        if normalized.len() >= limit {
            break;
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::update_package;
    use crate::domain::package::{PackageStatus, PackageUpdateRequest};
    use crate::storage::filesystem;

    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_project_root_path() -> PathBuf {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "skill-notebook-package-service-{}-{}",
            std::process::id(),
            seed
        ))
    }

    fn copy_example_project_root(destination: &PathBuf) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("examples")
            .join("project-root");
        filesystem::copy_directory_recursive(&root, destination).expect("copy project root");
        destination.clone()
    }

    #[test]
    fn updates_package_notebook_metadata() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);

        let updated = update_package(
            "pkg-interview",
            &PackageUpdateRequest {
                name: Some("Interview Signal Mapper".to_string()),
                description: Some("Maps interview notes into clear signals.".to_string()),
                tags: Some(vec![
                    "research".to_string(),
                    "signals".to_string(),
                    "research".to_string(),
                ]),
                status: Some(PackageStatus::NeedsEval),
                related_skills: Some(vec!["persona-composer".to_string()]),
                bundle_candidates: Some(vec!["research-pipeline".to_string()]),
            },
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("package updated");

        assert_eq!(updated.name, "Interview Signal Mapper");
        assert_eq!(updated.tags, vec!["research", "signals"]);
        assert!(matches!(updated.status, PackageStatus::NeedsEval));

        let notebook = filesystem::load_package_notebook(
            &filesystem::canonical_skills_root(&root).join("interview-insight-extractor"),
        )
        .expect("notebook reload");
        assert_eq!(notebook.name, "Interview Signal Mapper");

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn rejects_empty_package_name_update() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);

        let error = update_package(
            "pkg-interview",
            &PackageUpdateRequest {
                name: Some("   ".to_string()),
                ..PackageUpdateRequest::default()
            },
            Some(root.to_string_lossy().as_ref()),
        )
        .expect_err("empty name should fail");

        assert!(error.contains("name cannot be empty"));
        std::fs::remove_dir_all(root).ok();
    }
}
