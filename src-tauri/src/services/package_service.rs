use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::domain::package::{
    PackageFileContent, PackageFileEntry, PackageImportRequest, PackageImportResponse,
    PackageNotebookDocument, PackageReferenceItem, PackageReferenceItemKind,
    PackageReferenceResponse, PackageStatus, PackageUpdateRequest, SkillPackage,
};
use crate::services::eval_service::{self, EvalParams};
use crate::storage::filesystem;
use crate::utils::errors::AppError;
use crate::utils::ids::slugify;
use crate::utils::time::now_iso;

pub fn list_packages(root_path: Option<&str>) -> Result<Vec<SkillPackage>, AppError> {
    Ok(filesystem::scan_project_root(root_path)?.packages)
}

pub fn get_package(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Option<SkillPackage>, AppError> {
    let package = filesystem::scan_project_root(root_path)?
        .packages
        .into_iter()
        .find(|item| item.id == package_id);

    Ok(package)
}

pub fn list_package_files(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Vec<PackageFileEntry>, AppError> {
    let package_root = package_root(package_id, root_path)?;
    filesystem::list_package_file_tree(&package_root)
}

pub fn read_package_file(
    package_id: &str,
    path: &str,
    root_path: Option<&str>,
) -> Result<PackageFileContent, AppError> {
    let package_root = package_root(package_id, root_path)?;
    filesystem::read_package_text_file(&package_root, path)
}

pub fn write_package_file(
    package_id: &str,
    path: &str,
    content: &str,
    root_path: Option<&str>,
) -> Result<PackageFileContent, AppError> {
    let package_root = package_root(package_id, root_path)?;
    filesystem::write_package_text_file(&package_root, path, content)
}

pub fn update_package(
    package_id: &str,
    payload: &PackageUpdateRequest,
    root_path: Option<&str>,
) -> Result<SkillPackage, AppError> {
    let package = get_package(package_id, root_path)?
        .ok_or_else(|| AppError::Other(format!("package not found: {}", package_id)))?;
    let package_root = PathBuf::from(&package.root_path);
    let mut notebook = filesystem::load_package_notebook(&package_root)?;

    if let Some(name) = payload.name.as_deref() {
        let next_name = name.trim();
        if next_name.is_empty() {
            return Err(AppError::Other("package name cannot be empty".to_string()));
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
        AppError::Other(format!(
            "package {} was updated but could not be reloaded from {}",
            package_id,
            package_root.display()
        ))
    })
}

pub fn reference_package(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<PackageReferenceResponse, AppError> {
    let scanned = filesystem::scan_project_root(root_path)?;
    let package = scanned
        .packages
        .into_iter()
        .find(|item| item.id == package_id)
        .ok_or_else(|| AppError::Other(format!("package not found: {}", package_id)))?;
    let project_root = scanned.project_root;
    let package_path = package.root_path.clone();
    let skill_md_path = PathBuf::from(&package_path)
        .join("SKILL.md")
        .to_string_lossy()
        .to_string();
    let project_claude_dir = PathBuf::from(&project_root.root_path)
        .join(".claude")
        .join("skills");
    let project_claude_target = project_claude_dir.join(&package.slug);
    let markdown_reference = format!(
        "Use the `{}` skill package at `{}`.",
        package.slug, skill_md_path
    );
    let cli_reference = format!(
        "skill --project_root {} reference {}",
        shell_quote(&project_root.root_path),
        package.id
    );
    let terminal_command = format!("cd {}", shell_quote(&package_path));

    let items = vec![
        PackageReferenceItem {
            id: "package-path".to_string(),
            label: "Package path".to_string(),
            value: package_path.clone(),
            kind: PackageReferenceItemKind::Path,
        },
        PackageReferenceItem {
            id: "skill-md-path".to_string(),
            label: "SKILL.md path".to_string(),
            value: skill_md_path.clone(),
            kind: PackageReferenceItemKind::Path,
        },
        PackageReferenceItem {
            id: "markdown-reference".to_string(),
            label: "Markdown reference".to_string(),
            value: markdown_reference,
            kind: PackageReferenceItemKind::Snippet,
        },
        PackageReferenceItem {
            id: "cli-reference".to_string(),
            label: "CLI reference".to_string(),
            value: cli_reference,
            kind: PackageReferenceItemKind::Snippet,
        },
        PackageReferenceItem {
            id: "terminal-command".to_string(),
            label: "Open package in terminal".to_string(),
            value: terminal_command,
            kind: PackageReferenceItemKind::Command,
        },
        PackageReferenceItem {
            id: "global-claude-link".to_string(),
            label: "Link to global Claude skills".to_string(),
            value: format!(
                "mkdir -p ~/.claude/skills && ln -sfn {} ~/.claude/skills/{}",
                shell_quote(&package_path),
                package.slug
            ),
            kind: PackageReferenceItemKind::Command,
        },
        PackageReferenceItem {
            id: "project-claude-link".to_string(),
            label: "Link to project Claude skills".to_string(),
            value: format!(
                "mkdir -p {} && ln -sfn {} {}",
                shell_quote(project_claude_dir.to_string_lossy().as_ref()),
                shell_quote(&package_path),
                shell_quote(project_claude_target.to_string_lossy().as_ref())
            ),
            kind: PackageReferenceItemKind::Command,
        },
    ];

    Ok(PackageReferenceResponse {
        package_id: package.id,
        slug: package.slug,
        package_path,
        skill_md_path,
        items,
    })
}

pub fn import_package(
    payload: &PackageImportRequest,
    root_path: Option<&str>,
) -> Result<PackageImportResponse, AppError> {
    let project_root_path = filesystem::project_root_for_id(&payload.project_root_id, root_path)?;
    import_package_from_path(
        &project_root_path,
        &payload.source_path,
        payload.slug.as_deref(),
        payload.run_eval.unwrap_or(true),
    )
}

pub fn import_package_from_path(
    project_root_path: &Path,
    source_path: &str,
    requested_slug: Option<&str>,
    run_eval: bool,
) -> Result<PackageImportResponse, AppError> {
    let source = resolve_source_path(project_root_path, source_path)?;
    let metadata = fs::symlink_metadata(&source)?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::Other(format!(
            "refusing to import symlinked path: {}",
            source.display()
        )));
    }
    if !metadata.is_dir() {
        return Err(AppError::Other(format!(
            "import source must be a directory: {}",
            source.display()
        )));
    }
    if !source.join("SKILL.md").exists() {
        return Err(AppError::Other(format!(
            "import source has no SKILL.md: {}",
            source.display()
        )));
    }

    let skills_root = filesystem::canonical_skills_root(project_root_path);
    filesystem::ensure_directory(&skills_root)?;
    let base_slug = requested_slug
        .map(slugify)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            source
                .file_name()
                .map(|value| slugify(&value.to_string_lossy()))
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "imported-skill".to_string())
        });
    let slug = unique_slug(&skills_root, &base_slug);
    let package_root = skills_root.join(&slug);

    copy_import_package(&source, &package_root)?;

    let imported_at = now_iso();
    let metadata = infer_package_metadata(&package_root, &slug)?;
    let package_id = format!("pkg-{}", slug);
    let mut notebook = PackageNotebookDocument {
        id: package_id.clone(),
        name: metadata.name,
        description: metadata.description,
        tags: metadata.tags,
        status: PackageStatus::Draft,
        current_version: 0,
        last_eval_status: None,
        related_skills: Vec::new(),
        bundle_candidates: Vec::new(),
        created_at: imported_at.clone(),
        updated_at: imported_at.clone(),
        versions: Vec::new(),
        eval_reports: Vec::new(),
    };
    filesystem::save_package_notebook(&package_root, &notebook)?;

    let eval_report = if run_eval {
        let evaluation = eval_service::evaluate_package(&EvalParams {
            project_root_path,
            package_root: &package_root,
            package_id: &package_id,
            slug: &slug,
            package_name: &notebook.name,
            fallback_description: &notebook.description,
            iteration: 1,
        })?;
        notebook.last_eval_status = Some(evaluation.report.overall_status.clone());
        notebook.status = evaluation.suggested_status;
        notebook.updated_at = now_iso();
        notebook.eval_reports.insert(0, evaluation.report.clone());
        filesystem::save_package_notebook(&package_root, &notebook)?;
        Some(evaluation.report)
    } else {
        None
    };

    let package_path = package_root.to_string_lossy().to_string();
    Ok(PackageImportResponse {
        package_id: package_id.clone(),
        slug: slug.clone(),
        package_path,
        eval_report,
        eval_command: format!(
            "skill --project_root {} eval {}",
            shell_quote(project_root_path.to_string_lossy().as_ref()),
            package_id
        ),
        version_command: format!(
            "skill --project_root {} version save {}",
            shell_quote(project_root_path.to_string_lossy().as_ref()),
            package_id
        ),
        reference_command: format!(
            "skill --project_root {} reference {}",
            shell_quote(project_root_path.to_string_lossy().as_ref()),
            package_id
        ),
        imported_at,
    })
}

fn package_root(package_id: &str, root_path: Option<&str>) -> Result<PathBuf, AppError> {
    filesystem::find_package_root_by_id(package_id, root_path)
}

#[derive(Debug, Clone)]
struct InferredPackageMetadata {
    name: String,
    description: String,
    tags: Vec<String>,
}

fn resolve_source_path(project_root_path: &Path, source_path: &str) -> Result<PathBuf, AppError> {
    let trimmed = source_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::Other("import source path cannot be empty".to_string()));
    }
    let candidate = PathBuf::from(trimmed);
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        project_root_path.join(candidate)
    };
    Ok(fs::canonicalize(&resolved).unwrap_or(resolved))
}

fn unique_slug(skills_root: &Path, base_slug: &str) -> String {
    let root_slug = if base_slug.trim().is_empty() {
        "imported-skill"
    } else {
        base_slug
    };
    if !skills_root.join(root_slug).exists() {
        return root_slug.to_string();
    }

    for index in 2..1000 {
        let candidate = format!("{}-{}", root_slug, index);
        if !skills_root.join(&candidate).exists() {
            return candidate;
        }
    }

    format!(
        "{}-{}",
        root_slug,
        now_iso().replace([':', '.', 'T', 'Z'], "-")
    )
}

fn copy_import_package(source: &Path, destination: &Path) -> Result<(), AppError> {
    filesystem::ensure_directory(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "notebook.json" || name == "draft.json" {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let destination_path = destination.join(&name);
        if metadata.is_dir() {
            copy_import_package(&path, &destination_path)?;
        } else if metadata.is_file() {
            if let Some(parent) = destination_path.parent() {
                filesystem::ensure_directory(parent)?;
            }
            fs::copy(&path, &destination_path)?;
        }
    }
    Ok(())
}

fn infer_package_metadata(
    package_root: &Path,
    slug: &str,
) -> Result<InferredPackageMetadata, AppError> {
    let skill_md_path = package_root.join("SKILL.md");
    let content = fs::read_to_string(&skill_md_path)?;
    let frontmatter = crate::utils::frontmatter::extract(&content);
    let name = frontmatter
        .as_deref()
        .and_then(|block| crate::utils::frontmatter::get_value(block, "name"))
        .or_else(|| extract_markdown_title(&content))
        .unwrap_or_else(|| title_from_slug(slug));
    let description = frontmatter
        .as_deref()
        .and_then(|block| crate::utils::frontmatter::get_value(block, "description"))
        .or_else(|| first_body_paragraph(&content))
        .unwrap_or_else(|| format!("Imported skill package `{}`.", slug));
    let tags = frontmatter
        .as_deref()
        .and_then(|block| extract_frontmatter_tags(block))
        .unwrap_or_default();

    Ok(InferredPackageMetadata {
        name,
        description,
        tags,
    })
}

fn extract_frontmatter_tags(block: &str) -> Option<Vec<String>> {
    let raw = crate::utils::frontmatter::get_value(block, "tags")?;
    let tags = raw
        .trim_matches(['[', ']'])
        .split(',')
        .map(|item| item.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
}

fn extract_markdown_title(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let title = line.trim().strip_prefix("# ")?;
        Some(title.trim().to_string()).filter(|value| !value.is_empty())
    })
}

fn first_body_paragraph(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && *line != "---")
        .find(|line| !line.contains(':') || line.split_whitespace().count() > 3)
        .map(|line| line.chars().take(240).collect::<String>())
}

fn title_from_slug(slug: &str) -> String {
    crate::utils::ids::title_from_slug(slug)
}

pub fn shell_quote(value: &str) -> String {
    let mut quoted = String::new();
    for character in value.chars() {
        match character {
            '\\' | '"' | '$' | '`' => {
                quoted.push('\\');
                quoted.push(character);
            }
            _ => quoted.push(character),
        }
    }
    format!("\"{}\"", quoted)
}

pub fn sanitize_relative_path(relative_path: &str) -> Result<PathBuf, AppError> {
    let raw = relative_path.trim();
    if raw.is_empty() {
        return Err(AppError::Other("relative path cannot be empty".to_string()));
    }
    let candidate = Path::new(raw);
    if candidate.is_absolute() {
        return Err(AppError::InvalidPath(format!("absolute paths are not allowed: {}", raw)));
    }
    let mut cleaned = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => cleaned.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(AppError::InvalidPath(format!("parent traversal is not allowed: {}", raw)))
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::InvalidPath(format!("absolute paths are not allowed: {}", raw)))
            }
        }
    }
    Ok(cleaned)
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
    use super::{import_package_from_path, reference_package, update_package};
    use crate::domain::package::{PackageStatus, PackageUpdateRequest};
    use crate::storage::filesystem;
    use crate::test_helpers::{copy_example_project_root, tmp_project_root_path};

    use std::fs;

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

        assert!(error.to_string().contains("name cannot be empty"));
        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn imports_package_from_existing_directory_without_running_eval() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let source = root.join("incoming-skill");
        fs::create_dir_all(source.join("references")).expect("source dirs");
        fs::write(
            source.join("SKILL.md"),
            r#"---
name: Imported Notes
description: "Turns notes into reusable output."
tags: ["research", "notes"]
---

# Imported Notes

Use this skill when a pile of raw notes needs a compact reusable summary.
"#,
        )
        .expect("write skill");
        fs::write(source.join("references").join("sample.md"), "reference").expect("write ref");

        let imported = import_package_from_path(
            &root,
            source.to_string_lossy().as_ref(),
            Some("Imported Notes"),
            false,
        )
        .expect("import package");

        assert_eq!(imported.package_id, "pkg-imported-notes");
        assert_eq!(imported.slug, "imported-notes");
        assert!(imported.eval_report.is_none());
        assert!(imported
            .reference_command
            .contains("reference pkg-imported-notes"));

        let package_root = filesystem::canonical_skills_root(&root).join("imported-notes");
        let notebook = filesystem::load_package_notebook(&package_root).expect("notebook");
        assert_eq!(notebook.name, "Imported Notes");
        assert_eq!(notebook.description, "Turns notes into reusable output.");
        assert_eq!(notebook.tags, vec!["research", "notes"]);
        assert!(matches!(notebook.status, PackageStatus::Draft));
        assert!(package_root.join("references").join("sample.md").exists());

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn reference_package_returns_copyable_paths_and_commands() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);

        let reference = reference_package("pkg-interview", Some(root.to_string_lossy().as_ref()))
            .expect("reference package");
        let item_ids = reference
            .items
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(reference.package_id, "pkg-interview");
        assert!(reference.skill_md_path.ends_with("SKILL.md"));
        assert!(item_ids.contains(&"markdown-reference"));
        assert!(item_ids.contains(&"cli-reference"));
        assert!(item_ids.contains(&"global-claude-link"));

        std::fs::remove_dir_all(root).ok();
    }
}
