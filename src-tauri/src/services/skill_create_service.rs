use std::env;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{Duration as TimeDuration, OffsetDateTime};

use crate::domain::package::{
    CommitPackagePreviewRequest, CreatePackageFromNlRequest, CreatePackageFromNlResponse,
    CreatePackageFromSourcesRequest, CreatePackageFromUrlRequest, CreatePackagePreviewResponse,
    DiscardPackagePreviewRequest, PackageNotebookDocument, PackagePreviewFile, PackageStatus,
};
use crate::services::eval_service;
use crate::storage::filesystem;
use crate::utils::ids::slugify;
use crate::utils::time::{now_iso, parse_iso, today_slug};

const DEFAULT_CLAUDE_BINARY: &str = "claude";
const DEFAULT_CLAUDE_TIMEOUT_SECS: u64 = 300;
const DEFAULT_SKILL_CREATE_BINARY: &str = "skill-create";
const DEFAULT_SKILL_CREATE_TIMEOUT_SECS: u64 = 60;
const CLAUDE_POLL_INTERVAL_MS: u64 = 100;
const CREATOR_FALLBACK: &str = "template_fallback";
const CREATOR_SKILL_CREATE: &str = "skill_create_cli";
const CREATOR_CLAUDE: &str = "claude_cli";
const CREATE_PREVIEW_TTL_HOURS: i64 = 24;
const SOURCE_FILE_LIMIT: usize = 40;
const SOURCE_EXCERPT_LIMIT_CHARS: usize = 1800;
const SOURCE_CONTEXT_LIMIT_CHARS: usize = 18_000;
const URL_SOURCE_LIMIT_BYTES: usize = 1_048_576;
const URL_CONTEXT_LIMIT_CHARS: usize = 16_000;

#[derive(Debug, Clone)]
struct DraftPackage {
    name: String,
    slug: String,
    description: String,
    tags: Vec<String>,
    skill_md: String,
    system_prompt: String,
    task_prompt: String,
    example_markdown: String,
    smoke_prompt: String,
    expected_output: String,
    expectations: Vec<String>,
}

#[derive(Debug, Clone)]
struct DraftResult {
    draft: DraftPackage,
    generator_used: String,
    generation_summary: String,
    prompt_log: Option<String>,
    response_log: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreatorMode {
    Auto,
    Template,
    SkillCreate,
    ClaudeCli,
}

#[derive(Debug, Clone)]
struct CreatorBridgeOptions {
    mode: CreatorMode,
    skill_create_bin: String,
    skill_create_timeout_secs: u64,
    claude_bin: String,
    claude_model: Option<String>,
    claude_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct GeneratorDraftPayload {
    name: String,
    slug: String,
    description: String,
    skill_md: String,
    system_prompt: String,
    task_prompt: String,
    example_markdown: String,
    smoke_prompt: String,
    expected_output: String,
    expectations: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePreviewManifest {
    preview_id: String,
    project_root_id: String,
    name: String,
    slug: String,
    description: String,
    tags: Vec<String>,
    generator_used: String,
    generation_summary: String,
    prompt_log: Option<String>,
    response_log: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone)]
struct SourceContext {
    requested_paths: Vec<String>,
    inventory_markdown: String,
    generation_context: String,
}

#[derive(Debug, Clone)]
struct SourceFileSummary {
    path: String,
    size_bytes: u64,
    kind: String,
    excerpt: Option<String>,
}

#[derive(Debug, Clone)]
struct UrlSourceContext {
    url: String,
    inventory_markdown: String,
    generation_context: String,
}

pub fn create_package_from_nl(
    req: &CreatePackageFromNlRequest,
    root_path: Option<&str>,
) -> Result<CreatePackageFromNlResponse, String> {
    let project_root_path = filesystem::project_root_for_id(&req.project_root_id, root_path)?;
    let options = resolve_creator_bridge_options();
    create_package_in_workspace_with_options(
        &project_root_path,
        &req.project_root_id,
        req,
        &options,
    )
}

pub fn generate_package_preview_from_nl(
    req: &CreatePackageFromNlRequest,
    root_path: Option<&str>,
) -> Result<CreatePackagePreviewResponse, String> {
    let project_root_path = filesystem::project_root_for_id(&req.project_root_id, root_path)?;
    let options = resolve_creator_bridge_options();
    generate_package_preview_in_workspace_with_options(
        &project_root_path,
        &req.project_root_id,
        req,
        &options,
    )
}

pub fn generate_package_preview_from_sources(
    req: &CreatePackageFromSourcesRequest,
    root_path: Option<&str>,
) -> Result<CreatePackagePreviewResponse, String> {
    let project_root_path = filesystem::project_root_for_id(&req.project_root_id, root_path)?;
    let options = resolve_creator_bridge_options();
    generate_package_preview_from_sources_in_workspace_with_options(
        &project_root_path,
        &req.project_root_id,
        req,
        &options,
    )
}

pub fn generate_package_preview_from_url(
    req: &CreatePackageFromUrlRequest,
    root_path: Option<&str>,
) -> Result<CreatePackagePreviewResponse, String> {
    let project_root_path = filesystem::project_root_for_id(&req.project_root_id, root_path)?;
    let options = resolve_creator_bridge_options();
    generate_package_preview_from_url_in_workspace_with_options(
        &project_root_path,
        &req.project_root_id,
        req,
        &options,
    )
}

pub fn commit_package_preview(
    req: &CommitPackagePreviewRequest,
    root_path: Option<&str>,
) -> Result<CreatePackageFromNlResponse, String> {
    let project_root_path = filesystem::project_root_for_id(&req.project_root_id, root_path)?;
    commit_package_preview_in_workspace(&project_root_path, req)
}

pub fn discard_package_preview(
    req: &DiscardPackagePreviewRequest,
    root_path: Option<&str>,
) -> Result<bool, String> {
    let project_root_path = filesystem::project_root_for_id(&req.project_root_id, root_path)?;
    discard_package_preview_in_workspace(&project_root_path, req)
}

pub fn cleanup_stale_package_previews(root_path: &str) -> Result<usize, String> {
    let project_root_path = PathBuf::from(root_path);
    cleanup_stale_package_previews_in_workspace_with_now(
        &project_root_path,
        TimeDuration::hours(CREATE_PREVIEW_TTL_HOURS),
        OffsetDateTime::now_utc(),
    )
}

pub fn creator_bridge_status() -> serde_json::Value {
    let options = resolve_creator_bridge_options();
    let claude_path = resolve_command_path(&options.claude_bin);
    let skill_create_path = resolve_command_path(&options.skill_create_bin);
    let claude_available = claude_path.is_some();
    let skill_create_available = skill_create_path.is_some();
    let preferred_generator = match options.mode {
        CreatorMode::Template => CREATOR_FALLBACK,
        CreatorMode::SkillCreate => CREATOR_SKILL_CREATE,
        CreatorMode::ClaudeCli => CREATOR_CLAUDE,
        CreatorMode::Auto => {
            if skill_create_available {
                CREATOR_SKILL_CREATE
            } else if claude_available {
                CREATOR_CLAUDE
            } else {
                CREATOR_FALLBACK
            }
        }
    };

    json!({
        "mode": options.mode.as_str(),
        "preferredGenerator": preferred_generator,
        "claudeCliAvailable": claude_available,
        "skillCreateCommandAvailable": skill_create_available,
        "claudeBinary": options.claude_bin,
        "claudeResolvedPath": claude_path.map(|path| path.to_string_lossy().to_string()),
        "skillCreateResolvedPath": skill_create_path.map(|path| path.to_string_lossy().to_string()),
        "claudeModel": options.claude_model,
        "claudeTimeoutSecs": options.claude_timeout_secs,
        "fallbackGenerator": CREATOR_FALLBACK,
    })
}

fn resolve_creator_bridge_options() -> CreatorBridgeOptions {
    let mode = match env::var("SKILL_NOTEBOOK_CREATOR_MODE")
        .unwrap_or_else(|_| "auto".to_string())
        .to_lowercase()
        .as_str()
    {
        "template" | "fallback" => CreatorMode::Template,
        "skill_create" | "skill-create" => CreatorMode::SkillCreate,
        "claude" | "claude_cli" => CreatorMode::ClaudeCli,
        _ => CreatorMode::Auto,
    };

    CreatorBridgeOptions {
        mode,
        skill_create_bin: env::var("SKILL_NOTEBOOK_SKILL_CREATE_BIN")
            .unwrap_or_else(|_| DEFAULT_SKILL_CREATE_BINARY.to_string()),
        skill_create_timeout_secs: env::var("SKILL_NOTEBOOK_SKILL_CREATE_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_SKILL_CREATE_TIMEOUT_SECS),
        claude_bin: env::var("SKILL_NOTEBOOK_CLAUDE_BIN")
            .unwrap_or_else(|_| DEFAULT_CLAUDE_BINARY.to_string()),
        claude_model: env::var("SKILL_NOTEBOOK_CLAUDE_MODEL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        claude_timeout_secs: env::var("SKILL_NOTEBOOK_CLAUDE_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_CLAUDE_TIMEOUT_SECS),
    }
}

fn create_package_in_workspace_with_options(
    project_root_path: &Path,
    _workspace_id: &str,
    req: &CreatePackageFromNlRequest,
    options: &CreatorBridgeOptions,
) -> Result<CreatePackageFromNlResponse, String> {
    let packages_root = filesystem::canonical_skills_root(project_root_path);
    filesystem::ensure_directory(&packages_root)?;

    let draft_result = prepare_draft_result(project_root_path, req, options)?;
    let DraftResult {
        draft,
        generator_used,
        generation_summary,
        prompt_log,
        response_log,
    } = draft_result;
    let slug = draft.slug.clone();
    let package_root = packages_root.join(&slug);
    let package_id = format!("pkg-{}", slug);
    let created_at = now_iso();

    write_draft_files(&package_root, &draft, &slug)?;
    write_generator_log(
        project_root_path,
        &slug,
        &generator_used,
        &generation_summary,
        prompt_log.as_deref(),
        response_log.as_deref(),
    )?;

    let mut notebook = PackageNotebookDocument {
        id: package_id.clone(),
        name: draft.name.clone(),
        description: draft.description.clone(),
        tags: draft.tags.clone(),
        status: PackageStatus::Draft,
        current_version: 0,
        last_eval_status: None,
        related_skills: Vec::new(),
        bundle_candidates: Vec::new(),
        created_at: created_at.clone(),
        updated_at: created_at.clone(),
        versions: Vec::new(),
        eval_reports: Vec::new(),
    };

    filesystem::save_package_notebook(&package_root, &notebook)?;

    let evaluation = eval_service::evaluate_package(
        project_root_path,
        &package_root,
        &package_id,
        &slug,
        &draft.name,
        &draft.description,
        1,
    )?;

    notebook.last_eval_status = Some(evaluation.report.overall_status.clone());
    notebook.status = match evaluation.suggested_status {
        PackageStatus::Validated => PackageStatus::NeedsEval,
        other => other,
    };
    notebook.updated_at = now_iso();
    notebook.eval_reports.push(evaluation.report.clone());

    filesystem::save_package_notebook(&package_root, &notebook)?;

    Ok(CreatePackageFromNlResponse {
        package_id,
        name: notebook.name,
        slug: slug.clone(),
        root_path: package_root.to_string_lossy().to_string(),
        eval_workspace_path: project_root_path
            .join(".42eval")
            .join(&slug)
            .to_string_lossy()
            .to_string(),
        draft_created: true,
        auto_eval_started: true,
        validation_summary: evaluation.validation_summary,
        generator_used,
        generation_summary,
    })
}

fn generate_package_preview_in_workspace_with_options(
    project_root_path: &Path,
    project_root_id: &str,
    req: &CreatePackageFromNlRequest,
    options: &CreatorBridgeOptions,
) -> Result<CreatePackagePreviewResponse, String> {
    let _ = cleanup_stale_package_previews_in_workspace_with_now(
        project_root_path,
        TimeDuration::hours(CREATE_PREVIEW_TTL_HOURS),
        OffsetDateTime::now_utc(),
    );

    let draft_result = prepare_draft_result(project_root_path, req, options)?;
    let DraftResult {
        draft,
        generator_used,
        generation_summary,
        prompt_log,
        response_log,
    } = draft_result;
    let slug = draft.slug.clone();
    let created_at = now_iso();
    let preview_id = build_preview_id(&slug);
    let preview_dir = create_preview_dir(project_root_path, &preview_id)?;
    let preview_package_root = preview_dir.join("package");
    let files = draft_preview_files(&draft, &slug)?;

    write_preview_files(&preview_package_root, &files)?;
    filesystem::write_json_file(
        &preview_dir.join("preview.json"),
        &CreatePreviewManifest {
            preview_id: preview_id.clone(),
            project_root_id: project_root_id.to_string(),
            name: draft.name.clone(),
            slug: slug.clone(),
            description: draft.description.clone(),
            tags: draft.tags.clone(),
            generator_used: generator_used.clone(),
            generation_summary: generation_summary.clone(),
            prompt_log,
            response_log,
            created_at: created_at.clone(),
        },
    )?;

    let file_tree = filesystem::list_package_file_tree(&preview_package_root)?;

    Ok(CreatePackagePreviewResponse {
        preview_id,
        project_root_id: project_root_id.to_string(),
        name: draft.name,
        slug,
        description: draft.description,
        tags: draft.tags,
        files,
        file_tree,
        generator_used,
        generation_summary,
        created_at,
    })
}

fn generate_package_preview_from_sources_in_workspace_with_options(
    project_root_path: &Path,
    project_root_id: &str,
    req: &CreatePackageFromSourcesRequest,
    options: &CreatorBridgeOptions,
) -> Result<CreatePackagePreviewResponse, String> {
    let source_context = build_source_context(project_root_path, &req.source_paths)?;
    let source_goal = req
        .prompt
        .as_deref()
        .map(normalize_text)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "Create a reusable skill from these local source materials: {}.",
                summarize_source_names(&source_context.requested_paths)
            )
        });
    let mut context_parts = Vec::new();
    if let Some(context) = req
        .context
        .as_deref()
        .map(normalize_text)
        .filter(|value| !value.trim().is_empty())
    {
        context_parts.push(format!("User notes:\n{}", context));
    }
    context_parts.push(source_context.generation_context.clone());

    let synthetic_req = CreatePackageFromNlRequest {
        project_root_id: project_root_id.to_string(),
        prompt: source_goal,
        context: Some(context_parts.join("\n\n")),
    };
    let mut response = generate_package_preview_in_workspace_with_options(
        project_root_path,
        project_root_id,
        &synthetic_req,
        options,
    )?;
    let inventory_file = PackagePreviewFile {
        path: "references/source-inventory.md".to_string(),
        content: source_context.inventory_markdown,
        encoding: "utf-8".to_string(),
    };
    let preview_root = preview_dir(project_root_path, &safe_preview_id(&response.preview_id)?);
    let preview_package_root = preview_root.join("package");
    filesystem::write_text_file(
        &preview_package_root.join(&inventory_file.path),
        &inventory_file.content,
    )?;
    response.files.push(inventory_file);
    response.file_tree = filesystem::list_package_file_tree(&preview_package_root)?;
    response.generation_summary = format!(
        "{} Source inventory attached from {} local path(s).",
        response.generation_summary,
        source_context.requested_paths.len()
    );
    let manifest_path = preview_root.join("preview.json");
    let mut manifest = read_preview_manifest(&manifest_path)?;
    manifest.generation_summary = response.generation_summary.clone();
    filesystem::write_json_file(&manifest_path, &manifest)?;

    Ok(response)
}

fn generate_package_preview_from_url_in_workspace_with_options(
    project_root_path: &Path,
    project_root_id: &str,
    req: &CreatePackageFromUrlRequest,
    options: &CreatorBridgeOptions,
) -> Result<CreatePackagePreviewResponse, String> {
    let url_context = build_url_source_context(&req.url)?;
    let source_goal = req
        .prompt
        .as_deref()
        .map(normalize_text)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("Create a reusable skill from {}.", url_context.url));
    let mut context_parts = Vec::new();
    if let Some(context) = req
        .context
        .as_deref()
        .map(normalize_text)
        .filter(|value| !value.trim().is_empty())
    {
        context_parts.push(format!("User notes:\n{}", context));
    }
    context_parts.push(url_context.generation_context.clone());

    let synthetic_req = CreatePackageFromNlRequest {
        project_root_id: project_root_id.to_string(),
        prompt: source_goal,
        context: Some(context_parts.join("\n\n")),
    };
    let mut response = generate_package_preview_in_workspace_with_options(
        project_root_path,
        project_root_id,
        &synthetic_req,
        options,
    )?;
    let source_file = PackagePreviewFile {
        path: "references/url-source.md".to_string(),
        content: url_context.inventory_markdown,
        encoding: "utf-8".to_string(),
    };
    let preview_root = preview_dir(project_root_path, &safe_preview_id(&response.preview_id)?);
    let preview_package_root = preview_root.join("package");
    filesystem::write_text_file(
        &preview_package_root.join(&source_file.path),
        &source_file.content,
    )?;
    response.files.push(source_file);
    response.file_tree = filesystem::list_package_file_tree(&preview_package_root)?;
    response.generation_summary = format!(
        "{} URL source attached from {}.",
        response.generation_summary, url_context.url
    );
    let manifest_path = preview_root.join("preview.json");
    let mut manifest = read_preview_manifest(&manifest_path)?;
    manifest.generation_summary = response.generation_summary.clone();
    filesystem::write_json_file(&manifest_path, &manifest)?;

    Ok(response)
}

fn commit_package_preview_in_workspace(
    project_root_path: &Path,
    req: &CommitPackagePreviewRequest,
) -> Result<CreatePackageFromNlResponse, String> {
    let preview_id = safe_preview_id(&req.preview_id)?;
    let preview_root = preview_dir(project_root_path, &preview_id);
    let preview_package_root = preview_root.join("package");
    let manifest_path = preview_root.join("preview.json");
    let manifest = read_preview_manifest(&manifest_path)?;

    if manifest.project_root_id != req.project_root_id {
        return Err("preview belongs to a different project root".to_string());
    }

    if !preview_package_root.is_dir() {
        return Err(format!(
            "preview package directory is missing: {}",
            preview_package_root.display()
        ));
    }

    let packages_root = filesystem::canonical_skills_root(project_root_path);
    filesystem::ensure_directory(&packages_root)?;
    let package_root = packages_root.join(&manifest.slug);

    if package_root.exists() {
        return Err(format!(
            "package `{}` already exists. Regenerate the preview to allocate a new slug.",
            manifest.slug
        ));
    }

    if let Err(error) = filesystem::copy_directory_recursive(&preview_package_root, &package_root) {
        std::fs::remove_dir_all(&package_root).ok();
        return Err(error);
    }

    let eval_workspace_path = project_root_path.join(".42eval").join(&manifest.slug);
    let generator_log_dir = project_root_path
        .join(".skill-notebook")
        .join("generator-runs")
        .join(&manifest.slug);
    let commit_result = (|| {
        write_generator_log(
            project_root_path,
            &manifest.slug,
            &manifest.generator_used,
            &manifest.generation_summary,
            manifest.prompt_log.as_deref(),
            manifest.response_log.as_deref(),
        )?;

        let package_id = format!("pkg-{}", manifest.slug);
        let created_at = now_iso();
        let mut notebook = PackageNotebookDocument {
            id: package_id.clone(),
            name: manifest.name.clone(),
            description: manifest.description.clone(),
            tags: manifest.tags.clone(),
            status: PackageStatus::Draft,
            current_version: 0,
            last_eval_status: None,
            related_skills: Vec::new(),
            bundle_candidates: Vec::new(),
            created_at: created_at.clone(),
            updated_at: created_at.clone(),
            versions: Vec::new(),
            eval_reports: Vec::new(),
        };

        filesystem::save_package_notebook(&package_root, &notebook)?;

        let evaluation = eval_service::evaluate_package(
            project_root_path,
            &package_root,
            &package_id,
            &manifest.slug,
            &manifest.name,
            &manifest.description,
            1,
        )?;

        notebook.last_eval_status = Some(evaluation.report.overall_status.clone());
        notebook.status = match evaluation.suggested_status {
            PackageStatus::Validated => PackageStatus::NeedsEval,
            other => other,
        };
        notebook.updated_at = now_iso();
        notebook.eval_reports.push(evaluation.report.clone());

        filesystem::save_package_notebook(&package_root, &notebook)?;

        Ok(CreatePackageFromNlResponse {
            package_id,
            name: notebook.name,
            slug: manifest.slug.clone(),
            root_path: package_root.to_string_lossy().to_string(),
            eval_workspace_path: eval_workspace_path.to_string_lossy().to_string(),
            draft_created: true,
            auto_eval_started: true,
            validation_summary: evaluation.validation_summary,
            generator_used: manifest.generator_used.clone(),
            generation_summary: manifest.generation_summary.clone(),
        })
    })();

    match commit_result {
        Ok(response) => {
            std::fs::remove_dir_all(&preview_root).ok();
            Ok(response)
        }
        Err(error) => {
            std::fs::remove_dir_all(&package_root).ok();
            std::fs::remove_dir_all(&eval_workspace_path).ok();
            std::fs::remove_dir_all(&generator_log_dir).ok();
            Err(error)
        }
    }
}

fn discard_package_preview_in_workspace(
    project_root_path: &Path,
    req: &DiscardPackagePreviewRequest,
) -> Result<bool, String> {
    let preview_id = safe_preview_id(&req.preview_id)?;
    let preview_root = preview_dir(project_root_path, &preview_id);

    if !preview_root.exists() {
        return Ok(false);
    }

    let manifest_path = preview_root.join("preview.json");
    if manifest_path.exists() {
        let manifest = read_preview_manifest(&manifest_path)?;
        if manifest.project_root_id != req.project_root_id {
            return Err("preview belongs to a different project root".to_string());
        }
    }

    std::fs::remove_dir_all(&preview_root).map_err(|error| {
        format!(
            "failed to discard preview {}: {}",
            preview_root.display(),
            error
        )
    })?;
    Ok(true)
}

fn cleanup_stale_package_previews_in_workspace_with_now(
    project_root_path: &Path,
    ttl: TimeDuration,
    now: OffsetDateTime,
) -> Result<usize, String> {
    let previews_root = project_root_path
        .join(".skill-notebook")
        .join("create-previews");
    if !previews_root.exists() {
        return Ok(0);
    }

    let mut removed = 0;
    for entry in std::fs::read_dir(&previews_root)
        .map_err(|error| format!("failed to read {}: {}", previews_root.display(), error))?
    {
        let path = entry
            .map_err(|error| format!("failed to inspect preview entry: {}", error))?
            .path();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {}", path.display(), error))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }

        if preview_workspace_is_stale(&path, ttl, now, &metadata) {
            std::fs::remove_dir_all(&path).map_err(|error| {
                format!(
                    "failed to remove stale preview {}: {}",
                    path.display(),
                    error
                )
            })?;
            removed += 1;
        }
    }

    Ok(removed)
}

fn preview_workspace_is_stale(
    preview_root: &Path,
    ttl: TimeDuration,
    now: OffsetDateTime,
    metadata: &std::fs::Metadata,
) -> bool {
    let manifest_path = preview_root.join("preview.json");
    if manifest_path.exists() {
        if let Ok(manifest) = read_preview_manifest(&manifest_path) {
            if let Ok(created_at) = parse_iso(&manifest.created_at) {
                return now - created_at > ttl;
            }
        }
    }

    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .map(|age| age > ttl.unsigned_abs())
        .unwrap_or(false)
}

fn build_source_context(
    project_root_path: &Path,
    source_paths: &[String],
) -> Result<SourceContext, String> {
    let requested = source_paths
        .iter()
        .map(|value| normalize_source_path_input(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if requested.is_empty() {
        return Err("at least one source file or directory path is required".to_string());
    }

    let mut requested_paths = Vec::new();
    let mut files = Vec::new();
    for raw_path in requested {
        let source_path = resolve_source_path(project_root_path, &raw_path)?;
        requested_paths.push(display_source_path(project_root_path, &source_path));
        collect_source_files(project_root_path, &source_path, &mut files)?;
        if files.len() >= SOURCE_FILE_LIMIT {
            break;
        }
    }

    if files.is_empty() {
        return Err("no source files were found in the provided paths".to_string());
    }

    let included_count = files.len();
    let excerpt_count = files.iter().filter(|file| file.excerpt.is_some()).count();
    let mut inventory = String::new();
    inventory.push_str("# Source Inventory\n\n");
    inventory.push_str("This preview was generated from local source paths.\n\n");
    inventory.push_str("## Requested Paths\n\n");
    for path in &requested_paths {
        inventory.push_str(&format!("- `{}`\n", path));
    }
    inventory.push_str("\n## Included Files\n\n");
    for file in &files {
        inventory.push_str(&format!(
            "- `{}` ({}, {})\n",
            file.path,
            format_size(file.size_bytes),
            file.kind
        ));
    }
    inventory.push_str("\n## Text Excerpts\n\n");
    for file in files.iter().filter(|file| file.excerpt.is_some()) {
        inventory.push_str(&format!("### `{}`\n\n", file.path));
        inventory.push_str("```text\n");
        inventory.push_str(file.excerpt.as_deref().unwrap_or_default());
        inventory.push_str("\n```\n\n");
    }
    if excerpt_count == 0 {
        inventory.push_str(
            "No UTF-8 text excerpts were available; generation used file names and metadata.\n",
        );
    }

    let generation_context = truncate_chars(
        &format!(
            "Local source inventory:\n\n{}\n\nUse these files as source material. Build a reusable skill that helps repeat the workflow implied by the paths, filenames, and excerpts. Preserve uncertainty when source content is unavailable.",
            inventory
        ),
        SOURCE_CONTEXT_LIMIT_CHARS,
    );
    let inventory_markdown = format!(
        "{}\n---\n\nIncluded {} file(s); {} file(s) contributed text excerpts.\n",
        inventory, included_count, excerpt_count
    );

    Ok(SourceContext {
        requested_paths,
        inventory_markdown,
        generation_context,
    })
}

fn build_url_source_context(raw_url: &str) -> Result<UrlSourceContext, String> {
    let url = raw_url.trim();
    if !is_http_url(url) {
        return Err("URL source must start with http:// or https://".to_string());
    }
    if url.chars().any(char::is_whitespace) {
        return Err("URL source cannot contain whitespace".to_string());
    }

    let fetched = fetch_url_text(url)?;
    let excerpt = truncate_chars(&fetched, URL_CONTEXT_LIMIT_CHARS);
    let inventory_markdown = format!(
        "# URL Source\n\n- URL: {}\n- Fetched bytes: {}\n\n## Text Excerpt\n\n```text\n{}\n```\n",
        url,
        fetched.len(),
        excerpt
    );
    let generation_context = truncate_chars(
        &format!(
            "Remote URL source:\n{}\n\nFetched text excerpt:\n{}",
            url, excerpt
        ),
        URL_CONTEXT_LIMIT_CHARS,
    );

    Ok(UrlSourceContext {
        url: url.to_string(),
        inventory_markdown,
        generation_context,
    })
}

fn is_http_url(value: &str) -> bool {
    let lowered = value.to_lowercase();
    lowered.starts_with("http://") || lowered.starts_with("https://")
}

fn fetch_url_text(url: &str) -> Result<String, String> {
    let curl_path = resolve_command_path("curl")
        .ok_or_else(|| "URL generation requires curl on PATH".to_string())?;

    let output = Command::new(curl_path)
        .arg("--location")
        .arg("--max-time")
        .arg("12")
        .arg("--silent")
        .arg("--show-error")
        .arg("--fail")
        .arg("--user-agent")
        .arg("SkillNotebook/0.1")
        .arg(url)
        .env("PATH", augmented_command_path_value())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("failed to fetch URL with curl: {}", error))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "URL fetch failed: {}",
            summarize_error(error.trim())
        ));
    }

    let bytes = if output.stdout.len() > URL_SOURCE_LIMIT_BYTES {
        &output.stdout[..URL_SOURCE_LIMIT_BYTES]
    } else {
        &output.stdout
    };
    let text = String::from_utf8_lossy(bytes)
        .replace('\0', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if text.trim().is_empty() {
        return Err("URL fetched successfully, but no readable text was found".to_string());
    }

    Ok(text)
}

fn resolve_source_path(project_root_path: &Path, raw_path: &str) -> Result<PathBuf, String> {
    let cleaned_path = normalize_source_path_input(raw_path);
    let candidate = Path::new(&cleaned_path);
    let path = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        project_root_path.join(candidate)
    };

    let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
        if cleaned_path == raw_path {
            format!("source path not found `{}`: {}", cleaned_path, error)
        } else {
            format!(
                "source path not found `{}` (normalized from `{}`): {}",
                cleaned_path, raw_path, error
            )
        }
    })?;
    if metadata.file_type().is_symlink() {
        return Err(format!("source path cannot be a symlink: {}", cleaned_path));
    }
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(format!(
            "source path is not a file or directory: {}",
            cleaned_path
        ));
    }

    Ok(path)
}

fn normalize_source_path_input(value: &str) -> String {
    let mut normalized = value.trim().to_string();
    loop {
        let Some(first) = normalized.chars().next() else {
            return normalized;
        };
        let Some(last) = normalized.chars().last() else {
            return normalized;
        };
        let quoted = (first == '"' && last == '"')
            || (first == '\'' && last == '\'')
            || (first == '“' && last == '”')
            || (first == '‘' && last == '’');
        if !quoted || normalized.chars().count() < 2 {
            break;
        }
        normalized = normalized
            .chars()
            .skip(1)
            .take(normalized.chars().count().saturating_sub(2))
            .collect::<String>()
            .trim()
            .to_string();
    }

    if normalized.starts_with("file://") {
        normalized = percent_decode_path(normalized.trim_start_matches("file://"));
    }

    unescape_pasted_path(&normalized)
}

fn unescape_pasted_path(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek().copied() {
                Some('\\' | ' ' | '"' | '\'' | '(' | ')' | ':') => {
                    output.push(chars.next().unwrap_or_default());
                }
                _ => output.push(ch),
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn percent_decode_path(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[index + 1..index + 3]) {
                if let Ok(decoded) = u8::from_str_radix(hex, 16) {
                    output.push(decoded);
                    index += 3;
                    continue;
                }
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).to_string()
}

fn summarize_source_names(paths: &[String]) -> String {
    let mut names = paths
        .iter()
        .filter_map(|path| Path::new(path).file_name().and_then(|name| name.to_str()))
        .filter(|name| !name.trim().is_empty())
        .take(3)
        .map(|name| format!("`{}`", name))
        .collect::<Vec<_>>();

    if names.is_empty() {
        names.push(format!("{} source path(s)", paths.len()));
    } else if paths.len() > names.len() {
        names.push(format!("and {} more", paths.len() - names.len()));
    }

    names.join(", ")
}

fn collect_source_files(
    project_root_path: &Path,
    path: &Path,
    files: &mut Vec<SourceFileSummary>,
) -> Result<(), String> {
    if files.len() >= SOURCE_FILE_LIMIT {
        return Ok(());
    }

    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {}: {}", path.display(), error))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        files.push(summarize_source_file(project_root_path, path, &metadata));
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }

    let mut entries = std::fs::read_dir(path)
        .map_err(|error| {
            format!(
                "failed to read source directory {}: {}",
                path.display(),
                error
            )
        })?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.to_string_lossy().cmp(&right.to_string_lossy()));

    for entry in entries {
        if files.len() >= SOURCE_FILE_LIMIT {
            break;
        }
        let name = entry
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_default();
        if should_skip_source_entry(&name) {
            continue;
        }
        collect_source_files(project_root_path, &entry, files)?;
    }

    Ok(())
}

fn summarize_source_file(
    project_root_path: &Path,
    path: &Path,
    metadata: &std::fs::Metadata,
) -> SourceFileSummary {
    let kind = path
        .extension()
        .map(|value| value.to_string_lossy().to_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "file".to_string());
    let excerpt = if is_text_source_file(path) {
        std::fs::read_to_string(path)
            .ok()
            .map(|content| truncate_chars(&content, SOURCE_EXCERPT_LIMIT_CHARS))
    } else {
        None
    };

    SourceFileSummary {
        path: display_source_path(project_root_path, path),
        size_bytes: metadata.len(),
        kind,
        excerpt,
    }
}

fn should_skip_source_entry(name: &str) -> bool {
    name.starts_with('.')
        || matches!(
            name,
            "node_modules"
                | "target"
                | "dist"
                | "build"
                | ".git"
                | ".42eval"
                | ".skill-notebook"
                | ".skills"
        )
}

fn is_text_source_file(path: &Path) -> bool {
    let Some(extension) = path
        .extension()
        .map(|value| value.to_string_lossy().to_lowercase())
    else {
        return false;
    };

    matches!(
        extension.as_str(),
        "md" | "txt"
            | "json"
            | "yaml"
            | "yml"
            | "csv"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "rs"
            | "sh"
            | "toml"
            | "html"
            | "css"
    )
}

fn display_source_path(project_root_path: &Path, path: &Path) -> String {
    path.strip_prefix(project_root_path)
        .map(|value| format!("./{}", value.to_string_lossy().replace('\\', "/")))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn truncate_chars(value: &str, limit: usize) -> String {
    let mut truncated = value.chars().take(limit).collect::<String>();
    if value.chars().count() > limit {
        truncated.push_str("...");
    }
    truncated
}

fn prepare_draft_result(
    project_root_path: &Path,
    req: &CreatePackageFromNlRequest,
    options: &CreatorBridgeOptions,
) -> Result<DraftResult, String> {
    let packages_root = filesystem::canonical_skills_root(project_root_path);
    filesystem::ensure_directory(&packages_root)?;

    let mut draft_result = build_initial_draft(req, options)?;
    let slug = unique_package_slug(&draft_result.draft.slug, &packages_root);
    draft_result.draft.slug = slug.clone();
    if draft_result.draft.name.trim().is_empty() {
        draft_result.draft.name = title_case_slug(&slug).replace("  ", " ");
    } else {
        draft_result.draft.name = sanitize_title(&draft_result.draft.name);
    }
    draft_result.draft.skill_md = normalize_skill_markdown(
        &draft_result.draft.skill_md,
        &slug,
        &draft_result.draft.description,
        &draft_result.draft.name,
        &summarize_goal(
            &normalize_text(&req.prompt),
            &normalize_text(req.context.as_deref().unwrap_or_default()),
            &draft_result.draft.tags,
        ),
        &draft_result.draft.tags,
        &draft_result.draft.expectations,
        &draft_result.draft.system_prompt,
    );

    Ok(draft_result)
}

fn build_initial_draft(
    req: &CreatePackageFromNlRequest,
    options: &CreatorBridgeOptions,
) -> Result<DraftResult, String> {
    let fallback = || fallback_draft_result(req, "Local template draft used.");
    let skill_create_available = command_exists(&options.skill_create_bin);
    let claude_available = command_exists(&options.claude_bin);

    match options.mode {
        CreatorMode::Template => Ok(fallback()),
        CreatorMode::SkillCreate => generate_with_skill_create_cli(req, options),
        CreatorMode::ClaudeCli => generate_with_claude_cli(req, options),
        CreatorMode::Auto => {
            if skill_create_available {
                match generate_with_skill_create_cli(req, options) {
                    Ok(result) => return Ok(result),
                    Err(error) => {
                        if claude_available {
                            match generate_with_claude_cli(req, options) {
                                Ok(result) => return Ok(result),
                                Err(claude_error) => return Err(format!(
                                    "skill-create draft generation failed and Claude CLI draft generation failed: {}; {}",
                                    summarize_error(&error),
                                    summarize_error(&claude_error)
                                )),
                            }
                        }

                        return Err(format!(
                            "skill-create draft generation failed: {}",
                            summarize_error(&error)
                        ));
                    }
                }
            }

            if !claude_available {
                return Ok(fallback_draft_result(
                    req,
                    "skill-create and Claude CLI were not found, so the local template draft was used.",
                ));
            }

            match generate_with_claude_cli(req, options) {
                Ok(result) => Ok(result),
                Err(error) => Err(format!(
                    "Claude CLI draft generation failed: {}",
                    summarize_error(&error)
                )),
            }
        }
    }
}

fn generate_with_skill_create_cli(
    req: &CreatePackageFromNlRequest,
    options: &CreatorBridgeOptions,
) -> Result<DraftResult, String> {
    if !command_exists(&options.skill_create_bin) {
        return Err(format!(
            "skill-create binary not found: {}",
            options.skill_create_bin
        ));
    }

    let prompt = build_generation_prompt(req.prompt.as_str(), req.context.as_deref());
    let response = call_skill_create_text(&prompt, options)?;
    let payload = parse_generator_draft_payload(&response)?;
    let draft = normalize_generator_draft(payload, req.prompt.as_str(), req.context.as_deref());

    Ok(DraftResult {
        draft,
        generator_used: CREATOR_SKILL_CREATE.to_string(),
        generation_summary: "Initial draft generated via skill-create.".to_string(),
        prompt_log: Some(prompt),
        response_log: Some(response),
    })
}

fn generate_with_claude_cli(
    req: &CreatePackageFromNlRequest,
    options: &CreatorBridgeOptions,
) -> Result<DraftResult, String> {
    if !command_exists(&options.claude_bin) {
        return Err(format!("claude binary not found: {}", options.claude_bin));
    }

    let prompt = build_generation_prompt(req.prompt.as_str(), req.context.as_deref());
    let response = call_claude_text(&prompt, options)?;
    let payload = parse_generator_draft_payload(&response)?;
    let draft = normalize_generator_draft(payload, req.prompt.as_str(), req.context.as_deref());

    Ok(DraftResult {
        draft,
        generator_used: CREATOR_CLAUDE.to_string(),
        generation_summary: if let Some(model) = &options.claude_model {
            format!(
                "Initial draft generated via Claude CLI using model `{}`.",
                model
            )
        } else {
            "Initial draft generated via Claude CLI.".to_string()
        },
        prompt_log: Some(prompt),
        response_log: Some(response),
    })
}

fn fallback_draft_result(req: &CreatePackageFromNlRequest, summary: &str) -> DraftResult {
    DraftResult {
        draft: derive_template_draft(req.prompt.as_str(), req.context.as_deref()),
        generator_used: CREATOR_FALLBACK.to_string(),
        generation_summary: summary.to_string(),
        prompt_log: None,
        response_log: None,
    }
}

fn call_claude_text(prompt: &str, options: &CreatorBridgeOptions) -> Result<String, String> {
    let claude_path = resolve_command_path(&options.claude_bin)
        .ok_or_else(|| format!("claude binary not found: {}", options.claude_bin))?;
    let mut command = Command::new(claude_path);
    command
        .arg("-p")
        .arg("--output-format")
        .arg("text")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR"))),
        );

    if let Some(model) = &options.claude_model {
        command.arg("--model").arg(model);
    }

    command.env_remove("CLAUDECODE");
    command.env("PATH", augmented_command_path_value());

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to spawn Claude CLI: {}", error))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|error| format!("failed to write Claude prompt: {}", error))?;
    }

    let timeout = Duration::from_secs(options.claude_timeout_secs);
    let started_at = Instant::now();

    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed waiting for Claude CLI: {}", error))?
        {
            let stdout = read_child_stream(child.stdout.take(), "Claude CLI", "stdout")?;
            let stderr = read_child_stream(child.stderr.take(), "Claude CLI", "stderr")?;

            if !status.success() {
                return Err(format!(
                    "Claude CLI exited {}: {}",
                    status,
                    summarize_process_output(&stdout, &stderr)
                ));
            }

            return Ok(stdout.trim().to_string());
        }

        if started_at.elapsed() >= timeout {
            child.kill().ok();
            child.wait().ok();
            let stderr =
                read_child_stream(child.stderr.take(), "Claude CLI", "stderr").unwrap_or_default();
            let details = if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(" {}", summarize_error(stderr.trim()))
            };
            return Err(format!(
                "Claude CLI timed out after {}s.{} Set SKILL_NOTEBOOK_CLAUDE_TIMEOUT_SECS to a larger value for long source materials.",
                options.claude_timeout_secs, details
            ));
        }

        thread::sleep(Duration::from_millis(CLAUDE_POLL_INTERVAL_MS));
    }
}

fn call_skill_create_text(prompt: &str, options: &CreatorBridgeOptions) -> Result<String, String> {
    let skill_create_path = resolve_command_path(&options.skill_create_bin).ok_or_else(|| {
        format!(
            "skill-create binary not found: {}",
            options.skill_create_bin
        )
    })?;
    let mut command = Command::new(skill_create_path);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR"))),
        );
    command.env("PATH", augmented_command_path_value());

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to spawn skill-create: {}", error))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|error| format!("failed to write skill-create prompt: {}", error))?;
    }

    let timeout = Duration::from_secs(options.skill_create_timeout_secs);
    let started_at = Instant::now();

    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed waiting for skill-create: {}", error))?
        {
            let stdout = read_child_stream(child.stdout.take(), "skill-create", "stdout")?;
            let stderr = read_child_stream(child.stderr.take(), "skill-create", "stderr")?;

            if !status.success() {
                return Err(format!(
                    "skill-create exited {}: {}",
                    status,
                    summarize_process_output(&stdout, &stderr)
                ));
            }

            return Ok(stdout.trim().to_string());
        }

        if started_at.elapsed() >= timeout {
            child.kill().ok();
            child.wait().ok();
            let stderr = read_child_stream(child.stderr.take(), "skill-create", "stderr")
                .unwrap_or_default();
            let details = if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(" {}", summarize_error(stderr.trim()))
            };
            return Err(format!(
                "skill-create timed out after {}s.{}",
                options.skill_create_timeout_secs, details
            ));
        }

        thread::sleep(Duration::from_millis(CLAUDE_POLL_INTERVAL_MS));
    }
}

fn read_child_stream<T: Read>(
    stream: Option<T>,
    tool_label: &str,
    stream_label: &str,
) -> Result<String, String> {
    let Some(mut stream) = stream else {
        return Ok(String::new());
    };

    let mut buffer = String::new();
    stream
        .read_to_string(&mut buffer)
        .map_err(|error| format!("failed to read {} {}: {}", tool_label, stream_label, error))?;
    Ok(buffer)
}

fn build_generation_prompt(prompt: &str, context: Option<&str>) -> String {
    format!(
        "Draft a reusable skill package from the request below.\n\nReturn only one JSON object wrapped in <draft_json> tags. Do not include markdown code fences.\n\nThe JSON schema is:\n{{\n  \"name\": \"Human-readable title\",\n  \"slug\": \"kebab-case skill id, max 64 chars, lowercase letters/numbers/hyphens only, and never containing reserved words claude or anthropic\",\n  \"description\": \"1-2 sentences describing what the skill does and when to use it. It must include the phrase 'Use when'.\",\n  \"skill_md\": \"Full SKILL.md content including YAML frontmatter and markdown body.\",\n  \"system_prompt\": \"System framing text for prompts/system.md\",\n  \"task_prompt\": \"Execution checklist for prompts/task.md\",\n  \"example_markdown\": \"A short markdown example showing input and output\",\n  \"smoke_prompt\": \"A realistic smoke-test prompt\",\n  \"expected_output\": \"What success looks like\",\n  \"expectations\": [\"A few objective checks\"],\n  \"tags\": [\"up to 4 lowercase topic tags\"]\n}}\n\nRules:\n- ASCII only.\n- Keep the skill focused on the user's intent, not implementation trivia.\n- SKILL.md must contain: Overview, When to Use, When Not to Use, Inputs, Outputs, Workflow, Quick Reference, Resources.\n- The frontmatter name must match the slug.\n- The description should be specific and a little pushy about when to trigger.\n- The workflow should feel like a real reusable skill, not placeholder text.\n- Keep it concise enough to validate cleanly.\n\nUser request:\n{}\n\nAdditional context:\n{}\n",
        prompt.trim(),
        context.unwrap_or("None provided.").trim()
    )
}

fn parse_generator_draft_payload(response: &str) -> Result<GeneratorDraftPayload, String> {
    let raw_json = extract_tagged_payload(response, "draft_json").unwrap_or(response);
    let cleaned = strip_code_fences(raw_json);
    serde_json::from_str::<GeneratorDraftPayload>(cleaned.trim()).map_err(|error| {
        format!(
            "failed to parse generator draft JSON: {}",
            summarize_error(&format!("{}\nresponse: {}", error, cleaned))
        )
    })
}

fn normalize_generator_draft(
    payload: GeneratorDraftPayload,
    prompt: &str,
    context: Option<&str>,
) -> DraftPackage {
    let fallback = derive_template_draft(prompt, context);
    let mut tags = merge_unique_strings(payload.tags, fallback.tags.clone(), 4);
    if tags.is_empty() {
        tags = fallback.tags.clone();
    }

    let slug = sanitize_skill_slug(&payload.slug, &payload.name, &tags, &fallback.slug);
    let name = if payload.name.trim().is_empty() {
        title_case_slug(&slug)
    } else {
        sanitize_title(&payload.name)
    };
    let description = normalize_description(&payload.description, &tags, &fallback.description);
    let expectations = normalize_expectations(&payload.expectations, &fallback.expectations);
    let system_prompt = nonempty_or(&payload.system_prompt, &fallback.system_prompt);
    let task_prompt = nonempty_or(&payload.task_prompt, &fallback.task_prompt);
    let example_markdown = nonempty_or(&payload.example_markdown, &fallback.example_markdown);
    let smoke_prompt = nonempty_or(&payload.smoke_prompt, &fallback.smoke_prompt);
    let expected_output = nonempty_or(&payload.expected_output, &fallback.expected_output);
    let summary = summarize_goal(
        &normalize_text(prompt),
        &normalize_text(context.unwrap_or_default()),
        &tags,
    );
    let skill_md = normalize_skill_markdown(
        &payload.skill_md,
        &slug,
        &description,
        &name,
        &summary,
        &tags,
        &expectations,
        &system_prompt,
    );

    DraftPackage {
        name,
        slug,
        description,
        tags,
        skill_md,
        system_prompt,
        task_prompt,
        example_markdown,
        smoke_prompt,
        expected_output,
        expectations,
    }
}

fn normalize_skill_markdown(
    skill_md: &str,
    slug: &str,
    description: &str,
    name: &str,
    summary: &str,
    tags: &[String],
    expectations: &[String],
    system_prompt: &str,
) -> String {
    let fallback = build_skill_md(
        slug,
        description,
        name,
        summary,
        tags,
        expectations,
        system_prompt,
    );

    let body = strip_existing_frontmatter(skill_md).trim().to_string();
    if body.is_empty() {
        return fallback;
    }

    let mut normalized_body = body;
    ensure_section(
        &mut normalized_body,
        "## When to Use",
        &format!(
            "- Use when the user mentions {}.\n- Use when this workflow should be repeated reliably.\n",
            human_keyword_list(tags)
        ),
    );
    ensure_section(
        &mut normalized_body,
        "## When Not to Use",
        "- Do not use for unrelated requests.\n- Ask for missing source material instead of guessing.\n",
    );
    ensure_section(
        &mut normalized_body,
        "## Inputs",
        "- primary task request\n- optional supporting context or files\n",
    );
    ensure_section(
        &mut normalized_body,
        "## Outputs",
        &format!(
            "- a structured deliverable for {}\n- follow-up notes and risks\n",
            summary
        ),
    );
    ensure_section(
        &mut normalized_body,
        "## Workflow",
        "1. Restate the goal.\n2. Inspect the provided material.\n3. Apply the workflow.\n4. Return a structured result.\n",
    );
    ensure_section(
        &mut normalized_body,
        "## Quick Reference",
        "| Operation | How |\n|-----------|-----|\n| Draft the result | Follow `prompts/task.md` |\n| Stay on-brief | Use `prompts/system.md` |\n",
    );
    ensure_section(
        &mut normalized_body,
        "## Resources",
        "- `prompts/` - Task framing.\n- `examples/` - Sample output.\n- `evals/` - Re-runnable expectations.\n",
    );

    format!(
        "---\nname: {}\ndescription: {}\nmetadata:\n  author: skill-notebook\n  version: 0.1.0\n---\n\n{}\n",
        slug,
        yaml_safe_string(description),
        normalized_body.trim()
    )
}

fn build_skill_md(
    slug: &str,
    description: &str,
    name: &str,
    summary: &str,
    tags: &[String],
    expectations: &[String],
    system_prompt: &str,
) -> String {
    format!(
        "---\nname: {}\ndescription: {}\nmetadata:\n  author: skill-notebook\n  version: 0.1.0\n---\n\n# {}\n\n## Overview\n\nThis skill helps with {} while keeping the output consistent and reusable.\n\n## When to Use\n\n- Use when the user mentions {}.\n- Use when this workflow should become a repeatable skill package.\n\n## When Not to Use\n\n- Do not use for unrelated tasks that do not share the same workflow.\n- Do not invent missing source material; ask for the necessary inputs instead.\n\n## Inputs\n\n- primary task request\n- optional supporting context or source files\n- any constraints that affect the final deliverable\n\n## Outputs\n\n- a structured final deliverable for {}\n- concise notes about gaps, risks, or follow-up actions\n\n## Workflow\n\n1. Restate the goal and confirm the expected outcome.\n2. Inspect the provided material before making decisions.\n3. Apply the workflow in a clear sequence.\n4. Return the final deliverable in a structured format.\n5. Surface missing inputs or uncertainty before finishing.\n\n## Quick Reference\n\n| Operation | How |\n|-----------|-----|\n| Draft the result | Follow the task prompt in `prompts/task.md` |\n| Stay on-brief | Use the framing in `prompts/system.md` |\n| Check quality | Review `evals/evals.json` and `tests/smoke-test.json` |\n\n## Quality Checks\n\n- {}\n- {}\n- {}\n\n## Resources\n\n- `prompts/` - System and task framing for the workflow.\n- `examples/` - Concrete sample output.\n- `evals/` - Re-runnable expectations for structural review.\n- `tests/` - Smoke-test metadata for quick checks.\n\n<!-- system prompt seed: {} -->\n",
        slug,
        yaml_safe_string(description),
        name,
        summary,
        human_keyword_list(tags),
        summary,
        expectations.first().cloned().unwrap_or_default(),
        expectations.get(1).cloned().unwrap_or_default(),
        expectations.get(2).cloned().unwrap_or_default(),
        sanitize_inline_comment(system_prompt)
    )
}

fn build_preview_id(slug: &str) -> String {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let raw = format!("preview-{}-{}-{}", slug, std::process::id(), nonce);
    slugify(&raw)
}

fn safe_preview_id(value: &str) -> Result<String, String> {
    let normalized = slugify(value);
    if normalized.is_empty() || normalized != value {
        return Err("invalid preview id".to_string());
    }

    Ok(normalized)
}

fn preview_dir(project_root_path: &Path, preview_id: &str) -> PathBuf {
    project_root_path
        .join(".skill-notebook")
        .join("create-previews")
        .join(preview_id)
}

fn create_preview_dir(project_root_path: &Path, preview_id: &str) -> Result<PathBuf, String> {
    let preview_id = safe_preview_id(preview_id)?;
    let path = preview_dir(project_root_path, &preview_id);

    if path.exists() {
        std::fs::remove_dir_all(&path)
            .map_err(|error| format!("failed to reset preview {}: {}", path.display(), error))?;
    }

    filesystem::ensure_directory(&path.join("package"))?;
    Ok(path)
}

fn read_preview_manifest(path: &Path) -> Result<CreatePreviewManifest, String> {
    let content = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read preview manifest {}: {}",
            path.display(),
            error
        )
    })?;
    serde_json::from_str(&content).map_err(|error| {
        format!(
            "failed to parse preview manifest {}: {}",
            path.display(),
            error
        )
    })
}

fn draft_preview_files(
    draft: &DraftPackage,
    slug: &str,
) -> Result<Vec<PackagePreviewFile>, String> {
    let smoke_test = serde_json::to_string_pretty(&json!({
        "name": "smoke-test",
        "package": slug,
        "prompt": draft.smoke_prompt,
        "expectedOutput": draft.expected_output,
        "checks": draft.expectations,
    }))
    .map_err(|error| format!("failed to serialize smoke test: {}", error))?;

    let evals = serde_json::to_string_pretty(&json!({
        "skill_name": slug,
        "evals": [
            {
                "id": 1,
                "prompt": draft.smoke_prompt,
                "expected_output": draft.expected_output,
                "files": [],
                "expectations": draft.expectations,
            }
        ]
    }))
    .map_err(|error| format!("failed to serialize eval definitions: {}", error))?;

    Ok(vec![
        PackagePreviewFile {
            path: "SKILL.md".to_string(),
            content: draft.skill_md.clone(),
            encoding: "utf-8".to_string(),
        },
        PackagePreviewFile {
            path: "prompts/system.md".to_string(),
            content: draft.system_prompt.clone(),
            encoding: "utf-8".to_string(),
        },
        PackagePreviewFile {
            path: "prompts/task.md".to_string(),
            content: draft.task_prompt.clone(),
            encoding: "utf-8".to_string(),
        },
        PackagePreviewFile {
            path: "examples/example-01.md".to_string(),
            content: draft.example_markdown.clone(),
            encoding: "utf-8".to_string(),
        },
        PackagePreviewFile {
            path: "tests/smoke-test.json".to_string(),
            content: format!("{}\n", smoke_test),
            encoding: "utf-8".to_string(),
        },
        PackagePreviewFile {
            path: "evals/evals.json".to_string(),
            content: format!("{}\n", evals),
            encoding: "utf-8".to_string(),
        },
    ])
}

fn write_preview_files(root: &Path, files: &[PackagePreviewFile]) -> Result<(), String> {
    for file in files {
        filesystem::write_text_file(&root.join(&file.path), &file.content)?;
    }

    Ok(())
}

fn write_draft_files(package_root: &Path, draft: &DraftPackage, slug: &str) -> Result<(), String> {
    let files = draft_preview_files(draft, slug)?;
    write_preview_files(package_root, &files)
}

fn write_generator_log(
    project_root_path: &Path,
    slug: &str,
    generator_used: &str,
    generation_summary: &str,
    prompt_log: Option<&str>,
    response_log: Option<&str>,
) -> Result<(), String> {
    if prompt_log.is_none() && response_log.is_none() {
        return Ok(());
    }

    let path = project_root_path
        .join(".skill-notebook")
        .join("generator-runs")
        .join(slug)
        .join("create-from-nl.json");
    filesystem::write_json_file(
        &path,
        &json!({
            "generatorUsed": generator_used,
            "generationSummary": generation_summary,
            "createdAt": now_iso(),
            "prompt": prompt_log,
            "response": response_log,
        }),
    )
}

fn derive_template_draft(prompt: &str, context: Option<&str>) -> DraftPackage {
    let normalized_prompt = normalize_text(prompt);
    let normalized_context = normalize_text(context.unwrap_or_default());
    let keyword_source = format!("{} {}", normalized_prompt, normalized_context);
    let mut keywords = extract_keywords(&keyword_source);
    if keywords.is_empty() {
        keywords = vec!["workflow".to_string(), "skill".to_string()];
    }

    let base_slug = keywords
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("-");
    let safe_slug = sanitize_skill_slug(
        &base_slug,
        "",
        &keywords,
        &format!("skill-{}", today_slug()),
    );
    let name = title_case_slug(&safe_slug);
    let summary = summarize_goal(&normalized_prompt, &normalized_context, &keywords);
    let description = normalize_description(
        &format!(
            "Structures a reusable workflow for {}. Use when the user mentions {} or asks to turn this task into a repeatable skill.",
            summary,
            human_keyword_list(&keywords)
        ),
        &keywords,
        "Structures a reusable workflow. Use when this task should become a repeatable skill.",
    );

    let system_prompt = format!(
        "You are the {} skill. Stay within the package instructions, clarify missing inputs, and return a structured result instead of free-form rambling.\n\nFocus keywords: {}.\n",
        name,
        keywords.join(", ")
    );
    let task_prompt = format!(
        "1. Confirm the goal in one sentence.\n2. Inspect the provided inputs before deciding on output shape.\n3. Produce the final deliverable for {}.\n4. Call out uncertainty, missing data, and follow-up recommendations.\n",
        summary
    );
    let example_markdown = format!(
        "## Example\n\nInput: {}\n\nOutput:\n- Goal: {}\n- Key steps: review inputs, apply the workflow, present a structured result\n- Risks: missing context or incomplete source material\n",
        if normalized_prompt.is_empty() {
            "Create a reusable skill package from this request."
        } else {
            normalized_prompt.as_str()
        },
        summary
    );
    let expectations = vec![
        "SKILL.md frontmatter validates successfully.".to_string(),
        "The package describes when to use the skill and what it outputs.".to_string(),
        "Prompt, example, and eval files are present.".to_string(),
    ];

    let skill_md = build_skill_md(
        &safe_slug,
        &description,
        &name,
        &summary,
        &keywords,
        &expectations,
        &system_prompt,
    );

    DraftPackage {
        name,
        slug: safe_slug,
        description,
        tags: keywords.iter().take(4).cloned().collect(),
        skill_md,
        system_prompt,
        task_prompt,
        example_markdown,
        smoke_prompt: format!(
            "Use this skill for the following request: {}",
            if normalized_prompt.is_empty() {
                "Create a reusable workflow from the provided task."
            } else {
                normalized_prompt.as_str()
            }
        ),
        expected_output: format!(
            "A structured deliverable for {} with clear steps, output sections, and follow-up notes.",
            summary
        ),
        expectations,
    }
}

fn sanitize_skill_slug(
    slug_hint: &str,
    name_hint: &str,
    keywords: &[String],
    fallback: &str,
) -> String {
    let mut candidate = if !slug_hint.trim().is_empty() {
        slugify(slug_hint)
    } else if !name_hint.trim().is_empty() {
        slugify(name_hint)
    } else if !keywords.is_empty() {
        slugify(
            &keywords
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("-"),
        )
    } else {
        slugify(fallback)
    };

    if candidate.contains("claude") || candidate.contains("anthropic") {
        candidate = candidate
            .split('-')
            .filter(|segment| *segment != "claude" && *segment != "anthropic")
            .collect::<Vec<_>>()
            .join("-");
    }

    if candidate.is_empty() {
        candidate = slugify(fallback);
    }

    if candidate.len() > 64 {
        candidate = candidate.chars().take(64).collect::<String>();
        candidate = candidate.trim_matches('-').to_string();
    }

    if candidate.is_empty() {
        format!("skill-{}", today_slug())
    } else {
        candidate
    }
}

fn normalize_description(candidate: &str, keywords: &[String], fallback: &str) -> String {
    let mut description = normalize_text(candidate);
    if description.is_empty() {
        description = fallback.to_string();
    }

    if !description.to_lowercase().contains("use when") {
        let suffix = format!(
            " Use when the user mentions {} or asks for this workflow explicitly.",
            human_keyword_list(keywords)
        );
        description = format!("{}{}", description.trim_end_matches('.'), suffix);
    }

    let description = description
        .replace('<', "")
        .replace('>', "")
        .chars()
        .take(1024)
        .collect::<String>();

    description.trim().to_string()
}

fn normalize_expectations(candidate: &[String], fallback: &[String]) -> Vec<String> {
    let mut expectations = candidate
        .iter()
        .map(|value| normalize_text(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if expectations.is_empty() {
        return fallback.to_vec();
    }

    expectations.truncate(4);
    expectations
}

fn merge_unique_strings(primary: Vec<String>, secondary: Vec<String>, limit: usize) -> Vec<String> {
    let mut merged = Vec::new();
    for item in primary.into_iter().chain(secondary.into_iter()) {
        let normalized = normalize_text(&item).to_lowercase();
        if normalized.is_empty() || merged.contains(&normalized) {
            continue;
        }
        merged.push(normalized);
        if merged.len() == limit {
            break;
        }
    }
    merged
}

fn nonempty_or(value: &str, fallback: &str) -> String {
    let normalized = normalize_text(value);
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

fn ensure_section(content: &mut String, heading: &str, body: &str) {
    if content.contains(heading) {
        return;
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(heading);
    content.push_str("\n\n");
    content.push_str(body.trim());
    content.push('\n');
}

fn strip_existing_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return trimmed.to_string();
    }

    let mut lines = trimmed.lines();
    if lines.next() != Some("---") {
        return trimmed.to_string();
    }

    let mut seen_end = false;
    let mut remainder = Vec::new();
    for line in lines {
        if seen_end {
            remainder.push(line);
        } else if line == "---" {
            seen_end = true;
        }
    }

    if seen_end {
        remainder.join("\n")
    } else {
        trimmed.to_string()
    }
}

fn strip_code_fences(content: &str) -> String {
    let trimmed = content.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

fn extract_tagged_payload<'a>(content: &'a str, tag: &str) -> Option<&'a str> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    let start = content.find(&start_tag)? + start_tag.len();
    let end = content[start..].find(&end_tag)? + start;
    Some(&content[start..end])
}

fn command_exists(command: &str) -> bool {
    resolve_command_path(command).is_some()
}

fn resolve_command_path(command: &str) -> Option<PathBuf> {
    if command.contains('/') {
        let path = PathBuf::from(command);
        return path.exists().then_some(path);
    }

    command_search_paths()
        .into_iter()
        .map(|path| path.join(command))
        .find(|path| path.exists())
}

fn command_search_paths() -> Vec<PathBuf> {
    let mut paths = env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        paths.push(home.join(".local").join("bin"));
        paths.push(home.join(".cargo").join("bin"));
        paths.push(home.join(".bun").join("bin"));
        paths.push(home.join("Library").join("pnpm"));
    }

    paths.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/opt/homebrew/sbin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/sbin"),
    ]);

    let mut unique_paths = Vec::new();
    for path in paths {
        if !unique_paths.iter().any(|existing| existing == &path) {
            unique_paths.push(path);
        }
    }
    unique_paths
}

fn augmented_command_path_value() -> String {
    env::join_paths(command_search_paths())
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn summarize_error(error: &str) -> String {
    let one_line = error.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut summary = one_line.chars().take(180).collect::<String>();
    if one_line.chars().count() > 180 {
        summary.push_str("...");
    }
    summary
}

fn summarize_process_output(stdout: &str, stderr: &str) -> String {
    let mut parts = Vec::new();
    let stdout = stdout.trim();
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        parts.push(format!("stderr: {}", stderr));
    }
    if !stdout.is_empty() {
        parts.push(format!("stdout: {}", stdout));
    }
    if parts.is_empty() {
        return "no output".to_string();
    }
    summarize_error(&parts.join(" "))
}

fn sanitize_title(value: &str) -> String {
    normalize_text(value)
        .split(' ')
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn unique_package_slug(base_slug: &str, packages_root: &Path) -> String {
    let root_slug = if base_slug.trim().is_empty() {
        format!("skill-{}", today_slug())
    } else {
        base_slug.to_string()
    };

    if !packages_root.join(&root_slug).exists() {
        return root_slug;
    }

    for suffix in 2..1000 {
        let candidate = format!("{}-{}", root_slug, suffix);
        if !packages_root.join(&candidate).exists() {
            return candidate;
        }
    }

    format!("{}-{}", root_slug, today_slug())
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_keywords(content: &str) -> Vec<String> {
    let stop_words = [
        "a", "an", "and", "as", "be", "build", "create", "for", "from", "help", "into", "of", "on",
        "or", "please", "skill", "task", "that", "the", "this", "to", "turn", "with", "workflow",
        "you",
    ];

    let mut keywords = Vec::new();
    for word in content
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
    {
        let lowered = word.to_lowercase();
        if lowered.len() < 3 || stop_words.contains(&lowered.as_str()) {
            continue;
        }
        if !keywords.contains(&lowered) {
            keywords.push(lowered);
        }
    }

    keywords
}

fn summarize_goal(prompt: &str, context: &str, keywords: &[String]) -> String {
    if !prompt.is_empty() {
        let trimmed = prompt.trim_end_matches(|character| {
            matches!(character, '.' | '!' | '?' | '。' | '！' | '？')
        });
        return trimmed.chars().take(96).collect::<String>();
    }

    if !context.is_empty() {
        return context.chars().take(96).collect::<String>();
    }

    if keywords.is_empty() {
        "the requested workflow".to_string()
    } else {
        format!("{} work", human_keyword_list(keywords))
    }
}

fn human_keyword_list(keywords: &[String]) -> String {
    let items = keywords.iter().take(3).cloned().collect::<Vec<_>>();
    match items.as_slice() {
        [] => "this workflow".to_string(),
        [only] => only.clone(),
        [first, second] => format!("{} and {}", first, second),
        [first, second, third] => format!("{}, {}, and {}", first, second, third),
        _ => "this workflow".to_string(),
    }
}

fn title_case_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn yaml_safe_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn sanitize_inline_comment(value: &str) -> String {
    value.replace('\n', " ").replace("--", "-")
}

impl CreatorMode {
    fn as_str(self) -> &'static str {
        match self {
            CreatorMode::Auto => "auto",
            CreatorMode::Template => "template",
            CreatorMode::SkillCreate => "skill_create",
            CreatorMode::ClaudeCli => "claude_cli",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::{env, fs};

    use crate::storage::filesystem;
    use crate::utils::time::{now_iso, parse_iso};
    use serde_json::json;
    use time::Duration as TimeDuration;

    use super::{
        cleanup_stale_package_previews_in_workspace_with_now, commit_package_preview_in_workspace,
        create_package_in_workspace_with_options, creator_bridge_status,
        discard_package_preview_in_workspace,
        generate_package_preview_from_sources_in_workspace_with_options,
        generate_package_preview_in_workspace_with_options, read_preview_manifest,
        unique_package_slug, CommitPackagePreviewRequest, CreatePackageFromNlRequest,
        CreatePackageFromSourcesRequest, CreatorBridgeOptions, CreatorMode,
        DiscardPackagePreviewRequest, CREATOR_CLAUDE, CREATOR_FALLBACK, CREATOR_SKILL_CREATE,
        DEFAULT_CLAUDE_BINARY, DEFAULT_CLAUDE_TIMEOUT_SECS, DEFAULT_SKILL_CREATE_BINARY,
        DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
    };

    fn make_temp_project_root(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "skill-notebook-create-{}-{}",
            name,
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).ok();
        }

        fs::create_dir_all(root.join(".skill-notebook")).expect("project_root config dir");
        fs::create_dir_all(filesystem::canonical_skills_root(&root)).expect("skills dir");
        filesystem::write_text_file(
            &root.join(".skill-notebook").join("config.json"),
            &format!(
                "{{\"id\":\"project_root-test\",\"name\":\"Test ProjectRoot\",\"createdAt\":\"{}\",\"updatedAt\":\"{}\"}}",
                now_iso(),
                now_iso()
            ),
        )
        .expect("project_root config");
        root
    }

    fn template_options() -> CreatorBridgeOptions {
        CreatorBridgeOptions {
            mode: CreatorMode::Template,
            skill_create_bin: DEFAULT_SKILL_CREATE_BINARY.to_string(),
            skill_create_timeout_secs: DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
            claude_bin: DEFAULT_CLAUDE_BINARY.to_string(),
            claude_model: None,
            claude_timeout_secs: DEFAULT_CLAUDE_TIMEOUT_SECS,
        }
    }

    fn set_preview_created_at(preview_root: &PathBuf, created_at: &str) {
        let manifest_path = preview_root.join("preview.json");
        let content = fs::read_to_string(&manifest_path).expect("read preview manifest");
        let mut manifest: serde_json::Value =
            serde_json::from_str(&content).expect("parse preview manifest");
        manifest["createdAt"] = json!(created_at);
        filesystem::write_text_file(
            &manifest_path,
            &format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
        )
        .expect("write preview manifest");
    }

    #[test]
    fn allocates_unique_slugs_when_the_base_exists() {
        let root = make_temp_project_root("slug");
        fs::create_dir_all(filesystem::canonical_skills_root(&root).join("meeting-actions"))
            .expect("existing package");

        let slug =
            unique_package_slug("meeting-actions", &filesystem::canonical_skills_root(&root));

        assert_eq!(slug, "meeting-actions-2");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn creates_a_real_package_from_template_mode() {
        let root = make_temp_project_root("create");
        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Turn customer interview notes into recurring action items and themes."
                .to_string(),
            context: Some("The package should help with synthesis and follow-up.".to_string()),
        };

        let response = create_package_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &template_options(),
        )
        .expect("package created");

        assert!(filesystem::canonical_skills_root(&root)
            .join(&response.slug)
            .join("SKILL.md")
            .exists());
        assert!(root
            .join(".42eval")
            .join(&response.slug)
            .join("config.json")
            .exists());
        assert_eq!(response.generator_used, CREATOR_FALLBACK);
        assert!(response.auto_eval_started);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn previews_a_generated_package_before_committing_it() {
        let root = make_temp_project_root("preview");
        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for extracting contract renewal risks.".to_string(),
            context: Some("The output should be a checklist with evidence notes.".to_string()),
        };

        let preview = generate_package_preview_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &template_options(),
        )
        .expect("preview created");
        let preview_root = root
            .join(".skill-notebook")
            .join("create-previews")
            .join(&preview.preview_id);

        assert_eq!(preview.project_root_id, "project_root-test");
        assert!(preview.files.iter().any(|file| file.path == "SKILL.md"));
        assert!(preview_root.join("package").join("SKILL.md").exists());
        assert!(!filesystem::canonical_skills_root(&root)
            .join(&preview.slug)
            .exists());

        let response = commit_package_preview_in_workspace(
            &root,
            &CommitPackagePreviewRequest {
                project_root_id: "project_root-test".to_string(),
                preview_id: preview.preview_id.clone(),
            },
        )
        .expect("preview committed");

        assert_eq!(response.slug, preview.slug);
        assert!(filesystem::canonical_skills_root(&root)
            .join(&response.slug)
            .join("SKILL.md")
            .exists());
        assert!(root
            .join(".42eval")
            .join(&response.slug)
            .join("config.json")
            .exists());
        assert!(!preview_root.exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn discards_a_generated_preview_without_creating_a_package() {
        let root = make_temp_project_root("preview-discard");
        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for reviewing support escalations.".to_string(),
            context: None,
        };

        let preview = generate_package_preview_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &template_options(),
        )
        .expect("preview created");
        let preview_root = root
            .join(".skill-notebook")
            .join("create-previews")
            .join(&preview.preview_id);

        let discarded = discard_package_preview_in_workspace(
            &root,
            &DiscardPackagePreviewRequest {
                project_root_id: "project_root-test".to_string(),
                preview_id: preview.preview_id.clone(),
            },
        )
        .expect("preview discarded");

        assert!(discarded);
        assert!(!preview_root.exists());
        assert!(!filesystem::canonical_skills_root(&root)
            .join(&preview.slug)
            .exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn previews_a_package_from_local_source_paths() {
        let root = make_temp_project_root("preview-sources");
        let source_dir = root.join("research-notes");
        filesystem::write_text_file(
            &source_dir.join("interview-notes.md"),
            "# Interview Notes\n\nUsers need clearer onboarding examples and evidence-linked insights.\n",
        )
        .expect("source note");
        filesystem::write_text_file(
            &source_dir.join("raw.json"),
            "{\"theme\":\"onboarding\",\"need\":\"examples\"}\n",
        )
        .expect("source json");
        let request = CreatePackageFromSourcesRequest {
            project_root_id: "project_root-test".to_string(),
            source_paths: vec![format!("\"{}\"", source_dir.display())],
            prompt: Some(
                "Create a skill for turning research notes into insight cards.".to_string(),
            ),
            context: Some("Keep evidence visible in the output.".to_string()),
        };

        let preview = generate_package_preview_from_sources_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &template_options(),
        )
        .expect("source preview created");
        let inventory = preview
            .files
            .iter()
            .find(|file| file.path == "references/source-inventory.md")
            .expect("inventory reference");

        assert!(inventory.content.contains("interview-notes.md"));
        assert!(inventory.content.contains("evidence-linked insights"));
        assert!(preview
            .generation_summary
            .contains("Source inventory attached"));
        let manifest = read_preview_manifest(
            &root
                .join(".skill-notebook")
                .join("create-previews")
                .join(&preview.preview_id)
                .join("preview.json"),
        )
        .expect("preview manifest");
        assert_eq!(manifest.generation_summary, preview.generation_summary);
        assert!(preview
            .file_tree
            .iter()
            .any(|entry| entry.path == "references" && entry.is_directory));
        assert!(!filesystem::canonical_skills_root(&root)
            .join(&preview.slug)
            .exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn failed_preview_commit_cleans_partial_package_and_keeps_preview() {
        let root = make_temp_project_root("preview-commit-fail");
        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for checking vendor renewal risk.".to_string(),
            context: None,
        };

        let preview = generate_package_preview_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &template_options(),
        )
        .expect("preview created");
        let preview_root = root
            .join(".skill-notebook")
            .join("create-previews")
            .join(&preview.preview_id);
        let package_root = filesystem::canonical_skills_root(&root).join(&preview.slug);
        filesystem::write_text_file(&root.join(".42eval").join(&preview.slug), "block eval dir")
            .expect("eval path blocker");

        let error = commit_package_preview_in_workspace(
            &root,
            &CommitPackagePreviewRequest {
                project_root_id: "project_root-test".to_string(),
                preview_id: preview.preview_id.clone(),
            },
        )
        .expect_err("commit should fail when eval workspace cannot be created");

        assert!(error.contains("failed to create directory"));
        assert!(!package_root.exists());
        assert!(preview_root.exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn cleanup_removes_only_expired_preview_workspaces() {
        let root = make_temp_project_root("preview-ttl");
        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for triaging procurement notes.".to_string(),
            context: None,
        };
        let other_request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for triaging launch notes.".to_string(),
            context: None,
        };

        let stale_preview = generate_package_preview_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &template_options(),
        )
        .expect("stale preview created");
        let fresh_preview = generate_package_preview_in_workspace_with_options(
            &root,
            "project_root-test",
            &other_request,
            &template_options(),
        )
        .expect("fresh preview created");
        let stale_root = root
            .join(".skill-notebook")
            .join("create-previews")
            .join(&stale_preview.preview_id);
        let fresh_root = root
            .join(".skill-notebook")
            .join("create-previews")
            .join(&fresh_preview.preview_id);

        set_preview_created_at(&stale_root, "2026-04-26T00:00:00Z");
        set_preview_created_at(&fresh_root, "2026-04-27T12:00:00Z");

        let removed = cleanup_stale_package_previews_in_workspace_with_now(
            &root,
            TimeDuration::hours(24),
            parse_iso("2026-04-28T00:00:00Z").expect("fixed now"),
        )
        .expect("cleanup previews");

        assert_eq!(removed, 1);
        assert!(!stale_root.exists());
        assert!(fresh_root.exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn uses_mocked_claude_cli_when_available() {
        let root = make_temp_project_root("claude");
        let mock_bin = root.join("mock-claude.sh");
        filesystem::write_text_file(
            &mock_bin,
            "#!/bin/sh\ncat <<'EOF'\n<draft_json>{\"name\":\"Meeting Mapper\",\"slug\":\"meeting-mapper\",\"description\":\"Maps meeting notes into actions. Use when the user needs action-oriented synthesis.\",\"skill_md\":\"---\\nname: meeting-mapper\\ndescription: \\\"Maps meeting notes into actions. Use when the user needs action-oriented synthesis.\\\"\\n---\\n\\n# Meeting Mapper\\n\\n## Overview\\nTransform notes into owners and deadlines.\\n\",\"system_prompt\":\"Stay concise and action-oriented.\",\"task_prompt\":\"1. Read the notes.\\n2. Extract owners and deadlines.\",\"example_markdown\":\"## Example\\n\\nInput: sprint planning notes\\n\\nOutput: owner and deadline list\",\"smoke_prompt\":\"Summarize these planning notes into clear actions.\",\"expected_output\":\"An action list with owners and deadlines.\",\"expectations\":[\"SKILL.md frontmatter validates successfully.\",\"The package describes when to use the skill and what it outputs.\"],\"tags\":[\"meeting\",\"actions\",\"planning\"]}</draft_json>\nEOF\n",
        )
        .expect("mock claude");
        let mut permissions = fs::metadata(&mock_bin).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&mock_bin, permissions).expect("chmod");

        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for turning sprint planning notes into action items."
                .to_string(),
            context: None,
        };

        let options = CreatorBridgeOptions {
            mode: CreatorMode::ClaudeCli,
            skill_create_bin: DEFAULT_SKILL_CREATE_BINARY.to_string(),
            skill_create_timeout_secs: DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
            claude_bin: mock_bin.to_string_lossy().to_string(),
            claude_model: Some("mock-model".to_string()),
            claude_timeout_secs: DEFAULT_CLAUDE_TIMEOUT_SECS,
        };

        let response = create_package_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &options,
        )
        .expect("claude package created");
        let skill_md = fs::read_to_string(
            filesystem::canonical_skills_root(&root)
                .join(&response.slug)
                .join("SKILL.md"),
        )
        .expect("skill md");

        assert_eq!(response.generator_used, CREATOR_CLAUDE);
        assert!(response.generation_summary.contains("Claude CLI"));
        assert!(skill_md.contains("name: meeting-mapper"));
        assert!(root
            .join(".skill-notebook")
            .join("generator-runs")
            .join(&response.slug)
            .join("create-from-nl.json")
            .exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn keeps_the_generated_title_when_slug_is_deduplicated() {
        let root = make_temp_project_root("claude-title");
        fs::create_dir_all(filesystem::canonical_skills_root(&root).join("meeting-mapper"))
            .expect("existing package");
        let mock_bin = root.join("mock-claude.sh");
        filesystem::write_text_file(
            &mock_bin,
            "#!/bin/sh\ncat <<'EOF'\n<draft_json>{\"name\":\"Meeting Mapper\",\"slug\":\"meeting-mapper\",\"description\":\"Maps meeting notes into actions. Use when the user needs action-oriented synthesis.\",\"skill_md\":\"---\\nname: meeting-mapper\\ndescription: \\\"Maps meeting notes into actions. Use when the user needs action-oriented synthesis.\\\"\\n---\\n\\n# Meeting Mapper\\n\\n## Overview\\nTransform notes into owners and deadlines.\\n\",\"system_prompt\":\"Stay concise and action-oriented.\",\"task_prompt\":\"1. Read the notes.\\n2. Extract owners and deadlines.\",\"example_markdown\":\"## Example\\n\\nInput: sprint planning notes\\n\\nOutput: owner and deadline list\",\"smoke_prompt\":\"Summarize these planning notes into clear actions.\",\"expected_output\":\"An action list with owners and deadlines.\",\"expectations\":[\"SKILL.md frontmatter validates successfully.\",\"The package describes when to use the skill and what it outputs.\"],\"tags\":[\"meeting\",\"actions\",\"planning\"]}</draft_json>\nEOF\n",
        )
        .expect("mock claude");
        let mut permissions = fs::metadata(&mock_bin).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&mock_bin, permissions).expect("chmod");

        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for turning sprint planning notes into action items."
                .to_string(),
            context: None,
        };
        let options = CreatorBridgeOptions {
            mode: CreatorMode::ClaudeCli,
            skill_create_bin: DEFAULT_SKILL_CREATE_BINARY.to_string(),
            skill_create_timeout_secs: DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
            claude_bin: mock_bin.to_string_lossy().to_string(),
            claude_model: Some("mock-model".to_string()),
            claude_timeout_secs: DEFAULT_CLAUDE_TIMEOUT_SECS,
        };

        let response = create_package_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &options,
        )
        .expect("claude package created");

        assert_eq!(response.slug, "meeting-mapper-2");
        assert_eq!(response.name, "Meeting Mapper");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn auto_mode_reports_claude_cli_failure_instead_of_using_template() {
        let root = make_temp_project_root("claude-timeout");
        let mock_bin = root.join("slow-claude.sh");
        filesystem::write_text_file(
            &mock_bin,
            "#!/bin/sh\nsleep 2\ncat <<'EOF'\n<draft_json>{\"name\":\"Slow Skill\",\"slug\":\"slow-skill\",\"description\":\"Stays slow. Use when the user wants a timeout.\",\"skill_md\":\"# Slow Skill\",\"system_prompt\":\"Stay slow.\",\"task_prompt\":\"Wait.\",\"example_markdown\":\"## Example\",\"smoke_prompt\":\"Wait.\",\"expected_output\":\"Timeout.\",\"expectations\":[\"Timeout is handled.\"],\"tags\":[\"slow\"]}</draft_json>\nEOF\n",
        )
        .expect("mock claude");
        let mut permissions = fs::metadata(&mock_bin).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&mock_bin, permissions).expect("chmod");

        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a reusable skill for handling long-running requests.".to_string(),
            context: None,
        };
        let options = CreatorBridgeOptions {
            mode: CreatorMode::Auto,
            skill_create_bin: DEFAULT_SKILL_CREATE_BINARY.to_string(),
            skill_create_timeout_secs: DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
            claude_bin: mock_bin.to_string_lossy().to_string(),
            claude_model: None,
            claude_timeout_secs: 1,
        };

        let error = create_package_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &options,
        )
        .expect_err("configured Claude failure should be surfaced");

        assert!(error.contains("Claude CLI draft generation failed"));
        assert!(error.contains("timed out"));
        assert!(filesystem::canonical_skills_root(&root)
            .read_dir()
            .expect("skills dir exists")
            .next()
            .is_none());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn uses_mocked_skill_create_cli_when_available() {
        let root = make_temp_project_root("skill-create");
        let mock_bin = root.join("mock-skill-create.sh");
        filesystem::write_text_file(
            &mock_bin,
            "#!/bin/sh\ncat <<'EOF'\n<draft_json>{\"name\":\"Doc Triage\",\"slug\":\"doc-triage\",\"description\":\"Triage a doc set into actions and questions. Use when the user needs a structured review workflow.\",\"skill_md\":\"---\\nname: doc-triage\\ndescription: \\\"Triage a doc set into actions and questions. Use when the user needs a structured review workflow.\\\"\\n---\\n\\n# Doc Triage\\n\\n## Overview\\nTurn docs into follow-ups.\\n\",\"system_prompt\":\"Stay crisp.\",\"task_prompt\":\"1. Read docs.\\n2. Output action list.\",\"example_markdown\":\"## Example\\n\\nInput: PRD\\n\\nOutput: action list\",\"smoke_prompt\":\"Triage this PRD.\",\"expected_output\":\"A list of issues and next steps.\",\"expectations\":[\"SKILL.md includes a When to Use section.\"],\"tags\":[\"docs\",\"triage\"]}</draft_json>\nEOF\n",
        )
        .expect("mock skill-create");
        let mut permissions = fs::metadata(&mock_bin).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&mock_bin, permissions).expect("chmod");

        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create a skill that reviews a PRD and produces a checklist.".to_string(),
            context: None,
        };

        let options = CreatorBridgeOptions {
            mode: CreatorMode::SkillCreate,
            skill_create_bin: mock_bin.to_string_lossy().to_string(),
            skill_create_timeout_secs: DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
            claude_bin: DEFAULT_CLAUDE_BINARY.to_string(),
            claude_model: None,
            claude_timeout_secs: DEFAULT_CLAUDE_TIMEOUT_SECS,
        };

        let response = create_package_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &options,
        )
        .expect("skill-create package created");

        assert_eq!(response.generator_used, CREATOR_SKILL_CREATE);
        assert!(response.generation_summary.contains("skill-create"));
        assert!(filesystem::canonical_skills_root(&root)
            .join(&response.slug)
            .join("SKILL.md")
            .exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn auto_mode_prefers_skill_create_when_available() {
        let root = make_temp_project_root("auto-skill-create");
        let mock_skill_create = root.join("mock-skill-create.sh");
        filesystem::write_text_file(
            &mock_skill_create,
            "#!/bin/sh\ncat <<'EOF'\n<draft_json>{\"name\":\"Auto Preferred\",\"slug\":\"auto-preferred\",\"description\":\"Prefers skill-create. Use when testing selection logic.\",\"skill_md\":\"# Auto Preferred\",\"system_prompt\":\"\",\"task_prompt\":\"\",\"example_markdown\":\"\",\"smoke_prompt\":\"\",\"expected_output\":\"\",\"expectations\":[],\"tags\":[\"auto\"]}</draft_json>\nEOF\n",
        )
        .expect("mock skill-create");
        let mut permissions = fs::metadata(&mock_skill_create)
            .expect("metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&mock_skill_create, permissions).expect("chmod");

        let mock_claude = root.join("mock-claude.sh");
        filesystem::write_text_file(
            &mock_claude,
            "#!/bin/sh\ncat <<'EOF'\n<draft_json>{\"name\":\"Claude Fallback\",\"slug\":\"claude-fallback\",\"description\":\"Should not be used. Use when testing selection logic.\",\"skill_md\":\"# Claude Fallback\",\"system_prompt\":\"\",\"task_prompt\":\"\",\"example_markdown\":\"\",\"smoke_prompt\":\"\",\"expected_output\":\"\",\"expectations\":[],\"tags\":[\"auto\"]}</draft_json>\nEOF\n",
        )
        .expect("mock claude");
        let mut permissions = fs::metadata(&mock_claude).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&mock_claude, permissions).expect("chmod");

        let request = CreatePackageFromNlRequest {
            project_root_id: "project_root-test".to_string(),
            prompt: "Create anything".to_string(),
            context: None,
        };

        let options = CreatorBridgeOptions {
            mode: CreatorMode::Auto,
            skill_create_bin: mock_skill_create.to_string_lossy().to_string(),
            skill_create_timeout_secs: DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
            claude_bin: mock_claude.to_string_lossy().to_string(),
            claude_model: None,
            claude_timeout_secs: DEFAULT_CLAUDE_TIMEOUT_SECS,
        };

        let response = create_package_in_workspace_with_options(
            &root,
            "project_root-test",
            &request,
            &options,
        )
        .expect("auto package created");

        assert_eq!(response.generator_used, CREATOR_SKILL_CREATE);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn bridge_status_reports_known_fields() {
        let status = creator_bridge_status();
        assert!(status.get("preferredGenerator").is_some());
        assert!(status.get("fallbackGenerator").is_some());
        assert!(status.get("claudeTimeoutSecs").is_some());
        assert!(status.get("claudeResolvedPath").is_some());
        assert!(status.get("skillCreateResolvedPath").is_some());
    }
}
