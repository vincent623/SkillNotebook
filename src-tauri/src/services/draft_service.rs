use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::domain::draft::{
    DraftDiscardRequest, DraftImportRequest, DraftImportResponse, DraftSourceKind,
    DraftStartRequest, DraftWorkspace,
};
use crate::domain::package::PackageImportResponse;
use crate::services::package_service;
use crate::storage::filesystem;
use crate::utils::errors::AppError;
use crate::utils::ids::slugify;
use crate::utils::time::now_iso;

const DEFAULT_AGENT_COMMAND: &str = "codex";

pub fn start_draft(
    payload: &DraftStartRequest,
    root_path: Option<&str>,
) -> Result<DraftWorkspace, AppError> {
    let project_root_path = filesystem::project_root_for_id(&payload.project_root_id, root_path)?;
    let prompt = payload
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let source_paths = payload
        .source_paths
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let source_url = payload
        .source_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let source_kind = if !source_paths.is_empty() {
        DraftSourceKind::Files
    } else if source_url.is_some() {
        DraftSourceKind::Url
    } else if prompt.is_some() {
        DraftSourceKind::Text
    } else {
        DraftSourceKind::Empty
    };
    let source_summary = build_source_summary(prompt, &source_paths, source_url);
    let slug_seed = prompt
        .or(source_url)
        .or_else(|| source_paths.first().map(String::as_str))
        .unwrap_or("new skill");
    let intended_slug = slugify(slug_seed)
        .split('-')
        .filter(|segment| !segment.is_empty())
        .take(5)
        .collect::<Vec<_>>()
        .join("-");
    let intended_slug = if intended_slug.is_empty() {
        "new-skill".to_string()
    } else {
        intended_slug
    };
    let created_at = now_iso();
    let draft_id = format!(
        "draft-{}-{}",
        intended_slug,
        created_at
            .replace([':', '.', 'T', 'Z'], "-")
            .trim_matches('-')
    );
    let draft_path = drafts_root(&project_root_path).join(&draft_id);
    let brief_path = draft_path.join("BRIEF.md");
    let agent_command = payload
        .preferred_agent_command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_AGENT_COMMAND);
    let suggested_command = format!(
        "cd {} && {}",
        package_service::shell_quote(draft_path.to_string_lossy().as_ref()),
        agent_command
    );
    let import_command = format!(
        "skill --project_root {} draft import {}",
        package_service::shell_quote(project_root_path.to_string_lossy().as_ref()),
        draft_id
    );

    filesystem::ensure_directory(&draft_path)?;
    for dir in [
        "prompts",
        "examples",
        "references",
        "scripts",
        "tests",
        "evals",
    ] {
        filesystem::ensure_directory(&draft_path.join(dir))?;
    }
    filesystem::write_text_file(
        &brief_path,
        &build_brief(
            prompt,
            &source_paths,
            source_url,
            &suggested_command,
            &import_command,
        ),
    )?;
    filesystem::write_text_file(
        &draft_path.join("SKILL.md"),
        &build_skill_scaffold(&intended_slug),
    )?;

    let workspace = DraftWorkspace {
        draft_id,
        project_root_id: payload.project_root_id.clone(),
        draft_path: draft_path.to_string_lossy().to_string(),
        brief_path: brief_path.to_string_lossy().to_string(),
        intended_slug,
        source_kind,
        source_summary,
        suggested_command,
        import_command,
        created_at,
    };
    filesystem::write_json_file(&draft_path.join("draft.json"), &workspace)?;

    Ok(workspace)
}

pub fn list_drafts(root_path: Option<&str>) -> Result<Vec<DraftWorkspace>, AppError> {
    let scanned = filesystem::scan_project_root(root_path)?;
    let root = drafts_root(Path::new(&scanned.project_root.root_path));
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut drafts = Vec::new();
    for entry in fs::read_dir(&root)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("draft.json");
        if !manifest.exists() {
            continue;
        }
        let workspace = fs::read_to_string(&manifest)
            .ok()
            .and_then(|content| serde_json::from_str::<DraftWorkspace>(&content).ok());
        if let Some(workspace) = workspace {
            drafts.push(workspace);
        }
    }
    drafts.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(drafts)
}

pub fn discard_draft(
    payload: &DraftDiscardRequest,
    root_path: Option<&str>,
) -> Result<bool, AppError> {
    let project_root_path = filesystem::project_root_for_id(&payload.project_root_id, root_path)?;
    let draft_path = draft_path_for_id(&project_root_path, &payload.draft_id)?;
    if !draft_path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(&draft_path)?;
    Ok(true)
}

pub fn import_draft(
    payload: &DraftImportRequest,
    root_path: Option<&str>,
) -> Result<DraftImportResponse, AppError> {
    let project_root_path = filesystem::project_root_for_id(&payload.project_root_id, root_path)?;
    let draft_path = draft_path_for_id(&project_root_path, &payload.draft_id)?;
    if !draft_path.exists() {
        return Err(AppError::Other(format!("draft not found: {}", payload.draft_id)));
    }
    let manifest = read_draft_manifest(&draft_path)?;
    let PackageImportResponse {
        package_id,
        slug,
        package_path,
        eval_report,
        eval_command,
        version_command,
        reference_command,
        imported_at,
    } = package_service::import_package_from_path(
        &project_root_path,
        draft_path.to_string_lossy().as_ref(),
        Some(&manifest.intended_slug),
        payload.run_eval.unwrap_or(true),
    )?;

    Ok(DraftImportResponse {
        draft_id: manifest.draft_id,
        package_id,
        slug,
        package_path,
        eval_report,
        eval_command,
        version_command,
        reference_command,
        imported_at,
    })
}

fn drafts_root(project_root_path: &Path) -> PathBuf {
    project_root_path.join(".skill-notebook").join("drafts")
}

fn draft_path_for_id(project_root_path: &Path, draft_id: &str) -> Result<PathBuf, AppError> {
    let raw = draft_id.trim();
    if raw.is_empty() {
        return Err(AppError::Other("draft id cannot be empty".to_string()));
    }

    let raw_path = Path::new(raw);
    if !raw_path.is_absolute() && raw_path.components().count() == 1 {
        let relative = package_service::sanitize_relative_path(raw)?;
        return Ok(drafts_root(project_root_path).join(relative));
    }

    let candidate = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        project_root_path.join(raw_path)
    };
    let candidate = normalize_existing_or_lexical_path(&candidate)?;
    let root = normalize_existing_or_lexical_path(&drafts_root(project_root_path))?;
    if !candidate.starts_with(&root) {
        return Err(AppError::InvalidPath(format!(
            "draft path must be under {}: {}",
            root.display(),
            raw
        )));
    }

    Ok(candidate)
}

fn normalize_existing_or_lexical_path(path: &Path) -> Result<PathBuf, AppError> {
    let path = normalize_lexical_path(path)?;
    Ok(fs::canonicalize(&path).unwrap_or(path))
}

fn normalize_lexical_path(path: &Path) -> Result<PathBuf, AppError> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                return Err(AppError::InvalidPath(format!(
                    "parent traversal is not allowed: {}",
                    path.display()
                )))
            }
        }
    }
    Ok(normalized)
}

fn read_draft_manifest(draft_path: &Path) -> Result<DraftWorkspace, AppError> {
    let manifest = draft_path.join("draft.json");
    let content = fs::read_to_string(&manifest)?;
    Ok(serde_json::from_str(&content)?)
}

fn build_source_summary(
    prompt: Option<&str>,
    source_paths: &[String],
    source_url: Option<&str>,
) -> String {
    if !source_paths.is_empty() {
        return format!("{} local source path(s)", source_paths.len());
    }
    if let Some(url) = source_url {
        return format!("URL source: {}", url);
    }
    prompt
        .map(|value| {
            if value.chars().count() > 120 {
                format!("{}...", value.chars().take(120).collect::<String>())
            } else {
                value.to_string()
            }
        })
        .unwrap_or_else(|| "Empty draft workspace".to_string())
}

fn build_brief(
    prompt: Option<&str>,
    source_paths: &[String],
    source_url: Option<&str>,
    suggested_command: &str,
    import_command: &str,
) -> String {
    let mut lines = vec![
        "# Skill Draft Brief".to_string(),
        String::new(),
        "Use this workspace to create or refine a reusable Skill package.".to_string(),
        String::new(),
        "## Goal".to_string(),
        String::new(),
        prompt.unwrap_or("Define the skill goal here.").to_string(),
        String::new(),
        "## Source Material".to_string(),
        String::new(),
    ];
    if source_paths.is_empty() && source_url.is_none() {
        lines.push("- No source material attached yet.".to_string());
    }
    for path in source_paths {
        lines.push(format!("- Local path: `{}`", path));
    }
    if let Some(url) = source_url {
        lines.push(format!("- URL: {}", url));
    }
    lines.extend([
        String::new(),
        "## Package Requirements".to_string(),
        String::new(),
        "- Keep `SKILL.md` as the entry point.".to_string(),
        "- Use `references/` for longer background material.".to_string(),
        "- Use `examples/` for at least one realistic input/output example.".to_string(),
        "- Use `tests/` or `evals/` for repeatable expectations when possible.".to_string(),
        "- Be explicit about when to use and when not to use the skill.".to_string(),
        String::new(),
        "## Handoff Commands".to_string(),
        String::new(),
        "```bash".to_string(),
        suggested_command.to_string(),
        "```".to_string(),
        String::new(),
        "When the draft is ready, import it:".to_string(),
        String::new(),
        "```bash".to_string(),
        import_command.to_string(),
        "```".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

fn build_skill_scaffold(slug: &str) -> String {
    format!(
        "---
name: {slug}
description: \"Draft skill package. Use when the workflow has been completed and evaluated.\"
---

# {}

## When To Use

- Define the trigger condition.

## When Not To Use

- Define boundaries here.

## Inputs

- Define expected input material.

## Outputs

- Define the expected output contract.

## Workflow

1. Inspect the input.
2. Apply the reusable workflow.
3. Return the agreed output shape.
",
        crate::utils::ids::title_from_slug(slug)
    )
}

#[cfg(test)]
mod tests {
    use super::{discard_draft, import_draft, list_drafts, start_draft};
    use crate::domain::draft::{DraftDiscardRequest, DraftImportRequest, DraftStartRequest};
    use crate::test_helpers::{copy_example_project_root, tmp_project_root_path};

    use std::path::Path;

    #[test]
    fn start_draft_creates_handoff_workspace() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let req = DraftStartRequest {
            project_root_id: "project-root-main".to_string(),
            prompt: Some("Create a research synthesis skill".to_string()),
            source_paths: None,
            source_url: None,
            preferred_agent_command: Some("codex --model gpt-5.4".to_string()),
        };

        let draft = start_draft(&req, Some(root.to_string_lossy().as_ref())).expect("start draft");

        assert!(draft.draft_id.starts_with("draft-create-a-research"));
        assert!(Path::new(&draft.brief_path).exists());
        assert!(Path::new(&draft.draft_path).join("SKILL.md").exists());
        assert!(draft.suggested_command.contains("codex --model gpt-5.4"));
        assert!(draft.import_command.contains("draft import"));

        let drafts = list_drafts(Some(root.to_string_lossy().as_ref())).expect("list drafts");
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].draft_id, draft.draft_id);

        let discarded = discard_draft(
            &DraftDiscardRequest {
                project_root_id: "project-root-main".to_string(),
                draft_id: draft.draft_id,
            },
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("discard draft");
        assert!(discarded);

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn import_draft_promotes_workspace_to_package_without_eval() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let draft = start_draft(
            &DraftStartRequest {
                project_root_id: "project-root-main".to_string(),
                prompt: Some("Summarize customer calls".to_string()),
                source_paths: Some(vec!["notes/call.md".to_string()]),
                source_url: None,
                preferred_agent_command: None,
            },
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("start draft");

        let imported = import_draft(
            &DraftImportRequest {
                project_root_id: "project-root-main".to_string(),
                draft_id: draft.draft_id.clone(),
                run_eval: Some(false),
            },
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("import draft");

        assert_eq!(imported.draft_id, draft.draft_id);
        assert!(imported.package_id.starts_with("pkg-"));
        assert!(imported.eval_report.is_none());
        assert!(imported.reference_command.contains("reference"));
        assert!(Path::new(&imported.package_path).join("SKILL.md").exists());

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn import_draft_accepts_workspace_path() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let draft = start_draft(
            &DraftStartRequest {
                project_root_id: "project-root-main".to_string(),
                prompt: Some("Draft from a workspace path".to_string()),
                source_paths: None,
                source_url: None,
                preferred_agent_command: None,
            },
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("start draft");
        let draft_relative_path = format!(".skill-notebook/drafts/{}", draft.draft_id);

        let imported = import_draft(
            &DraftImportRequest {
                project_root_id: "project-root-main".to_string(),
                draft_id: draft_relative_path,
                run_eval: Some(false),
            },
            Some(root.to_string_lossy().as_ref()),
        )
        .expect("import draft from workspace path");

        assert_eq!(imported.draft_id, draft.draft_id);
        assert!(Path::new(&imported.package_path).join("SKILL.md").exists());

        std::fs::remove_dir_all(root).ok();
    }
}
