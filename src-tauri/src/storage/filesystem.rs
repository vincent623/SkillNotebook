use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::domain::eval::EvalReport;
use crate::domain::package::{PackageNotebookDocument, SkillPackage};
use crate::domain::preview::PreviewModel;
use crate::domain::version::PackageVersion;
use crate::domain::workspace::Workspace;
use crate::utils::ids::slugify;

#[derive(Debug, Clone)]
pub struct ScannedWorkspace {
    pub workspace: Workspace,
    pub packages: Vec<SkillPackage>,
    pub eval_reports: Vec<EvalReport>,
    pub versions: Vec<PackageVersion>,
    pub previews: Vec<PreviewModel>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct WorkspaceConfigFile {
    id: Option<String>,
    name: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    last_opened_at: Option<String>,
}

pub fn default_workspace_root() -> PathBuf {
    if let Ok(value) = env::var("SKILL_NOTEBOOK_WORKSPACE") {
        return PathBuf::from(value);
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("examples")
        .join("workspace")
}

pub fn scan_workspace(root_path: Option<&str>) -> Result<ScannedWorkspace, String> {
    let workspace_root = resolve_workspace_root(root_path);
    let workspace = read_workspace(&workspace_root)?;
    let packages_root = workspace_root.join("packages");

    if !packages_root.exists() {
        return Err(format!(
            "workspace packages directory not found: {}",
            packages_root.display()
        ));
    }

    let mut packages = Vec::new();
    let mut eval_reports = Vec::new();
    let mut versions = Vec::new();
    let mut previews = Vec::new();

    for package_dir in list_directories(&packages_root)? {
        let notebook = load_package_notebook(&package_dir)?;
        let slug = package_dir
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled-package".to_string());

        let package_id = if notebook.id.trim().is_empty() {
            slugify(&slug)
        } else {
            notebook.id.clone()
        };

        let package = SkillPackage {
            id: package_id.clone(),
            workspace_id: workspace.id.clone(),
            slug,
            name: fallback_string(&notebook.name, "Untitled Package"),
            description: fallback_string(&notebook.description, "No description yet."),
            tags: notebook.tags.clone(),
            status: notebook.status.clone(),
            root_path: package_dir.to_string_lossy().to_string(),
            current_version: notebook.current_version,
            last_eval_status: notebook.last_eval_status.clone(),
            related_skills: notebook.related_skills.clone(),
            bundle_candidates: notebook.bundle_candidates.clone(),
            created_at: fallback_string(&notebook.created_at, "2026-04-13T00:00:00Z"),
            updated_at: fallback_string(&notebook.updated_at, "2026-04-13T00:00:00Z"),
        };

        let mut package_versions = notebook.versions.clone();
        for version in &mut package_versions {
            if version.package_id.trim().is_empty() {
                version.package_id = package_id.clone();
            }
        }

        let mut package_eval_reports = notebook.eval_reports.clone();
        for report in &mut package_eval_reports {
            if report.package_id.trim().is_empty() {
                report.package_id = package_id.clone();
            }
        }

        previews.push(build_preview(&package_dir, &package.id, &package.name)?);
        versions.extend(package_versions);
        eval_reports.extend(package_eval_reports);
        packages.push(package);
    }

    packages.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.name.cmp(&right.name))
    });

    versions.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.version_number.cmp(&left.version_number))
    });

    eval_reports.sort_by(|left, right| right.created_at.cmp(&left.created_at));

    Ok(ScannedWorkspace {
        workspace,
        packages,
        eval_reports,
        versions,
        previews,
    })
}

pub fn workspace_root_for_id(
    workspace_id: &str,
    root_path: Option<&str>,
) -> Result<PathBuf, String> {
    let scanned = scan_workspace(root_path)?;
    if scanned.workspace.id == workspace_id {
        Ok(PathBuf::from(scanned.workspace.root_path))
    } else {
        Err(format!(
            "workspace id mismatch: expected {}, found {}",
            workspace_id, scanned.workspace.id
        ))
    }
}

pub fn load_package_notebook(package_dir: &Path) -> Result<PackageNotebookDocument, String> {
    read_json_file(&package_dir.join("notebook.json"))
}

pub fn save_package_notebook(
    package_dir: &Path,
    notebook: &PackageNotebookDocument,
) -> Result<(), String> {
    write_json_file(&package_dir.join("notebook.json"), notebook)
}

pub fn ensure_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|error| format!("failed to create directory {}: {}", path.display(), error))
}

pub fn write_json_file<T>(path: &Path, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    let content = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {}", path.display(), error))?;
    write_text_file(path, &format!("{}\n", content))
}

pub fn write_text_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }

    fs::write(path, content)
        .map_err(|error| format!("failed to write {}: {}", path.display(), error))
}

pub fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    ensure_directory(destination)?;

    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read directory {}: {}", source.display(), error))?
    {
        let path = entry
            .map_err(|error| format!("failed to inspect directory entry: {}", error))?
            .path();
        let destination_path = destination.join(
            path.file_name()
                .ok_or_else(|| format!("missing file name for {}", path.display()))?,
        );

        if path.is_dir() {
            copy_directory_recursive(&path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                ensure_directory(parent)?;
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

fn resolve_workspace_root(root_path: Option<&str>) -> PathBuf {
    let path = root_path
        .map(PathBuf::from)
        .unwrap_or_else(default_workspace_root);

    fs::canonicalize(&path).unwrap_or(path)
}

fn read_workspace(root_path: &Path) -> Result<Workspace, String> {
    let config_path = root_path.join(".skill-notebook").join("config.json");
    let config: WorkspaceConfigFile = if config_path.exists() {
        read_json_file(&config_path)?
    } else {
        WorkspaceConfigFile::default()
    };

    let inferred_name = root_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "Skill Notebook Workspace".to_string());
    let name = config.name.unwrap_or_else(|| inferred_name.clone());

    Ok(Workspace {
        id: config
            .id
            .unwrap_or_else(|| format!("workspace-{}", slugify(&name))),
        name,
        root_path: root_path.to_string_lossy().to_string(),
        created_at: config
            .created_at
            .unwrap_or_else(|| "2026-04-13T00:00:00Z".to_string()),
        updated_at: config
            .updated_at
            .unwrap_or_else(|| "2026-04-13T00:00:00Z".to_string()),
        last_opened_at: config.last_opened_at,
    })
}

fn build_preview(
    package_dir: &Path,
    package_id: &str,
    package_name: &str,
) -> Result<PreviewModel, String> {
    let skill_path = package_dir.join("SKILL.md");
    let prompt_files = list_files(package_dir, "prompts")?;
    let example_files = list_files(package_dir, "examples")?;
    let reference_files = list_files(package_dir, "references")?;
    let script_files = list_files(package_dir, "scripts")?;
    let test_files = list_files(package_dir, "tests")?;

    let skill_md_preview = if skill_path.exists() {
        preview_text(&skill_path, 220)?
    } else {
        "No SKILL.md found yet.".to_string()
    };

    let example_preview = if let Some(first_example) = example_files.first() {
        preview_text(&package_dir.join(first_example), 180)?
    } else {
        "No example files yet.".to_string()
    };

    let final_preview = format!(
        "Package has {} prompt file(s), {} example file(s), {} reference file(s), and {} test file(s).",
        prompt_files.len(),
        example_files.len(),
        reference_files.len(),
        test_files.len()
    );

    Ok(PreviewModel {
        package_id: package_id.to_string(),
        name: package_name.to_string(),
        has_skill_md: skill_path.exists(),
        prompt_files,
        example_files,
        reference_files,
        script_files,
        test_files,
        skill_md_preview,
        example_preview,
        final_preview,
    })
}

fn list_directories(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut directories = fs::read_dir(root)
        .map_err(|error| format!("failed to read directory {}: {}", root.display(), error))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    directories.sort_by(|left, right| {
        left.file_name()
            .map(|value| value.to_string_lossy().to_string())
            .cmp(
                &right
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string()),
            )
    });

    Ok(directories)
}

fn list_files(package_root: &Path, subdir: &str) -> Result<Vec<String>, String> {
    let dir = package_root.join(subdir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_files(package_root, &dir, &mut files)?;
    files.sort();

    Ok(files)
}

fn collect_files(
    package_root: &Path,
    current: &Path,
    files: &mut Vec<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("failed to read directory {}: {}", current.display(), error))?
    {
        let path = entry
            .map_err(|error| format!("failed to inspect directory entry: {}", error))?
            .path();

        if path.is_dir() {
            collect_files(package_root, &path, files)?;
            continue;
        }

        let relative = path
            .strip_prefix(package_root)
            .map_err(|error| format!("failed to compute relative path: {}", error))?
            .to_string_lossy()
            .to_string();
        files.push(relative);
    }

    Ok(())
}

fn preview_text(path: &Path, limit: usize) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read preview file {}: {}", path.display(), error))?;
    Ok(truncate_preview(&content, limit))
}

fn truncate_preview(content: &str, limit: usize) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut truncated = compact.chars().take(limit).collect::<String>();

    if compact.chars().count() > limit {
        truncated.push_str("...");
    }

    truncated
}

fn fallback_string(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn read_json_file<T>(path: &Path) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {}", path.display(), error))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("failed to parse {}: {}", path.display(), error))
}

#[cfg(test)]
mod tests {
    use super::scan_workspace;

    #[test]
    fn scans_the_default_example_workspace() {
        let scanned = scan_workspace(None).expect("default workspace should scan");

        assert_eq!(scanned.workspace.id, "workspace-main");
        assert_eq!(scanned.packages.len(), 3);
        assert!(scanned
            .previews
            .iter()
            .any(|preview| preview.package_id == "pkg-interview"));
    }

    #[test]
    fn reads_versions_from_package_notebook_files() {
        let scanned = scan_workspace(None).expect("default workspace should scan");

        assert!(scanned
            .versions
            .iter()
            .any(|version| version.package_id == "pkg-pdf" && version.version_number == 1));
    }
}
