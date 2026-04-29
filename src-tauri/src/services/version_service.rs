use std::cmp::max;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use crate::domain::eval::{EvalOverallStatus, EvalReport};
use crate::domain::package::{PackageStatus, SkillPackage};
use crate::domain::version::{
    PackageVersion, PackageVersionDiff, VersionDiffChangeType, VersionDiffEntry,
};
use crate::storage::filesystem;
use crate::storage::snapshots;
use crate::utils::diff::diff_text;
use crate::utils::time::now_iso;

const FORMAL_VERSION_CAP: usize = 10;

pub fn list_versions(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Vec<PackageVersion>, String> {
    let mut versions = filesystem::scan_project_root(root_path)?
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
    let scanned = filesystem::scan_project_root(root_path)?;
    let package = scanned
        .packages
        .iter()
        .find(|item| item.id == package_id)
        .cloned()
        .ok_or_else(|| format!("package not found: {}", package_id))?;

    let project_root_path = PathBuf::from(&scanned.project_root.root_path);
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
        &project_root_path,
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

    let evicted = evict_versions(
        &project_root_path,
        &mut notebook.versions,
        FORMAL_VERSION_CAP,
    )
    .map_err(|error| {
        let _ = snapshots::delete_snapshot(&project_root_path, &snapshot_path);
        error
    })?;

    filesystem::save_package_notebook(&package_root, &notebook).map_err(|error| {
        let _ = snapshots::delete_snapshot(&project_root_path, &snapshot_path);
        error
    })?;

    for removed in evicted {
        let _ = snapshots::delete_snapshot(&project_root_path, &removed.snapshot_path);
    }

    Ok(version)
}

pub fn diff_version(
    version_id: &str,
    root_path: Option<&str>,
) -> Result<PackageVersionDiff, String> {
    let context = load_version_context(version_id, root_path)?;
    if !snapshots::snapshot_has_restorable_content(
        &context.project_root_path,
        &context.version.snapshot_path,
    )? {
        return Err(format!(
            "version {} was saved before restorable snapshots were captured; save a new version to compare",
            version_id
        ));
    }

    let snapshot_root = context
        .project_root_path
        .join(&context.version.snapshot_path);
    let current_files = snapshots::collect_snapshot_files(&context.package_root)?;
    let version_files = snapshots::collect_snapshot_files(&snapshot_root)?;
    let current_map = current_files
        .into_iter()
        .map(|path| {
            let content = read_optional_text_file(&context.package_root.join(&path))?;
            Ok((path, content))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    let version_map = version_files
        .into_iter()
        .map(|path| {
            let content = read_optional_text_file(&snapshot_root.join(&path))?;
            Ok((path, content))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;

    let mut all_paths = BTreeSet::new();
    all_paths.extend(current_map.keys().cloned());
    all_paths.extend(version_map.keys().cloned());

    let mut entries = Vec::new();
    for path in all_paths {
        let current_content = current_map.get(&path).cloned().flatten();
        let version_content = version_map.get(&path).cloned().flatten();

        let change_type = match (version_content.as_ref(), current_content.as_ref()) {
            (Some(previous), Some(current)) if previous == current => None,
            (Some(_), Some(_)) => Some(VersionDiffChangeType::Modified),
            (Some(_), None) => Some(VersionDiffChangeType::Removed),
            (None, Some(_)) => Some(VersionDiffChangeType::Added),
            (None, None) => None,
        };

        if let Some(change_type) = change_type {
            let diff_output = match (&version_content, &current_content) {
                (Some(previous), Some(current)) => diff_text(previous, current),
                (Some(previous), None) => diff_text(previous, ""),
                (None, Some(current)) => diff_text("", current),
                (None, None) => String::new(),
            };

            entries.push(VersionDiffEntry {
                path,
                change_type,
                diff_text: diff_output,
            });
        }
    }

    Ok(PackageVersionDiff {
        version_id: context.version.id,
        package_id: context.package.id,
        version_number: context.version.version_number,
        snapshot_path: context.version.snapshot_path,
        entries,
    })
}

pub fn restore_version(version_id: &str, root_path: Option<&str>) -> Result<SkillPackage, String> {
    let context = load_version_context(version_id, root_path)?;
    snapshots::restore_snapshot(
        &context.project_root_path,
        &context.package_root,
        &context.version.snapshot_path,
    )?;

    let mut notebook = filesystem::load_package_notebook(&context.package_root)?;
    notebook.current_version = context.version.version_number;
    notebook.updated_at = now_iso();

    let linked_report = context
        .version
        .eval_report_id
        .as_ref()
        .and_then(|report_id| {
            notebook
                .eval_reports
                .iter()
                .find(|item| item.id == *report_id)
        })
        .cloned();

    if let Some(report) = linked_report {
        notebook.last_eval_status = Some(report.overall_status.clone());
        notebook.status = status_from_eval_report(&report);
    } else {
        notebook.last_eval_status = None;
        notebook.status = PackageStatus::NeedsEval;
    }

    filesystem::save_package_notebook(&context.package_root, &notebook)?;

    filesystem::scan_project_root(Some(context.project_root_path.to_string_lossy().as_ref()))?
        .packages
        .into_iter()
        .find(|item| item.id == context.package.id)
        .ok_or_else(|| format!("package not found after restore: {}", context.package.id))
}

#[derive(Debug, Clone)]
struct VersionContext {
    project_root_path: PathBuf,
    package_root: PathBuf,
    package: SkillPackage,
    version: PackageVersion,
}

fn load_version_context(
    version_id: &str,
    root_path: Option<&str>,
) -> Result<VersionContext, String> {
    let scanned = filesystem::scan_project_root(root_path)?;
    let version = scanned
        .versions
        .iter()
        .find(|item| item.id == version_id)
        .cloned()
        .ok_or_else(|| format!("version not found: {}", version_id))?;
    let package = scanned
        .packages
        .iter()
        .find(|item| item.id == version.package_id)
        .cloned()
        .ok_or_else(|| format!("package not found for version: {}", version.package_id))?;

    Ok(VersionContext {
        project_root_path: PathBuf::from(&scanned.project_root.root_path),
        package_root: PathBuf::from(&package.root_path),
        package,
        version,
    })
}

fn read_optional_text_file(path: &PathBuf) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            Ok(Some("<<binary or non-utf8 content omitted>>".to_string()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("failed to read {}: {}", path.display(), error)),
    }
}

fn status_from_eval_report(report: &EvalReport) -> PackageStatus {
    match report.overall_status {
        EvalOverallStatus::Usable => PackageStatus::Validated,
        EvalOverallStatus::NeedsImprovement => PackageStatus::NeedsEval,
        EvalOverallStatus::Problematic => PackageStatus::Draft,
    }
}

fn evict_versions(
    project_root_path: &PathBuf,
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

        let absolute = project_root_path.join(&item.snapshot_path);
        if absolute.exists() && !absolute.starts_with(snapshots::snapshots_root(project_root_path))
        {
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

    use crate::domain::package::{PackageNotebookDocument, PackageStatus};
    use crate::services::version_service;
    use crate::storage::filesystem;

    fn tmp_project_root_path() -> PathBuf {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "skill-notebook-project_root-test-{}-{}",
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
    fn saves_a_new_formal_version_with_snapshot() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);

        let saved = version_service::save_version(
            "pkg-interview",
            Some("New release".to_string()),
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("save version");

        assert_eq!(saved.package_id, "pkg-interview");
        assert!(saved.version_number >= 4);
        assert!(saved.eval_report_id.is_some());

        let notebook_path = filesystem::canonical_skills_root(&root)
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
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let package_root = filesystem::canonical_skills_root(&root).join("no-eval-skill");

        filesystem::ensure_directory(&package_root).expect("create package dir");
        filesystem::save_package_notebook(
            &package_root,
            &PackageNotebookDocument {
                id: "pkg-no-eval".to_string(),
                name: "No Eval Skill".to_string(),
                description: "Package without eval reports.".to_string(),
                status: PackageStatus::Draft,
                ..PackageNotebookDocument::default()
            },
        )
        .expect("write notebook");

        let error = version_service::save_version(
            "pkg-no-eval",
            None,
            Some(root.to_string_lossy().as_ref()),
        )
        .expect_err("should fail");
        assert!(error.contains("no eval report"));
    }

    #[test]
    fn diffs_against_a_saved_version_snapshot() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let saved = version_service::save_version(
            "pkg-interview",
            Some("diff baseline".to_string()),
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("save version");
        let task_path = filesystem::canonical_skills_root(&root)
            .join("interview-insight-extractor")
            .join("prompts")
            .join("task.md");
        std::fs::write(&task_path, "changed line\nsecond line\n").expect("rewrite task");

        let diff = version_service::diff_version(&saved.id, Some(root.to_string_lossy().as_ref()))
            .expect("version diff");

        assert!(diff
            .entries
            .iter()
            .any(|entry| entry.path == "prompts/task.md"
                && entry.diff_text.contains("+ changed line")));
    }

    #[test]
    fn restores_package_files_from_a_saved_snapshot() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let saved = version_service::save_version(
            "pkg-interview",
            Some("restore baseline".to_string()),
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("save version");
        let task_path = filesystem::canonical_skills_root(&root)
            .join("interview-insight-extractor")
            .join("prompts")
            .join("task.md");
        std::fs::write(&task_path, "locally changed\n").expect("rewrite task");

        let restored =
            version_service::restore_version(&saved.id, Some(root.to_string_lossy().as_ref()))
                .expect("restore version");

        let restored_content = std::fs::read_to_string(&task_path).expect("read restored file");
        assert!(restored_content.contains("Extract user pain points"));
        assert_eq!(restored.current_version, saved.version_number);
    }
}
