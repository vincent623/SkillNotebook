use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::domain::package::PackageExportArtifact;
use crate::storage::filesystem;
use crate::utils::ids::slugify;
use crate::utils::time::now_iso;

pub fn export_package_zip(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<PackageExportArtifact, String> {
    let scanned = filesystem::scan_project_root(root_path)?;
    let project_root = PathBuf::from(&scanned.project_root.root_path);
    let package = scanned
        .packages
        .into_iter()
        .find(|item| item.id == package_id)
        .ok_or_else(|| format!("package not found: {}", package_id))?;
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
        fs::remove_file(&zip_path).map_err(|error| {
            format!(
                "failed to replace existing export {}: {}",
                zip_path.display(),
                error
            )
        })?;
    }
    if staging_parent.exists() {
        fs::remove_dir_all(&staging_parent).map_err(|error| {
            format!(
                "failed to reset export staging {}: {}",
                staging_parent.display(),
                error
            )
        })?;
    }

    copy_visible_package_files(&package_root, &staging_package_root)?;
    let export_result = zip_staged_package(&staging_parent, &package.slug, &zip_path);
    fs::remove_dir_all(&staging_parent).ok();
    export_result?;

    let size_bytes = fs::metadata(&zip_path)
        .map_err(|error| format!("failed to inspect export {}: {}", zip_path.display(), error))?
        .len();

    Ok(PackageExportArtifact {
        package_id: package.id,
        zip_path: zip_path.to_string_lossy().to_string(),
        size_bytes,
        created_at,
    })
}

fn copy_visible_package_files(source: &Path, destination: &Path) -> Result<(), String> {
    filesystem::ensure_directory(destination)?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read package {}: {}", source.display(), error))?
    {
        let entry = entry.map_err(|error| format!("failed to inspect package entry: {}", error))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip_export_entry(&name) {
            continue;
        }

        let source_path = entry.path();
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| format!("failed to inspect {}: {}", source_path.display(), error))?;
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
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!(
                    "failed to copy {} to {}: {}",
                    source_path.display(),
                    destination_path.display(),
                    error
                )
            })?;
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
) -> Result<(), String> {
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
        .map_err(|error| format!("failed to launch zip export command: {}", error))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "zip export failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::export_package_zip;
    use crate::storage::filesystem;

    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_project_root_path() -> PathBuf {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "skill-notebook-export-test-{}-{}",
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
        filesystem::copy_directory_recursive(&root, destination).expect("copy project_root");
        destination.clone()
    }

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
