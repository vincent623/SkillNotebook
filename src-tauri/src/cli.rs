use std::env;
use std::fmt::Write as _;
use std::process;

use serde_json::{json, Value};

use crate::domain::package::{
    CommitPackagePreviewRequest, CreatePackageFromNlRequest, CreatePackageFromSourcesRequest,
    CreatePackageFromUrlRequest, DiscardPackagePreviewRequest, PackageStatus,
};
use crate::services::{
    eval_service, export_service, package_service, search_service, skill_create_service,
    test_service, version_service,
};
use crate::storage::filesystem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillCli {
    project_root: Option<String>,
    json: bool,
    command: SkillCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SkillCommand {
    Doctor(DoctorCommand),
    Find { query: Option<String> },
    Create(CreateCommand),
    Eval { package_id: String },
    Test { package_id: String },
    Export(ExportCommand),
    Version(VersionCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DoctorCommand {
    Generator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CreateCommand {
    Direct {
        prompt: String,
        context: Option<String>,
    },
    Preview {
        source: CreatePreviewSource,
        prompt: Option<String>,
        context: Option<String>,
    },
    Commit {
        preview_id: String,
    },
    Discard {
        preview_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CreatePreviewSource {
    Text,
    Files { source_paths: Vec<String> },
    Url { url: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExportCommand {
    Zip { package_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionCommand {
    List {
        package_id: String,
    },
    Save {
        package_id: String,
        note: Option<String>,
    },
    Diff {
        version_id: String,
    },
    Restore {
        version_id: String,
    },
}

enum ParseOutcome {
    Run(SkillCli),
    Print(String),
}

pub fn main() {
    match parse_env() {
        Ok(ParseOutcome::Run(cli)) => {
            let as_json = cli.json;

            match execute(&cli) {
                Ok(rendered) => {
                    if as_json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&rendered.json)
                                .expect("cli output should always serialize")
                        );
                    } else {
                        println!("{}", rendered.human);
                    }
                }
                Err(error) => {
                    eprintln!("skill: {}", error);
                    process::exit(1);
                }
            }
        }
        Ok(ParseOutcome::Print(message)) => {
            println!("{}", message);
        }
        Err(error) => {
            eprintln!("skill: {}", error);
            eprintln!("Run `skill --help` for usage.");
            process::exit(2);
        }
    }
}

#[derive(Debug, Clone)]
struct RenderedOutput {
    human: String,
    json: Value,
}

fn parse_env() -> Result<ParseOutcome, String> {
    parse_from(env::args())
}

fn parse_from<I, S>(args: I) -> Result<ParseOutcome, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let raw = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let argv = if raw.is_empty() {
        vec!["skill".to_string()]
    } else {
        raw
    };

    let program = argv.first().cloned().unwrap_or_else(|| "skill".to_string());
    let mut index = 1usize;
    let mut project_root = None;
    let mut json = false;

    while index < argv.len() {
        match argv[index].as_str() {
            "-h" | "--help" => return Ok(ParseOutcome::Print(help_text(&program))),
            "-V" | "--version" => {
                return Ok(ParseOutcome::Print(format!(
                    "skill {}",
                    env!("CARGO_PKG_VERSION")
                )))
            }
            "--json" => {
                json = true;
                index += 1;
            }
            "--project_root" => {
                index += 1;
                project_root = Some(
                    argv.get(index)
                        .cloned()
                        .ok_or_else(|| "--project_root requires a path".to_string())?,
                );
                index += 1;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown option: {}", value));
            }
            _ => break,
        }
    }

    if index >= argv.len() {
        return Ok(ParseOutcome::Print(help_text(&program)));
    }

    let command = match argv[index].as_str() {
        "doctor" => {
            let target = argv
                .get(index + 1)
                .map(String::as_str)
                .ok_or_else(|| "doctor requires a target: generator".to_string())?;
            if argv.len() > index + 2 {
                return Err("doctor accepts exactly one target".to_string());
            }
            match target {
                "generator" => SkillCommand::Doctor(DoctorCommand::Generator),
                other => return Err(format!("unknown doctor target: {}", other)),
            }
        }
        "find" => {
            let query = argv.get(index + 1).cloned();
            if argv.len() > index + 2 {
                return Err("find accepts at most one optional query".to_string());
            }
            SkillCommand::Find { query }
        }
        "create" => {
            let action = argv
                .get(index + 1)
                .cloned()
                .ok_or_else(|| "create requires a prompt".to_string())?;

            match action.as_str() {
                "preview" => {
                    let mut prompt = None;
                    let mut context = None;
                    let mut source_paths = Vec::new();
                    let mut url = None;
                    let mut cursor = index + 2;

                    while cursor < argv.len() {
                        match argv[cursor].as_str() {
                            "--prompt" => {
                                cursor += 1;
                                prompt = Some(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--prompt requires a value".to_string())?,
                                );
                                cursor += 1;
                            }
                            "--context" => {
                                cursor += 1;
                                context = Some(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--context requires a value".to_string())?,
                                );
                                cursor += 1;
                            }
                            "--from-file" | "--from-path" => {
                                cursor += 1;
                                source_paths.push(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--from-file requires a path".to_string())?,
                                );
                                cursor += 1;
                            }
                            "--from-url" => {
                                cursor += 1;
                                url = Some(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--from-url requires a URL".to_string())?,
                                );
                                cursor += 1;
                            }
                            value => {
                                return Err(format!("unknown create preview argument: {}", value))
                            }
                        }
                    }

                    let source = match (source_paths.is_empty(), url) {
                        (false, Some(_)) => {
                            return Err(
                                "create preview accepts either --from-file/--from-path or --from-url, not both"
                                    .to_string(),
                            )
                        }
                        (false, None) => CreatePreviewSource::Files { source_paths },
                        (true, Some(url)) => CreatePreviewSource::Url { url },
                        (true, None) => {
                            if prompt
                                .as_deref()
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .is_none()
                            {
                                return Err(
                                    "create preview requires --prompt when no source path or URL is provided"
                                        .to_string(),
                                );
                            }
                            CreatePreviewSource::Text
                        }
                    };

                    SkillCommand::Create(CreateCommand::Preview {
                        source,
                        prompt,
                        context,
                    })
                }
                "commit" => {
                    let preview_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "create commit requires a preview id".to_string())?;
                    if argv.len() > index + 3 {
                        return Err("create commit accepts exactly one preview id".to_string());
                    }
                    SkillCommand::Create(CreateCommand::Commit { preview_id })
                }
                "discard" => {
                    let preview_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "create discard requires a preview id".to_string())?;
                    if argv.len() > index + 3 {
                        return Err("create discard accepts exactly one preview id".to_string());
                    }
                    SkillCommand::Create(CreateCommand::Discard { preview_id })
                }
                _ => {
                    let prompt = action;
                    let mut context = None;
                    let mut cursor = index + 2;

                    while cursor < argv.len() {
                        match argv[cursor].as_str() {
                            "--context" => {
                                cursor += 1;
                                context = Some(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--context requires a value".to_string())?,
                                );
                                cursor += 1;
                            }
                            value => return Err(format!("unknown create argument: {}", value)),
                        }
                    }

                    SkillCommand::Create(CreateCommand::Direct { prompt, context })
                }
            }
        }
        "eval" => {
            let package_id = argv
                .get(index + 1)
                .cloned()
                .ok_or_else(|| "eval requires a package id".to_string())?;
            if argv.len() > index + 2 {
                return Err("eval accepts exactly one package id".to_string());
            }
            SkillCommand::Eval { package_id }
        }
        "test" => {
            let package_id = argv
                .get(index + 1)
                .cloned()
                .ok_or_else(|| "test requires a package id".to_string())?;
            if argv.len() > index + 2 {
                return Err("test accepts exactly one package id".to_string());
            }
            SkillCommand::Test { package_id }
        }
        "export" => {
            let action = argv
                .get(index + 1)
                .map(String::as_str)
                .ok_or_else(|| "export requires a subcommand: zip".to_string())?;
            match action {
                "zip" => {
                    let package_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "export zip requires a package id".to_string())?;
                    if argv.len() > index + 3 {
                        return Err("export zip accepts exactly one package id".to_string());
                    }
                    SkillCommand::Export(ExportCommand::Zip { package_id })
                }
                other => return Err(format!("unknown export subcommand: {}", other)),
            }
        }
        "version" => {
            let action = argv.get(index + 1).map(String::as_str).ok_or_else(|| {
                "version requires a subcommand: list | save | diff | restore".to_string()
            })?;

            match action {
                "list" => {
                    let package_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "version list requires a package id".to_string())?;
                    if argv.len() > index + 3 {
                        return Err("version list accepts exactly one package id".to_string());
                    }
                    SkillCommand::Version(VersionCommand::List { package_id })
                }
                "save" => {
                    let package_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "version save requires a package id".to_string())?;
                    let mut note = None;
                    let mut cursor = index + 3;

                    while cursor < argv.len() {
                        match argv[cursor].as_str() {
                            "--note" => {
                                cursor += 1;
                                note = Some(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--note requires a value".to_string())?,
                                );
                                cursor += 1;
                            }
                            value => {
                                return Err(format!("unknown version save argument: {}", value))
                            }
                        }
                    }

                    SkillCommand::Version(VersionCommand::Save { package_id, note })
                }
                "diff" => {
                    let version_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "version diff requires a version id".to_string())?;
                    if argv.len() > index + 3 {
                        return Err("version diff accepts exactly one version id".to_string());
                    }
                    SkillCommand::Version(VersionCommand::Diff { version_id })
                }
                "restore" => {
                    let version_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "version restore requires a version id".to_string())?;
                    if argv.len() > index + 3 {
                        return Err("version restore accepts exactly one version id".to_string());
                    }
                    SkillCommand::Version(VersionCommand::Restore { version_id })
                }
                other => return Err(format!("unknown version subcommand: {}", other)),
            }
        }
        other => return Err(format!("unknown command: {}", other)),
    };

    Ok(ParseOutcome::Run(SkillCli {
        project_root,
        json,
        command,
    }))
}

fn help_text(program: &str) -> String {
    format!(
        "\
Skill Notebook core CLI

Usage:
  {program} [--project_root PATH] [--json] doctor generator
  {program} [--project_root PATH] [--json] find [query]
  {program} [--project_root PATH] [--json] create <prompt> [--context <text>]
  {program} [--project_root PATH] [--json] create preview --prompt <text> [--context <text>]
  {program} [--project_root PATH] [--json] create preview --from-file <path> [--from-file <path>...] [--prompt <text>] [--context <text>]
  {program} [--project_root PATH] [--json] create preview --from-url <url> [--prompt <text>] [--context <text>]
  {program} [--project_root PATH] [--json] create commit <preview-id>
  {program} [--project_root PATH] [--json] create discard <preview-id>
  {program} [--project_root PATH] [--json] eval <package-id>
  {program} [--project_root PATH] [--json] test <package-id>
  {program} [--project_root PATH] [--json] export zip <package-id>
  {program} [--project_root PATH] [--json] version list <package-id>
  {program} [--project_root PATH] [--json] version save <package-id> [--note <text>]
  {program} [--project_root PATH] [--json] version diff <version-id>
  {program} [--project_root PATH] [--json] version restore <version-id>

Global options:
  --project_root PATH   Run against a specific project_root root
  --json             Print structured JSON instead of human-readable text
  --help             Show this help message
  --version          Show the CLI version
"
    )
}

fn execute(cli: &SkillCli) -> Result<RenderedOutput, String> {
    let project_root_path = cli.project_root.as_deref();
    let scanned = filesystem::scan_project_root(project_root_path)?;
    let project_root = scanned.project_root;

    match &cli.command {
        SkillCommand::Doctor(DoctorCommand::Generator) => {
            let status = skill_create_service::creator_bridge_status();
            let preferred = status
                .get("preferredGenerator")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let claude_available = status
                .get("claudeCliAvailable")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let skill_create_available = status
                .get("skillCreateCommandAvailable")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let claude_timeout = status
                .get("claudeTimeoutSecs")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let claude_retry_attempts = status
                .get("claudeRetryAttempts")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let claude_retry_backoff = status
                .get("claudeRetryBackoffSecs")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let mut human = format!(
                "Generator doctor for {}\nPreferred: {}\nClaude CLI: {}\nskill-create: {}\nClaude timeout: {}s\nClaude retry: {} attempt(s), {}s base backoff",
                project_root.root_path,
                preferred,
                if claude_available { "available" } else { "unavailable" },
                if skill_create_available { "available" } else { "unavailable" },
                claude_timeout,
                claude_retry_attempts,
                claude_retry_backoff
            );
            if let Some(path) = status.get("claudeResolvedPath").and_then(Value::as_str) {
                let _ = write!(human, "\nClaude path: {}", path);
            }
            if let Some(path) = status
                .get("skillCreateResolvedPath")
                .and_then(Value::as_str)
            {
                let _ = write!(human, "\nskill-create path: {}", path);
            }

            Ok(RenderedOutput {
                human,
                json: json!({
                    "command": "doctor.generator",
                    "project_root": project_root,
                    "generator": status,
                }),
            })
        }
        SkillCommand::Find { query } => {
            if let Some(query) = query
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let results =
                    search_service::search_packages(query, Some(project_root.root_path.as_str()))?;
                let mut human = format!(
                    "ProjectRoot: {}\nFound {} package(s) matching \"{}\".",
                    project_root.root_path,
                    results.len(),
                    query
                );

                for item in &results {
                    let _ = write!(
                        human,
                        "\n- {} [{}] {}",
                        item.package_id,
                        status_label(&item.status),
                        item.name
                    );
                    if !item.description.trim().is_empty() {
                        let _ = write!(human, "\n  {}", item.description);
                    }
                }

                Ok(RenderedOutput {
                    human,
                    json: json!({
                        "command": "find",
                        "project_root": project_root,
                        "query": query,
                        "results": results,
                    }),
                })
            } else {
                let packages =
                    package_service::list_packages(Some(project_root.root_path.as_str()))?;
                let mut human = format!(
                    "ProjectRoot: {}\nListed {} package(s).",
                    project_root.root_path,
                    packages.len()
                );

                for item in &packages {
                    let _ = write!(
                        human,
                        "\n- {} [{}] {}",
                        item.id,
                        status_label(&item.status),
                        item.name
                    );
                    if !item.description.trim().is_empty() {
                        let _ = write!(human, "\n  {}", item.description);
                    }
                }

                Ok(RenderedOutput {
                    human,
                    json: json!({
                        "command": "find",
                        "project_root": project_root,
                        "query": Value::Null,
                        "packages": packages,
                    }),
                })
            }
        }
        SkillCommand::Create(command) => match command {
            CreateCommand::Direct { prompt, context } => {
                let req = CreatePackageFromNlRequest {
                    project_root_id: project_root.id.clone(),
                    prompt: prompt.clone(),
                    context: context.clone(),
                };
                let created = skill_create_service::create_package_from_nl(
                    &req,
                    Some(project_root.root_path.as_str()),
                )?;

                Ok(RenderedOutput {
                    human: format!(
                        "Created {} at {}\nGenerator: {}\nValidation: {}",
                        created.name,
                        created.root_path,
                        created.generator_used,
                        created.validation_summary
                    ),
                    json: json!({
                        "command": "create",
                        "project_root": project_root,
                        "result": created,
                    }),
                })
            }
            CreateCommand::Preview {
                source,
                prompt,
                context,
            } => {
                let preview = match source {
                    CreatePreviewSource::Text => {
                        let req = CreatePackageFromNlRequest {
                            project_root_id: project_root.id.clone(),
                            prompt: prompt.clone().unwrap_or_default(),
                            context: context.clone(),
                        };
                        skill_create_service::generate_package_preview_from_nl(
                            &req,
                            Some(project_root.root_path.as_str()),
                        )?
                    }
                    CreatePreviewSource::Files { source_paths } => {
                        let req = CreatePackageFromSourcesRequest {
                            project_root_id: project_root.id.clone(),
                            source_paths: source_paths.clone(),
                            prompt: prompt.clone(),
                            context: context.clone(),
                        };
                        skill_create_service::generate_package_preview_from_sources(
                            &req,
                            Some(project_root.root_path.as_str()),
                        )?
                    }
                    CreatePreviewSource::Url { url } => {
                        let req = CreatePackageFromUrlRequest {
                            project_root_id: project_root.id.clone(),
                            url: url.clone(),
                            prompt: prompt.clone(),
                            context: context.clone(),
                        };
                        skill_create_service::generate_package_preview_from_url(
                            &req,
                            Some(project_root.root_path.as_str()),
                        )?
                    }
                };
                let preview_id = preview.preview_id.clone();
                let name = preview.name.clone();
                let slug = preview.slug.clone();
                let generator_used = preview.generator_used.clone();
                let generation_summary = preview.generation_summary.clone();
                let file_count = preview.files.len();
                let commit_command = format!(
                    "skill --project_root '{}' create commit {}",
                    project_root.root_path, preview_id
                );
                Ok(RenderedOutput {
                    human: format!(
                        "Previewed {} ({})\nPreview id: {}\nGenerator: {}\nFiles: {}\nCommit with: skill --project_root '{}' create commit {}",
                        name,
                        slug,
                        preview_id,
                        generator_used,
                        file_count,
                        project_root.root_path,
                        preview_id
                    ),
                    json: json!({
                        "command": "create.preview",
                        "project_root": project_root,
                        "previewId": preview_id,
                        "name": name,
                        "slug": slug,
                        "generatorUsed": generator_used,
                        "generationSummary": generation_summary,
                        "fileCount": file_count,
                        "commitCommand": commit_command,
                        "preview": preview,
                    }),
                })
            }
            CreateCommand::Commit { preview_id } => {
                let req = CommitPackagePreviewRequest {
                    project_root_id: project_root.id.clone(),
                    preview_id: preview_id.clone(),
                };
                let created = skill_create_service::commit_package_preview(
                    &req,
                    Some(project_root.root_path.as_str()),
                )?;
                let package_id = created.package_id.clone();
                let name = created.name.clone();
                let slug = created.slug.clone();
                let package_path = created.root_path.clone();
                let generator_used = created.generator_used.clone();
                Ok(RenderedOutput {
                    human: format!(
                        "Committed preview {} as {}\nPackage path: {}\nGenerator: {}",
                        preview_id, name, package_path, generator_used
                    ),
                    json: json!({
                        "command": "create.commit",
                        "project_root": project_root,
                        "previewId": preview_id,
                        "packageId": package_id,
                        "name": name,
                        "slug": slug,
                        "packagePath": package_path,
                        "generatorUsed": generator_used,
                        "result": created,
                        "package": created,
                    }),
                })
            }
            CreateCommand::Discard { preview_id } => {
                let req = DiscardPackagePreviewRequest {
                    project_root_id: project_root.id.clone(),
                    preview_id: preview_id.clone(),
                };
                let discarded = skill_create_service::discard_package_preview(
                    &req,
                    Some(project_root.root_path.as_str()),
                )?;
                Ok(RenderedOutput {
                    human: format!(
                        "{} preview {} in {}.",
                        if discarded {
                            "Discarded"
                        } else {
                            "Did not find"
                        },
                        preview_id,
                        project_root.root_path
                    ),
                    json: json!({
                        "command": "create.discard",
                        "project_root": project_root,
                        "previewId": preview_id,
                        "discarded": discarded,
                    }),
                })
            }
        },
        SkillCommand::Eval { package_id } => {
            let report = eval_service::run_eval(package_id, Some(project_root.root_path.as_str()))?;
            let human = format!(
                "Evaluated {} in {}\nOverall: {}\nScores: completeness {:.2}, clarity {:.2}, executability {:.2}",
                package_id,
                project_root.root_path,
                overall_status_label(&report.overall_status),
                report.completeness_score,
                report.clarity_score,
                report.executability_score
            );

            Ok(RenderedOutput {
                human,
                json: json!({
                    "command": "eval",
                    "project_root": project_root,
                    "report": report,
                }),
            })
        }
        SkillCommand::Test { package_id } => {
            let report =
                test_service::run_package_test(package_id, Some(project_root.root_path.as_str()))?;
            let human = format!(
                "Tested {} in {}\nStatus: {}\nSummary: {}",
                package_id,
                project_root.root_path,
                test_status_label(&report.status),
                report.summary
            );

            Ok(RenderedOutput {
                human,
                json: json!({
                    "command": "test",
                    "project_root": project_root,
                    "report": report,
                }),
            })
        }
        SkillCommand::Export(ExportCommand::Zip { package_id }) => {
            let artifact = export_service::export_package_zip(
                package_id,
                Some(project_root.root_path.as_str()),
            )?;
            let zip_path = artifact.zip_path.clone();
            let size_bytes = artifact.size_bytes;
            Ok(RenderedOutput {
                human: format!(
                    "Exported {} to {}\nSize: {} bytes",
                    package_id, artifact.zip_path, artifact.size_bytes
                ),
                json: json!({
                    "command": "export.zip",
                    "project_root": project_root,
                    "packageId": package_id,
                    "zipPath": zip_path,
                    "sizeBytes": size_bytes,
                    "artifact": artifact,
                }),
            })
        }
        SkillCommand::Version(command) => match command {
            VersionCommand::List { package_id } => {
                let versions = version_service::list_versions(
                    package_id,
                    Some(project_root.root_path.as_str()),
                )?;
                let mut human = format!(
                    "ProjectRoot: {}\nFound {} formal version(s) for {}.",
                    project_root.root_path,
                    versions.len(),
                    package_id
                );

                for item in &versions {
                    let _ = write!(
                        human,
                        "\n- v{} {}{}",
                        item.version_number,
                        item.created_at,
                        if item.is_pinned { " [pinned]" } else { "" }
                    );
                    if let Some(note) = item
                        .note
                        .as_deref()
                        .filter(|value| !value.trim().is_empty())
                    {
                        let _ = write!(human, "\n  {}", note);
                    }
                }

                Ok(RenderedOutput {
                    human,
                    json: json!({
                        "command": "version.list",
                        "project_root": project_root,
                        "packageId": package_id,
                        "versions": versions,
                    }),
                })
            }
            VersionCommand::Save { package_id, note } => {
                let saved = version_service::save_version(
                    package_id,
                    note.clone(),
                    Some(project_root.root_path.as_str()),
                )?;
                Ok(RenderedOutput {
                    human: format!(
                        "Saved {} as v{} at {}\nSnapshot: {}",
                        package_id, saved.version_number, saved.created_at, saved.snapshot_path
                    ),
                    json: json!({
                        "command": "version.save",
                        "project_root": project_root,
                        "packageId": package_id,
                        "version": saved,
                    }),
                })
            }
            VersionCommand::Diff { version_id } => {
                let diff = version_service::diff_version(
                    version_id,
                    Some(project_root.root_path.as_str()),
                )?;
                let mut human = format!(
                    "Compared {} against the current draft in {}\nFound {} changed file(s).",
                    version_id,
                    project_root.root_path,
                    diff.entries.len()
                );

                for entry in &diff.entries {
                    let _ = write!(
                        human,
                        "\n\n# {} [{}]\n{}",
                        entry.path,
                        diff_change_label(&entry.change_type),
                        entry.diff_text
                    );
                }

                Ok(RenderedOutput {
                    human,
                    json: json!({
                        "command": "version.diff",
                        "project_root": project_root,
                        "versionId": version_id,
                        "diff": diff,
                    }),
                })
            }
            VersionCommand::Restore { version_id } => {
                let restored = version_service::restore_version(
                    version_id,
                    Some(project_root.root_path.as_str()),
                )?;
                Ok(RenderedOutput {
                    human: format!(
                        "Restored {} to v{} in {}\nPackage path: {}",
                        restored.id,
                        restored.current_version,
                        project_root.root_path,
                        restored.root_path
                    ),
                    json: json!({
                        "command": "version.restore",
                        "project_root": project_root,
                        "versionId": version_id,
                        "package": restored,
                    }),
                })
            }
        },
    }
}

fn status_label(status: &PackageStatus) -> &'static str {
    match status {
        PackageStatus::Draft => "draft",
        PackageStatus::Evaluating => "evaluating",
        PackageStatus::Validated => "validated",
        PackageStatus::NeedsEval => "needs_eval",
        PackageStatus::Archived => "archived",
    }
}

fn overall_status_label(status: &crate::domain::eval::EvalOverallStatus) -> &'static str {
    match status {
        crate::domain::eval::EvalOverallStatus::Usable => "usable",
        crate::domain::eval::EvalOverallStatus::NeedsImprovement => "needs_improvement",
        crate::domain::eval::EvalOverallStatus::Problematic => "problematic",
    }
}

fn test_status_label(status: &crate::domain::test::PackageTestStatus) -> &'static str {
    match status {
        crate::domain::test::PackageTestStatus::Passed => "passed",
        crate::domain::test::PackageTestStatus::Failed => "failed",
        crate::domain::test::PackageTestStatus::Missing => "missing",
    }
}

fn diff_change_label(change: &crate::domain::version::VersionDiffChangeType) -> &'static str {
    match change {
        crate::domain::version::VersionDiffChangeType::Added => "added",
        crate::domain::version::VersionDiffChangeType::Removed => "removed",
        crate::domain::version::VersionDiffChangeType::Modified => "modified",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        execute, parse_from, CreateCommand, CreatePreviewSource, DoctorCommand, ExportCommand,
        ParseOutcome, SkillCli, SkillCommand, VersionCommand,
    };
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::storage::filesystem;

    fn tmp_project_root_path() -> PathBuf {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "skill-notebook-cli-test-{}-{}",
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
    fn find_command_lists_packages_as_json() {
        let cli = match parse_from(["skill", "--json", "find"]).expect("parse find") {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        let rendered = execute(&cli).expect("find command should succeed");
        let packages = rendered
            .json
            .get("packages")
            .and_then(|value| value.as_array())
            .expect("packages array");

        assert!(!packages.is_empty());
    }

    #[test]
    fn version_save_command_returns_saved_version_json() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let project_root_path = root.to_string_lossy().to_string();
        let cli = match parse_from([
            "skill",
            "--json",
            "--project_root",
            project_root_path.as_str(),
            "version",
            "save",
            "pkg-interview",
            "--note",
            "cli smoke",
        ])
        .expect("parse version save")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        let rendered = execute(&cli).expect("version save should succeed");
        let version_number = rendered
            .json
            .get("version")
            .and_then(|value| value.get("versionNumber"))
            .and_then(|value| value.as_u64())
            .expect("version number");

        assert!(version_number >= 4);
    }

    #[test]
    fn parser_recognizes_create_context() {
        let cli = match parse_from([
            "skill",
            "--project_root",
            "/tmp/demo",
            "create",
            "draft a meeting skill",
            "--context",
            "output markdown",
        ])
        .expect("parse create")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: Some("/tmp/demo".to_string()),
                json: false,
                command: SkillCommand::Create(CreateCommand::Direct {
                    prompt: "draft a meeting skill".to_string(),
                    context: Some("output markdown".to_string()),
                }),
            }
        );
    }

    #[test]
    fn parser_recognizes_doctor_generator() {
        let cli = match parse_from(["skill", "--json", "doctor", "generator"])
            .expect("parse doctor generator")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: true,
                command: SkillCommand::Doctor(DoctorCommand::Generator),
            }
        );
    }

    #[test]
    fn parser_recognizes_create_preview_from_file() {
        let cli = match parse_from([
            "skill",
            "create",
            "preview",
            "--from-file",
            "\"/tmp/source note.txt\"",
            "--prompt",
            "draft from notes",
            "--context",
            "keep citations",
        ])
        .expect("parse create preview")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: false,
                command: SkillCommand::Create(CreateCommand::Preview {
                    source: CreatePreviewSource::Files {
                        source_paths: vec!["\"/tmp/source note.txt\"".to_string()],
                    },
                    prompt: Some("draft from notes".to_string()),
                    context: Some("keep citations".to_string()),
                }),
            }
        );
    }

    #[test]
    fn parser_recognizes_create_commit() {
        let cli = match parse_from(["skill", "create", "commit", "preview-demo"])
            .expect("parse create commit")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: false,
                command: SkillCommand::Create(CreateCommand::Commit {
                    preview_id: "preview-demo".to_string(),
                }),
            }
        );
    }

    #[test]
    fn parser_recognizes_export_zip() {
        let cli = match parse_from(["skill", "--json", "export", "zip", "pkg-pdf"])
            .expect("parse export zip")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: true,
                command: SkillCommand::Export(ExportCommand::Zip {
                    package_id: "pkg-pdf".to_string(),
                }),
            }
        );
    }

    #[test]
    fn parser_recognizes_version_diff() {
        let cli = match parse_from(["skill", "version", "diff", "version-pkg-interview-v3"])
            .expect("parse version diff")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: false,
                command: SkillCommand::Version(VersionCommand::Diff {
                    version_id: "version-pkg-interview-v3".to_string(),
                }),
            }
        );
    }

    #[test]
    fn parser_recognizes_test_command() {
        let cli = match parse_from(["skill", "--json", "test", "pkg-interview"])
            .expect("parse test command")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: true,
                command: SkillCommand::Test {
                    package_id: "pkg-interview".to_string(),
                },
            }
        );
    }
}
