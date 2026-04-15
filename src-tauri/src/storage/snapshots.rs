use std::fs;
use std::path::{Path, PathBuf};

use crate::storage::filesystem;

const SNAPSHOT_PREFIX: &str = ".skill-notebook/snapshots";

fn ensure_safe_snapshot_path(snapshot_path: &str) -> Result<(), String> {
    if Path::new(snapshot_path).is_absolute() {
        return Err(format!(
            "refusing to operate on absolute snapshot path: {}",
            snapshot_path
        ));
    }

    if Path::new(snapshot_path)
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "refusing to operate on snapshot path containing '..': {}",
            snapshot_path
        ));
    }

    if snapshot_path.starts_with(SNAPSHOT_PREFIX) {
        return Ok(());
    }

    Err(format!(
        "refusing to operate on snapshot path outside {}: {}",
        SNAPSHOT_PREFIX, snapshot_path
    ))
}

fn copy_directory_filtered(source: &Path, destination: &Path) -> Result<(), String> {
    filesystem::ensure_directory(destination)?;

    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read directory {}: {}", source.display(), error))?
    {
        let path = entry
            .map_err(|error| format!("failed to inspect directory entry: {}", error))?
            .path();
        let file_name = path
            .file_name()
            .ok_or_else(|| format!("missing file name for {}", path.display()))?
            .to_string_lossy()
            .to_string();

        if file_name == "notebook.json"
            || file_name == ".DS_Store"
            || file_name == ".git"
            || file_name == "node_modules"
        {
            continue;
        }

        let destination_path = destination.join(&file_name);
        if path.is_dir() {
            copy_directory_filtered(&path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                filesystem::ensure_directory(parent)?;
            }

            fs::copy(&path, &destination_path).map_err(|error| {
                format!(
                    "failed to copy {} to {}: {}",
                    path.display(),
                    destination_path.display(),
                    error
                )
            })?;
        }
    }

    Ok(())
}

pub fn snapshot_package(
    workspace_root: &Path,
    package_root: &Path,
    package_id: &str,
    version_number: u32,
) -> Result<String, String> {
    let snapshot_path = format!("{}/{}/v{}", SNAPSHOT_PREFIX, package_id, version_number);
    ensure_safe_snapshot_path(&snapshot_path)?;

    let destination = workspace_root.join(&snapshot_path);
    if destination.exists() {
        fs::remove_dir_all(&destination).map_err(|error| {
            format!(
                "failed to remove existing snapshot directory {}: {}",
                destination.display(),
                error
            )
        })?;
    }

    copy_directory_filtered(package_root, &destination)?;

    Ok(snapshot_path)
}

pub fn delete_snapshot(workspace_root: &Path, snapshot_path: &str) -> Result<(), String> {
    ensure_safe_snapshot_path(snapshot_path)?;

    let absolute = workspace_root.join(snapshot_path);
    if !absolute.exists() {
        return Ok(());
    }

    if absolute.is_dir() {
        fs::remove_dir_all(&absolute).map_err(|error| {
            format!(
                "failed to delete snapshot directory {}: {}",
                absolute.display(),
                error
            )
        })?;
        return Ok(());
    }

    fs::remove_file(&absolute).map_err(|error| {
        format!(
            "failed to delete snapshot file {}: {}",
            absolute.display(),
            error
        )
    })?;
    Ok(())
}

pub fn snapshots_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(SNAPSHOT_PREFIX)
}
