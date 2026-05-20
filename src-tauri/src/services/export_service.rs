use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::domain::package::PackageExportArtifact;
use crate::storage::filesystem;
use crate::utils::errors::AppError;
use crate::utils::ids::slugify;
use crate::utils::time::now_iso;

pub fn export_package_zip(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<PackageExportArtifact, AppError> {
    let scanned = filesystem::scan_project_root(root_path)?;
    let project_root = PathBuf::from(&scanned.project_root.root_path);
    let package = scanned
        .packages
        .into_iter()
        .find(|item| item.id == package_id)
        .ok_or_else(|| AppError::NotFound {
            entity: "package".to_string(),
            identifier: package_id.to_string(),
        })?;
    let package_root = PathBuf::from(&package.root_path);
    let created_at = now_iso();
    let timestamp = slugify(&created_at).trim_matches('-').to_string();
    let exports_root = project_root.join(".skill-notebook").join("exports");
    let staging_parent = project_root
        .join(".skill-notebook")
        .join("cache")
        .join(format!("export-{}-{}", package.slug, timestamp));
    let staging_package_root = staging_parent.join(&package.slug);
    let zip_path = exports_root.join(format!(
        "{}-v{}-{}.zip",
        package.slug, package.current_version, timestamp
    ));

    filesystem::ensure_directory(&exports_root)?;
    if zip_path.exists() {
        fs::remove_file(&zip_path)?;
    }
    if staging_parent.exists() {
        fs::remove_dir_all(&staging_parent)?;
    }

    copy_visible_package_files(&package_root, &staging_package_root)?;
    let export_result = zip_staged_package(&staging_parent, &package.slug, &zip_path);
    fs::remove_dir_all(&staging_parent).ok();
    export_result?;

    let size_bytes = fs::metadata(&zip_path)?.len();

    Ok(PackageExportArtifact {
        package_id: package.id,
        zip_path: zip_path.to_string_lossy().to_string(),
        size_bytes,
        created_at,
    })
}

fn copy_visible_package_files(source: &Path, destination: &Path) -> Result<(), AppError> {
    filesystem::ensure_directory(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip_export_entry(&name) {
            continue;
        }

        let source_path = entry.path();
        let metadata = fs::symlink_metadata(&source_path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }

        let destination_path = destination.join(&name);
        if metadata.is_dir() {
            copy_visible_package_files(&source_path, &destination_path)?;
        } else if metadata.is_file() {
            if let Some(parent) = destination_path.parent() {
                filesystem::ensure_directory(parent)?;
            }
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}

fn should_skip_export_entry(name: &str) -> bool {
    name.starts_with('.') || name == "notebook.json"
}

fn zip_staged_package(
    staging_parent: &Path,
    package_dir_name: &str,
    zip_path: &Path,
) -> Result<(), AppError> {
    let ditto_status = Command::new("ditto")
        .arg("-c")
        .arg("-k")
        .arg("--keepParent")
        .arg(package_dir_name)
        .arg(zip_path)
        .current_dir(staging_parent)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status();

    if let Ok(status) = ditto_status {
        if status.success() {
            return Ok(());
        }
    }

    let output = Command::new("zip")
        .arg("-q")
        .arg("-r")
        .arg(zip_path)
        .arg(package_dir_name)
        .current_dir(staging_parent)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| AppError::Other(format!("failed to launch zip export command: {}", error)))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::Other(format!(
            "zip export failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::export_package_zip;

    use std::path::PathBuf;

    use crate::test_helpers::{copy_example_project_root, tmp_project_root_path};

    #[test]
    fn exports_a_sanitized_package_zip() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);

        let artifact = export_package_zip("pkg-interview", Some(root.to_string_lossy().as_ref()))
            .expect("zip export should succeed");

        assert!(PathBuf::from(&artifact.zip_path).exists());
        assert!(artifact.size_bytes > 0);
        assert_eq!(artifact.package_id, "pkg-interview");
        std::fs::remove_dir_all(root).ok();
    }
}
