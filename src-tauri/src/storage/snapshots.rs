use std::fs;
use std::path::{Path, PathBuf};

use crate::storage::filesystem;
use crate::utils::errors::AppError;

const SNAPSHOT_PREFIX: &str = ".skill-notebook/snapshots";

fn ensure_safe_snapshot_path(snapshot_path: &str) -> Result<(), AppError> {
    if Path::new(snapshot_path).is_absolute() {
        return Err(AppError::InvalidPath(format!(
            "refusing to operate on absolute snapshot path: {}",
            snapshot_path
        )));
    }

    if Path::new(snapshot_path)
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(AppError::InvalidPath(format!(
            "refusing to operate on snapshot path containing '..': {}",
            snapshot_path
        )));
    }

    if snapshot_path.starts_with(SNAPSHOT_PREFIX) {
        return Ok(());
    }

    Err(AppError::InvalidPath(format!(
        "refusing to operate on snapshot path outside {}: {}",
        SNAPSHOT_PREFIX, snapshot_path
    )))
}

fn copy_directory_filtered(source: &Path, destination: &Path) -> Result<(), AppError> {
    filesystem::ensure_directory(destination)?;

    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| AppError::Other(format!("missing file name for {}", path.display())))?
            .to_string_lossy()
            .to_string();

        if should_skip_snapshot_entry(&file_name) {
            continue;
        }

        let destination_path = destination.join(&file_name);
        if path.is_dir() {
            copy_directory_filtered(&path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                filesystem::ensure_directory(parent)?;
            }

            fs::copy(&path, &destination_path)?;
        }
    }

    Ok(())
}

fn should_skip_snapshot_entry(file_name: &str) -> bool {
    file_name == "notebook.json"
        || file_name == ".DS_Store"
        || file_name == ".git"
        || file_name == "node_modules"
        || file_name.starts_with('.')
}

fn resolve_snapshot_root(project_root_path: &Path, snapshot_path: &str) -> Result<PathBuf, AppError> {
    ensure_safe_snapshot_path(snapshot_path)?;
    Ok(project_root_path.join(snapshot_path))
}

pub fn snapshot_has_restorable_content(
    project_root_path: &Path,
    snapshot_path: &str,
) -> Result<bool, AppError> {
    let snapshot_root = resolve_snapshot_root(project_root_path, snapshot_path)?;
    if !snapshot_root.exists() {
        return Err(AppError::Other(format!(
            "snapshot directory does not exist: {}",
            snapshot_root.display()
        )));
    }

    for entry in fs::read_dir(&snapshot_root)? {
        let path = entry?.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| AppError::Other(format!("missing file name for {}", path.display())))?
            .to_string_lossy()
            .to_string();

        if should_skip_snapshot_entry(&file_name) || file_name == "README.md" {
            continue;
        }

        return Ok(true);
    }

    Ok(false)
}

pub fn collect_snapshot_files(root: &Path) -> Result<Vec<String>, AppError> {
    let mut files = Vec::new();
    collect_snapshot_files_recursive(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_snapshot_files_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<String>,
) -> Result<(), AppError> {
    for entry in fs::read_dir(current)? {
        let path = entry?.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| AppError::Other(format!("missing file name for {}", path.display())))?
            .to_string_lossy()
            .to_string();

        if should_skip_snapshot_entry(&file_name) {
            continue;
        }

        if path.is_dir() {
            collect_snapshot_files_recursive(root, &path, files)?;
            continue;
        }

        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .to_string();
        files.push(relative);
    }

    Ok(())
}

pub fn restore_snapshot(
    project_root_path: &Path,
    package_root: &Path,
    snapshot_path: &str,
) -> Result<(), AppError> {
    let snapshot_root = resolve_snapshot_root(project_root_path, snapshot_path)?;
    if !snapshot_root.exists() {
        return Err(AppError::Other(format!(
            "snapshot directory does not exist: {}",
            snapshot_root.display()
        )));
    }

    if !snapshot_has_restorable_content(project_root_path, snapshot_path)? {
        return Err(AppError::Other(format!(
            "snapshot {} does not contain restorable package files",
            snapshot_path
        )));
    }

    clear_package_contents(package_root)?;
    copy_directory_filtered(&snapshot_root, package_root)?;
    Ok(())
}

fn clear_package_contents(package_root: &Path) -> Result<(), AppError> {
    for entry in fs::read_dir(package_root)? {
        let path = entry?.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| AppError::Other(format!("missing file name for {}", path.display())))?
            .to_string_lossy()
            .to_string();

        if should_skip_snapshot_entry(&file_name) {
            continue;
        }

        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}

pub fn snapshot_package(
    project_root_path: &Path,
    package_root: &Path,
    package_id: &str,
    version_number: u32,
) -> Result<String, AppError> {
    let snapshot_path = format!("{}/{}/v{}", SNAPSHOT_PREFIX, package_id, version_number);
    ensure_safe_snapshot_path(&snapshot_path)?;

    let destination = project_root_path.join(&snapshot_path);
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }

    copy_directory_filtered(package_root, &destination)?;

    Ok(snapshot_path)
}

pub fn delete_snapshot(project_root_path: &Path, snapshot_path: &str) -> Result<(), AppError> {
    ensure_safe_snapshot_path(snapshot_path)?;

    let absolute = project_root_path.join(snapshot_path);
    if !absolute.exists() {
        return Ok(());
    }

    if absolute.is_dir() {
        fs::remove_dir_all(&absolute)?;
        return Ok(());
    }

    fs::remove_file(&absolute)?;
    Ok(())
}

pub fn snapshots_root(project_root_path: &Path) -> PathBuf {
    project_root_path.join(SNAPSHOT_PREFIX)
}
