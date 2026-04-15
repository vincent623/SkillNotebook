use std::cmp::max;
use std::path::PathBuf;

use crate::domain::version::PackageVersion;
use crate::storage::filesystem;
use crate::storage::snapshots;
use crate::utils::time::now_iso;

const FORMAL_VERSION_CAP: usize = 10;

pub fn list_versions(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Vec<PackageVersion>, String> {
    let mut versions = filesystem::scan_workspace(root_path)?
        .versions
        .into_iter()
        .filter(|item| item.package_id == package_id)
        .collect::<Vec<_>>();

    versions.sort_by(|left, right| {
        right
            .version_number
            .cmp(&left.version_number)
            .then_with(|| right.created_at.cmp(&left.created_at))
    });

    Ok(versions)
}

pub fn save_version(
    package_id: &str,
    note: Option<String>,
    root_path: Option<&str>,
) -> Result<PackageVersion, String> {
    let scanned = filesystem::scan_workspace(root_path)?;
    let package = scanned
        .packages
        .iter()
        .find(|item| item.id == package_id)
        .cloned()
        .ok_or_else(|| format!("package not found: {}", package_id))?;

    let workspace_root = PathBuf::from(&scanned.workspace.root_path);
    let package_root = PathBuf::from(&package.root_path);
    let mut notebook = filesystem::load_package_notebook(&package_root)?;
    let eval_report_id = notebook
        .eval_reports
        .first()
        .map(|report| report.id.clone())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "package {} has no eval report yet; run eval before saving a formal version",
                package_id
            )
        })?;

    let max_existing = notebook
        .versions
        .iter()
        .map(|item| item.version_number)
        .max()
        .unwrap_or(0);
    let next_version_number = max(max_existing, notebook.current_version) + 1;
    let created_at = now_iso();

    let snapshot_path = snapshots::snapshot_package(
        &workspace_root,
        &package_root,
        package_id,
        next_version_number,
    )?;

    let version = PackageVersion {
        id: format!("version-{}-v{}", package_id, next_version_number),
        package_id: package_id.to_string(),
        version_number: next_version_number,
        note,
        snapshot_path: snapshot_path.clone(),
        eval_report_id: Some(eval_report_id),
        is_pinned: false,
        created_at: created_at.clone(),
    };

    notebook.current_version = next_version_number;
    notebook.updated_at = created_at;
    notebook.versions.insert(0, version.clone());

    notebook.versions.sort_by(|left, right| {
        right
            .version_number
            .cmp(&left.version_number)
            .then_with(|| right.created_at.cmp(&left.created_at))
    });

    let evicted = evict_versions(&workspace_root, &mut notebook.versions, FORMAL_VERSION_CAP)
        .map_err(|error| {
            let _ = snapshots::delete_snapshot(&workspace_root, &snapshot_path);
            error
        })?;

    filesystem::save_package_notebook(&package_root, &notebook).map_err(|error| {
        let _ = snapshots::delete_snapshot(&workspace_root, &snapshot_path);
        error
    })?;

    for removed in evicted {
        let _ = snapshots::delete_snapshot(&workspace_root, &removed.snapshot_path);
    }

    Ok(version)
}

fn evict_versions(
    workspace_root: &PathBuf,
    versions: &mut Vec<PackageVersion>,
    cap: usize,
) -> Result<Vec<PackageVersion>, String> {
    if versions.len() <= cap {
        return Ok(Vec::new());
    }

    let mut to_remove = versions.len() - cap;
    let mut evicted = Vec::new();

    while to_remove > 0 {
        if let Some(index) = (0..versions.len())
            .rev()
            .find(|&idx| !versions[idx].is_pinned)
        {
            evicted.push(versions.remove(index));
            to_remove -= 1;
        } else {
            return Err(format!(
                "cannot evict versions; all remaining versions are pinned (cap {})",
                cap
            ));
        }
    }

    // If something went very wrong and snapshot paths are unsafe, fail fast.
    for item in &evicted {
        if !item.snapshot_path.starts_with(".skill-notebook/snapshots") {
            return Err(format!(
                "unsafe snapshot path encountered while evicting: {}",
                item.snapshot_path
            ));
        }

        let absolute = workspace_root.join(&item.snapshot_path);
        if absolute.exists() && !absolute.starts_with(snapshots::snapshots_root(workspace_root)) {
            return Err(format!(
                "refusing to delete snapshot outside snapshots root: {}",
                absolute.display()
            ));
        }
    }

    Ok(evicted)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::services::version_service;
    use crate::storage::filesystem;

    fn tmp_workspace_root() -> PathBuf {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "skill-notebook-workspace-test-{}-{}",
            std::process::id(),
            seed
        ))
    }

    fn copy_example_workspace(destination: &PathBuf) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("examples")
            .join("workspace");
        filesystem::copy_directory_recursive(&root, destination).expect("copy workspace");
        destination.clone()
    }

    #[test]
    fn saves_a_new_formal_version_with_snapshot() {
        let workspace_root = tmp_workspace_root();
        let root = copy_example_workspace(&workspace_root);

        let saved = version_service::save_version("pkg-interview", Some("New release".to_string()), Some(root.to_string_lossy().as_ref()))
            .expect("save version");

        assert_eq!(saved.package_id, "pkg-interview");
        assert!(saved.version_number >= 4);
        assert!(saved.eval_report_id.is_some());

        let notebook_path = root
            .join("packages")
            .join("interview-insight-extractor")
            .join("notebook.json");
        let notebook: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&notebook_path).unwrap()).unwrap();
        assert_eq!(
            notebook["currentVersion"].as_u64().unwrap(),
            saved.version_number as u64
        );

        let snapshot_dir = root.join(saved.snapshot_path);
        assert!(snapshot_dir.exists());
        assert!(!snapshot_dir.join("notebook.json").exists());
        assert!(snapshot_dir.join("SKILL.md").exists());
    }

    #[test]
    fn refuses_to_save_without_eval_report() {
        let workspace_root = tmp_workspace_root();
        let root = copy_example_workspace(&workspace_root);

        let error =
            version_service::save_version("pkg-meeting", None, Some(root.to_string_lossy().as_ref()))
                .expect_err("should fail");
        assert!(error.contains("no eval report"));
    }
}
