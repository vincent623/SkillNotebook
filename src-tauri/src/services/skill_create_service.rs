use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::json;

use crate::domain::package::{
    CreatePackageFromNlRequest, CreatePackageFromNlResponse, PackageNotebookDocument, PackageStatus,
};
use crate::services::eval_service;
use crate::storage::filesystem;
use crate::utils::ids::slugify;
use crate::utils::time::{now_iso, today_slug};

const DEFAULT_CLAUDE_BINARY: &str = "claude";
const DEFAULT_CLAUDE_TIMEOUT_SECS: u64 = 60;
const DEFAULT_SKILL_CREATE_BINARY: &str = "skill-create";
const DEFAULT_SKILL_CREATE_TIMEOUT_SECS: u64 = 60;
const CLAUDE_POLL_INTERVAL_MS: u64 = 100;
const CREATOR_FALLBACK: &str = "template_fallback";
const CREATOR_SKILL_CREATE: &str = "skill_create_cli";
const CREATOR_CLAUDE: &str = "claude_cli";

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

pub fn create_package_from_nl(
    req: &CreatePackageFromNlRequest,
    root_path: Option<&str>,
) -> Result<CreatePackageFromNlResponse, String> {
    let workspace_root = filesystem::workspace_root_for_id(&req.workspace_id, root_path)?;
    let options = resolve_creator_bridge_options();
    create_package_in_workspace_with_options(&workspace_root, &req.workspace_id, req, &options)
}

pub fn creator_bridge_status() -> serde_json::Value {
    let options = resolve_creator_bridge_options();
    let claude_available = command_exists(&options.claude_bin);
    let skill_create_available = command_exists(&options.skill_create_bin);
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
    workspace_root: &Path,
    _workspace_id: &str,
    req: &CreatePackageFromNlRequest,
    options: &CreatorBridgeOptions,
) -> Result<CreatePackageFromNlResponse, String> {
    let packages_root = workspace_root.join("packages");
    filesystem::ensure_directory(&packages_root)?;

    let draft_result = build_initial_draft(req, options)?;
    let slug = unique_package_slug(&draft_result.draft.slug, &packages_root);
    let package_root = packages_root.join(&slug);
    let package_id = format!("pkg-{}", slug);
    let created_at = now_iso();

    let mut draft = draft_result.draft;
    draft.slug = slug.clone();
    if draft.name.trim().is_empty() {
        draft.name = title_case_slug(&slug).replace("  ", " ");
    } else {
        draft.name = sanitize_title(&draft.name);
    }
    draft.skill_md = normalize_skill_markdown(
        &draft.skill_md,
        &slug,
        &draft.description,
        &draft.name,
        &summarize_goal(
            &normalize_text(&req.prompt),
            &normalize_text(req.context.as_deref().unwrap_or_default()),
            &draft.tags,
        ),
        &draft.tags,
        &draft.expectations,
        &draft.system_prompt,
    );

    write_draft_files(&package_root, &draft, &slug)?;
    write_generator_log(
        workspace_root,
        &slug,
        &draft_result.generator_used,
        &draft_result.generation_summary,
        draft_result.prompt_log.as_deref(),
        draft_result.response_log.as_deref(),
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
        workspace_root,
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
        eval_workspace_path: workspace_root
            .join(".42eval")
            .join(&slug)
            .to_string_lossy()
            .to_string(),
        draft_created: true,
        auto_eval_started: true,
        validation_summary: evaluation.validation_summary,
        generator_used: draft_result.generator_used,
        generation_summary: draft_result.generation_summary,
    })
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
                                Err(claude_error) => {
                                    return Ok(fallback_draft_result(
                                        req,
                                        &format!(
                                            "skill-create draft generation failed and Claude CLI draft generation failed, so the local template draft was used instead: {}",
                                            summarize_error(&format!(
                                                "{}; {}",
                                                summarize_error(&error),
                                                summarize_error(&claude_error)
                                            ))
                                        ),
                                    ));
                                }
                            }
                        }

                        return Ok(fallback_draft_result(
                            req,
                            &format!(
                                "skill-create draft generation failed and the local template draft was used instead: {}",
                                summarize_error(&error)
                            ),
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
                Err(error) => Ok(fallback_draft_result(
                    req,
                    &format!(
                        "Claude CLI draft generation failed and the local template draft was used instead: {}",
                        summarize_error(&error)
                    ),
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
    let mut command = Command::new(&options.claude_bin);
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
                return Err(format!("Claude CLI exited {}: {}", status, stderr.trim()));
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
                "Claude CLI timed out after {}s.{}",
                options.claude_timeout_secs, details
            ));
        }

        thread::sleep(Duration::from_millis(CLAUDE_POLL_INTERVAL_MS));
    }
}

fn call_skill_create_text(prompt: &str, options: &CreatorBridgeOptions) -> Result<String, String> {
    let mut command = Command::new(&options.skill_create_bin);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR"))),
        );

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
                return Err(format!("skill-create exited {}: {}", status, stderr.trim()));
            }

            return Ok(stdout.trim().to_string());
        }

        if started_at.elapsed() >= timeout {
            child.kill().ok();
            child.wait().ok();
            let stderr =
                read_child_stream(child.stderr.take(), "skill-create", "stderr").unwrap_or_default();
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

fn write_draft_files(package_root: &Path, draft: &DraftPackage, slug: &str) -> Result<(), String> {
    filesystem::write_text_file(&package_root.join("SKILL.md"), &draft.skill_md)?;
    filesystem::write_text_file(
        &package_root.join("prompts").join("system.md"),
        &draft.system_prompt,
    )?;
    filesystem::write_text_file(
        &package_root.join("prompts").join("task.md"),
        &draft.task_prompt,
    )?;
    filesystem::write_text_file(
        &package_root.join("examples").join("example-01.md"),
        &draft.example_markdown,
    )?;
    filesystem::write_text_file(
        &package_root.join("tests").join("smoke-test.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "name": "smoke-test",
                "package": slug,
                "prompt": draft.smoke_prompt,
                "expectedOutput": draft.expected_output,
                "checks": draft.expectations,
            }))
            .map_err(|error| format!("failed to serialize smoke test: {}", error))?
        ),
    )?;
    filesystem::write_text_file(
        &package_root.join("evals").join("evals.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
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
            .map_err(|error| format!("failed to serialize eval definitions: {}", error))?
        ),
    )?;

    Ok(())
}

fn write_generator_log(
    workspace_root: &Path,
    slug: &str,
    generator_used: &str,
    generation_summary: &str,
    prompt_log: Option<&str>,
    response_log: Option<&str>,
) -> Result<(), String> {
    if prompt_log.is_none() && response_log.is_none() {
        return Ok(());
    }

    let path = workspace_root
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
    if command.contains('/') {
        return Path::new(command).exists();
    }

    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .any(|path| path.join(command).exists())
}

fn summarize_error(error: &str) -> String {
    let one_line = error.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut summary = one_line.chars().take(180).collect::<String>();
    if one_line.chars().count() > 180 {
        summary.push_str("...");
    }
    summary
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
    use crate::utils::time::now_iso;

    use super::{
        create_package_in_workspace_with_options, creator_bridge_status, unique_package_slug,
        CreatePackageFromNlRequest, CreatorBridgeOptions, CreatorMode, CREATOR_CLAUDE,
        CREATOR_FALLBACK, CREATOR_SKILL_CREATE, DEFAULT_CLAUDE_BINARY, DEFAULT_CLAUDE_TIMEOUT_SECS,
        DEFAULT_SKILL_CREATE_BINARY, DEFAULT_SKILL_CREATE_TIMEOUT_SECS,
    };

    fn make_temp_workspace(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "skill-notebook-create-{}-{}",
            name,
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).ok();
        }

        fs::create_dir_all(root.join(".skill-notebook")).expect("workspace config dir");
        fs::create_dir_all(root.join("packages")).expect("packages dir");
        filesystem::write_text_file(
            &root.join(".skill-notebook").join("config.json"),
            &format!(
                "{{\"id\":\"workspace-test\",\"name\":\"Test Workspace\",\"createdAt\":\"{}\",\"updatedAt\":\"{}\"}}",
                now_iso(),
                now_iso()
            ),
        )
        .expect("workspace config");
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

    #[test]
    fn allocates_unique_slugs_when_the_base_exists() {
        let root = make_temp_workspace("slug");
        fs::create_dir_all(root.join("packages").join("meeting-actions"))
            .expect("existing package");

        let slug = unique_package_slug("meeting-actions", &root.join("packages"));

        assert_eq!(slug, "meeting-actions-2");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn creates_a_real_package_from_template_mode() {
        let root = make_temp_workspace("create");
        let request = CreatePackageFromNlRequest {
            workspace_id: "workspace-test".to_string(),
            prompt: "Turn customer interview notes into recurring action items and themes."
                .to_string(),
            context: Some("The package should help with synthesis and follow-up.".to_string()),
        };

        let response = create_package_in_workspace_with_options(
            &root,
            "workspace-test",
            &request,
            &template_options(),
        )
        .expect("package created");

        assert!(root
            .join("packages")
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
    fn uses_mocked_claude_cli_when_available() {
        let root = make_temp_workspace("claude");
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
            workspace_id: "workspace-test".to_string(),
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

        let response =
            create_package_in_workspace_with_options(&root, "workspace-test", &request, &options)
                .expect("claude package created");
        let skill_md =
            fs::read_to_string(root.join("packages").join(&response.slug).join("SKILL.md"))
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
        let root = make_temp_workspace("claude-title");
        fs::create_dir_all(root.join("packages").join("meeting-mapper")).expect("existing package");
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
            workspace_id: "workspace-test".to_string(),
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

        let response =
            create_package_in_workspace_with_options(&root, "workspace-test", &request, &options)
                .expect("claude package created");

        assert_eq!(response.slug, "meeting-mapper-2");
        assert_eq!(response.name, "Meeting Mapper");

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn auto_mode_falls_back_when_claude_cli_times_out() {
        let root = make_temp_workspace("claude-timeout");
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
            workspace_id: "workspace-test".to_string(),
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

        let response =
            create_package_in_workspace_with_options(&root, "workspace-test", &request, &options)
                .expect("fallback package created");

        assert_eq!(response.generator_used, CREATOR_FALLBACK);
        assert!(response.generation_summary.contains("timed out"));
        assert!(root
            .join("packages")
            .join(&response.slug)
            .join("SKILL.md")
            .exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn uses_mocked_skill_create_cli_when_available() {
        let root = make_temp_workspace("skill-create");
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
            workspace_id: "workspace-test".to_string(),
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

        let response =
            create_package_in_workspace_with_options(&root, "workspace-test", &request, &options)
                .expect("skill-create package created");

        assert_eq!(response.generator_used, CREATOR_SKILL_CREATE);
        assert!(response.generation_summary.contains("skill-create"));
        assert!(root
            .join("packages")
            .join(&response.slug)
            .join("SKILL.md")
            .exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn auto_mode_prefers_skill_create_when_available() {
        let root = make_temp_workspace("auto-skill-create");
        let mock_skill_create = root.join("mock-skill-create.sh");
        filesystem::write_text_file(
            &mock_skill_create,
            "#!/bin/sh\ncat <<'EOF'\n<draft_json>{\"name\":\"Auto Preferred\",\"slug\":\"auto-preferred\",\"description\":\"Prefers skill-create. Use when testing selection logic.\",\"skill_md\":\"# Auto Preferred\",\"system_prompt\":\"\",\"task_prompt\":\"\",\"example_markdown\":\"\",\"smoke_prompt\":\"\",\"expected_output\":\"\",\"expectations\":[],\"tags\":[\"auto\"]}</draft_json>\nEOF\n",
        )
        .expect("mock skill-create");
        let mut permissions = fs::metadata(&mock_skill_create).expect("metadata").permissions();
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
            workspace_id: "workspace-test".to_string(),
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

        let response =
            create_package_in_workspace_with_options(&root, "workspace-test", &request, &options)
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
    }
}
