use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::domain::eval::EvalReport;
use crate::domain::package::{
    PackageFileContent, PackageFileEntry, PackageNotebookDocument, SkillPackage,
};
use crate::domain::preview::PreviewModel;
use crate::domain::project_root::ProjectRoot;
use crate::domain::version::PackageVersion;
use crate::utils::errors::AppError;
use crate::utils::ids::slugify;

#[derive(Debug, Clone)]
pub struct ScannedProjectRoot {
    pub project_root: ProjectRoot,
    pub packages: Vec<SkillPackage>,
    pub eval_reports: Vec<EvalReport>,
    pub versions: Vec<PackageVersion>,
    pub previews: Vec<PreviewModel>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct ProjectRootConfigFile {
    id: Option<String>,
    name: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    last_opened_at: Option<String>,
}

const CANONICAL_SKILLS_DIR: &str = ".skills";

pub fn default_project_root() -> PathBuf {
    if let Ok(value) = env::var("SKILL_NOTEBOOK_PROJECT_ROOT") {
        return PathBuf::from(value);
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("examples")
        .join("project-root")
}

pub fn scan_project_root(root_path: Option<&str>) -> Result<ScannedProjectRoot, AppError> {
    let project_root_path = resolve_project_root(root_path);
    let project_root = read_project_root(&project_root_path)?;
    let skills_root = locate_skills_root(&project_root_path)?;

    let mut packages = Vec::new();
    let mut eval_reports = Vec::new();
    let mut versions = Vec::new();
    let mut previews = Vec::new();

    for package_dir in list_directories(&skills_root)? {
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
            project_root_id: project_root.id.clone(),
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

    Ok(ScannedProjectRoot {
        project_root,
        packages,
        eval_reports,
        versions,
        previews,
    })
}

pub fn project_root_for_id(
    project_root_id: &str,
    root_path: Option<&str>,
) -> Result<PathBuf, AppError> {
    let scanned = scan_project_root(root_path)?;
    if scanned.project_root.id == project_root_id {
        Ok(PathBuf::from(scanned.project_root.root_path))
    } else {
        Err(AppError::Other(format!(
            "project root id mismatch: expected {}, found {}",
            project_root_id, scanned.project_root.id
        )))
    }
}

pub fn canonical_skills_root(project_root_path: &Path) -> PathBuf {
    project_root_path.join(CANONICAL_SKILLS_DIR)
}

pub fn find_package_root_by_id(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<PathBuf, AppError> {
    let project_root_path = resolve_project_root(root_path);
    let skills_root = locate_skills_root(&project_root_path)?;

    for entry in fs::read_dir(&skills_root)? {
        let dir = entry?.path();
        if !dir.is_dir() {
            continue;
        }
        let notebook_path = dir.join("notebook.json");
        if !notebook_path.exists() {
            continue;
        }
        let notebook: PackageNotebookDocument = read_json_file(&notebook_path)?;
        let slug = dir
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default();
        let resolved_id = if notebook.id.trim().is_empty() {
            slugify(&slug)
        } else {
            notebook.id.clone()
        };
        if resolved_id == package_id {
            return Ok(dir);
        }
    }

    Err(AppError::NotFound {
        entity: "package".into(),
        identifier: package_id.into(),
    })
}

pub fn locate_skills_root(project_root_path: &Path) -> Result<PathBuf, AppError> {
    let canonical = canonical_skills_root(project_root_path);
    if canonical.exists() {
        return Ok(canonical);
    }

    Err(AppError::Other(format!(
        "skill root not found under {}. Expected {}/.",
        project_root_path.display(),
        CANONICAL_SKILLS_DIR
    )))
}

pub fn load_package_notebook(package_dir: &Path) -> Result<PackageNotebookDocument, AppError> {
    read_json_file(&package_dir.join("notebook.json"))
}

pub fn save_package_notebook(
    package_dir: &Path,
    notebook: &PackageNotebookDocument,
) -> Result<(), AppError> {
    write_json_file(&package_dir.join("notebook.json"), notebook)
}

pub fn ensure_directory(path: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn write_json_file<T>(path: &Path, value: &T) -> Result<(), AppError>
where
    T: Serialize,
{
    let content = serde_json::to_string_pretty(value)?;
    write_text_file(path, &format!("{}\n", content))
}

pub fn write_text_file(path: &Path, content: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

pub fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<(), AppError> {
    ensure_directory(destination)?;

    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        let destination_path = destination.join(
            path.file_name()
                .ok_or_else(|| AppError::Other(format!("missing file name for {}", path.display())))?,
        );

        if path.is_dir() {
            copy_directory_recursive(&path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                ensure_directory(parent)?;
            }

            fs::copy(&path, &destination_path)?;
        }
    }

    Ok(())
}

pub fn list_package_file_tree(package_root: &Path) -> Result<Vec<PackageFileEntry>, AppError> {
    build_package_file_entries(package_root, package_root)
}

pub fn read_package_text_file(
    package_root: &Path,
    relative_path: &str,
) -> Result<PackageFileContent, AppError> {
    let (normalized_path, absolute_path) = resolve_package_text_path(package_root, relative_path)?;
    let metadata = fs::symlink_metadata(&absolute_path)?;

    if metadata.file_type().is_symlink() {
        return Err(AppError::Other(format!(
            "refusing to read symlinked file inside package: {}",
            normalized_path
        )));
    }

    if !metadata.is_file() {
        return Err(AppError::Other(format!("package path is not a file: {}", normalized_path)));
    }

    let content = fs::read_to_string(&absolute_path)?;

    Ok(PackageFileContent {
        path: normalized_path,
        content,
        encoding: "utf-8".to_string(),
    })
}

pub fn write_package_text_file(
    package_root: &Path,
    relative_path: &str,
    content: &str,
) -> Result<PackageFileContent, AppError> {
    let (normalized_path, absolute_path) = resolve_package_text_path(package_root, relative_path)?;

    if let Some(parent) = absolute_path.parent() {
        ensure_directory(parent)?;
    }

    fs::write(&absolute_path, content)?;

    Ok(PackageFileContent {
        path: normalized_path,
        content: content.to_string(),
        encoding: "utf-8".to_string(),
    })
}

fn resolve_project_root(root_path: Option<&str>) -> PathBuf {
    let path = root_path
        .map(PathBuf::from)
        .unwrap_or_else(default_project_root);

    fs::canonicalize(&path).unwrap_or(path)
}

fn build_package_file_entries(
    package_root: &Path,
    current: &Path,
) -> Result<Vec<PackageFileEntry>, AppError> {
    let mut entries = fs::read_dir(current)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if should_skip_package_entry(&name) {
                return None;
            }

            Some((name, path))
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        let left_is_dir = left.1.is_dir();
        let right_is_dir = right.1.is_dir();

        right_is_dir
            .cmp(&left_is_dir)
            .then_with(|| left.0.to_lowercase().cmp(&right.0.to_lowercase()))
    });

    let mut tree = Vec::new();

    for (name, path) in entries {
        let metadata = fs::symlink_metadata(&path)?;

        if metadata.file_type().is_symlink() {
            continue;
        }

        let relative = path
            .strip_prefix(package_root)?
            .to_string_lossy()
            .to_string();

        if metadata.is_dir() {
            let children = build_package_file_entries(package_root, &path)?;
            tree.push(PackageFileEntry {
                path: relative,
                name,
                is_directory: true,
                children: Some(children),
            });
        } else if metadata.is_file() {
            tree.push(PackageFileEntry {
                path: relative,
                name,
                is_directory: false,
                children: None,
            });
        }
    }

    Ok(tree)
}

fn should_skip_package_entry(name: &str) -> bool {
    name.starts_with('.') || name == "notebook.json"
}

fn resolve_package_text_path(
    package_root: &Path,
    relative_path: &str,
) -> Result<(String, PathBuf), AppError> {
    let cleaned = sanitize_package_relative_path(relative_path)?;
    let normalized = cleaned.to_string_lossy().to_string();
    let absolute = package_root.join(&cleaned);

    Ok((normalized, absolute))
}

fn sanitize_package_relative_path(relative_path: &str) -> Result<PathBuf, AppError> {
    let raw = relative_path.trim();
    if raw.is_empty() {
        return Err(AppError::Other("package file path cannot be empty".to_string()));
    }

    let candidate = Path::new(raw);
    if candidate.is_absolute() {
        return Err(AppError::InvalidPath(format!("absolute paths are not allowed: {}", raw)));
    }

    let mut cleaned = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => {
                let value = part.to_string_lossy().to_string();
                if should_skip_package_entry(&value) {
                    return Err(AppError::InvalidPath(format!("package file is not editable: {}", raw)));
                }
                cleaned.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(AppError::InvalidPath(format!(
                    "parent directory traversal is not allowed: {}",
                    raw
                )))
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::InvalidPath(format!("absolute paths are not allowed: {}", raw)))
            }
        }
    }

    if cleaned.as_os_str().is_empty() {
        return Err(AppError::InvalidPath(format!("invalid package file path: {}", raw)));
    }

    Ok(cleaned)
}

fn read_project_root(root_path: &Path) -> Result<ProjectRoot, AppError> {
    let config_path = root_path.join(".skill-notebook").join("config.json");
    let config: ProjectRootConfigFile = if config_path.exists() {
        read_json_file(&config_path)?
    } else {
        ProjectRootConfigFile::default()
    };

    let inferred_name = root_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "Skill Notebook Project Root".to_string());
    let name = config.name.unwrap_or_else(|| inferred_name.clone());

    Ok(ProjectRoot {
        id: config
            .id
            .unwrap_or_else(|| format!("project-root-{}", slugify(&name))),
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
) -> Result<PreviewModel, AppError> {
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

fn list_directories(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut directories = fs::read_dir(root)?
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

fn list_files(package_root: &Path, subdir: &str) -> Result<Vec<String>, AppError> {
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
) -> Result<(), AppError> {
    for entry in fs::read_dir(current)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_files(package_root, &path, files)?;
            continue;
        }

        let relative = path
            .strip_prefix(package_root)?
            .to_string_lossy()
            .to_string();
        files.push(relative);
    }

    Ok(())
}

fn preview_text(path: &Path, limit: usize) -> Result<String, AppError> {
    let content = fs::read_to_string(path)?;
    Ok(truncate_preview(&content, limit))
}

fn truncate_preview(content: &str, limit: usize) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let taken: String = compact.chars().take(limit).collect();
    if compact.chars().count() > limit {
        format!("{taken}...")
    } else {
        taken
    }
}

fn fallback_string(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn read_json_file<T>(path: &Path) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use super::{
        list_package_file_tree, read_package_text_file, scan_project_root, write_package_text_file,
    };
    use std::path::PathBuf;

    use crate::storage::filesystem;
    use crate::test_helpers::{copy_example_project_root, tmp_project_root_path};

    #[test]
    fn scans_the_default_example_workspace() {
        let scanned = scan_project_root(None).expect("default project_root should scan");

        assert_eq!(scanned.project_root.id, "project-root-main");
        assert!(scanned
            .packages
            .iter()
            .any(|package| package.id == "pkg-interview"));
        assert!(scanned
            .packages
            .iter()
            .any(|package| package.id == "pkg-meeting"));
        assert!(scanned
            .packages
            .iter()
            .any(|package| package.id == "pkg-pdf"));
        assert!(scanned
            .previews
            .iter()
            .any(|preview| preview.package_id == "pkg-interview"));
    }

    #[test]
    fn reads_versions_from_package_notebook_files() {
        let scanned = scan_project_root(None).expect("default project_root should scan");

        assert!(scanned
            .versions
            .iter()
            .any(|version| version.package_id == "pkg-pdf" && version.version_number == 1));
    }

    #[test]
    fn lists_visible_package_files_as_a_tree() {
        let scanned = scan_project_root(None).expect("default project_root should scan");
        let package_root = scanned
            .packages
            .iter()
            .find(|item| item.id == "pkg-interview")
            .map(|item| PathBuf::from(&item.root_path))
            .expect("pkg-interview should exist");

        let tree = list_package_file_tree(&package_root).expect("tree should build");

        assert!(tree.iter().any(|entry| entry.path == "SKILL.md"));
        assert!(tree
            .iter()
            .any(|entry| entry.path == "prompts" && entry.is_directory));
        assert!(!tree.iter().any(|entry| entry.path == "notebook.json"));
    }

    #[test]
    fn reads_and_writes_package_text_files() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let package_root =
            filesystem::canonical_skills_root(&root).join("interview-insight-extractor");

        let original = read_package_text_file(&package_root, "prompts/task.md")
            .expect("should read package file");
        assert!(original.content.contains("Extract user pain points"));

        let updated = write_package_text_file(
            &package_root,
            "prompts/task.md",
            "# updated\n\ncontent from test\n",
        )
        .expect("should write package file");
        assert_eq!(updated.path, "prompts/task.md");

        let reread = read_package_text_file(&package_root, "prompts/task.md")
            .expect("should re-read updated file");
        assert_eq!(reread.content, "# updated\n\ncontent from test\n");
    }

    #[test]
    fn rejects_hidden_or_traversal_package_paths() {
        let scanned = scan_project_root(None).expect("default project_root should scan");
        let package_root = scanned
            .packages
            .iter()
            .find(|item| item.id == "pkg-interview")
            .map(|item| PathBuf::from(&item.root_path))
            .expect("pkg-interview should exist");

        let traversal = read_package_text_file(&package_root, "../outside.txt")
            .expect_err("traversal path should fail");
        assert!(traversal.to_string().contains("traversal"));

        let hidden = read_package_text_file(&package_root, "notebook.json")
            .expect_err("metadata file should be blocked");
        assert!(hidden.to_string().contains("not editable"));
    }
}
