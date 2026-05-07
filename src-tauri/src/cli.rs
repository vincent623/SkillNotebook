use std::env;
use std::fmt::Write as _;
use std::process;

use serde_json::{json, Value};

use crate::domain::draft::{DraftDiscardRequest, DraftImportRequest, DraftStartRequest};
use crate::domain::package::{PackageImportRequest, PackageStatus};
use crate::services::{
    draft_service, eval_service, export_service, package_service, search_service, test_service,
    version_service,
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
    Find {
        query: Option<String>,
    },
    Eval {
        package_id: String,
    },
    Test {
        package_id: String,
    },
    Reference {
        package_id: String,
    },
    Import {
        source_path: String,
        slug: Option<String>,
        no_eval: bool,
    },
    Draft(DraftCommand),
    Export(ExportCommand),
    Version(VersionCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DraftCommand {
    Start {
        prompt: Option<String>,
        source_paths: Vec<String>,
        source_url: Option<String>,
        agent_command: Option<String>,
    },
    List,
    Import {
        draft_id: String,
        no_eval: bool,
    },
    Discard {
        draft_id: String,
    },
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

#[derive(Debug)]
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
        "find" => {
            let query = argv.get(index + 1).cloned();
            if argv.len() > index + 2 {
                return Err("find accepts at most one optional query".to_string());
            }
            SkillCommand::Find { query }
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
        "reference" | "use" => {
            let package_id = argv
                .get(index + 1)
                .cloned()
                .ok_or_else(|| "reference requires a package id".to_string())?;
            if argv.len() > index + 2 {
                return Err("reference accepts exactly one package id".to_string());
            }
            SkillCommand::Reference { package_id }
        }
        "import" => {
            let source_path = argv
                .get(index + 1)
                .cloned()
                .ok_or_else(|| "import requires a source directory".to_string())?;
            let mut slug = None;
            let mut no_eval = false;
            let mut cursor = index + 2;
            while cursor < argv.len() {
                match argv[cursor].as_str() {
                    "--slug" => {
                        cursor += 1;
                        slug = Some(
                            argv.get(cursor)
                                .cloned()
                                .ok_or_else(|| "--slug requires a value".to_string())?,
                        );
                        cursor += 1;
                    }
                    "--no-eval" => {
                        no_eval = true;
                        cursor += 1;
                    }
                    value => return Err(format!("unknown import argument: {}", value)),
                }
            }
            SkillCommand::Import {
                source_path,
                slug,
                no_eval,
            }
        }
        "draft" => {
            let action = argv.get(index + 1).map(String::as_str).ok_or_else(|| {
                "draft requires a subcommand: start | list | import | discard".to_string()
            })?;
            match action {
                "start" => {
                    let mut prompt = None;
                    let mut source_paths = Vec::new();
                    let mut source_url = None;
                    let mut agent_command = None;
                    let mut cursor = index + 2;
                    if let Some(value) = argv.get(cursor).filter(|value| !value.starts_with('-')) {
                        prompt = Some(value.clone());
                        cursor += 1;
                    }
                    while cursor < argv.len() {
                        match argv[cursor].as_str() {
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
                                source_url = Some(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--from-url requires a URL".to_string())?,
                                );
                                cursor += 1;
                            }
                            "--agent" => {
                                cursor += 1;
                                agent_command = Some(
                                    argv.get(cursor)
                                        .cloned()
                                        .ok_or_else(|| "--agent requires a command".to_string())?,
                                );
                                cursor += 1;
                            }
                            value => {
                                return Err(format!("unknown draft start argument: {}", value))
                            }
                        }
                    }
                    SkillCommand::Draft(DraftCommand::Start {
                        prompt,
                        source_paths,
                        source_url,
                        agent_command,
                    })
                }
                "list" => {
                    if argv.len() > index + 2 {
                        return Err("draft list accepts no arguments".to_string());
                    }
                    SkillCommand::Draft(DraftCommand::List)
                }
                "import" => {
                    let draft_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "draft import requires a draft id".to_string())?;
                    let mut no_eval = false;
                    let mut cursor = index + 3;
                    while cursor < argv.len() {
                        match argv[cursor].as_str() {
                            "--no-eval" => {
                                no_eval = true;
                                cursor += 1;
                            }
                            value => {
                                return Err(format!("unknown draft import argument: {}", value))
                            }
                        }
                    }
                    SkillCommand::Draft(DraftCommand::Import { draft_id, no_eval })
                }
                "discard" => {
                    let draft_id = argv
                        .get(index + 2)
                        .cloned()
                        .ok_or_else(|| "draft discard requires a draft id".to_string())?;
                    if argv.len() > index + 3 {
                        return Err("draft discard accepts exactly one draft id".to_string());
                    }
                    SkillCommand::Draft(DraftCommand::Discard { draft_id })
                }
                other => return Err(format!("unknown draft subcommand: {}", other)),
            }
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
  {program} [--project_root PATH] [--json] find [query]
  {program} [--project_root PATH] [--json] reference <package-id>
  {program} [--project_root PATH] [--json] import <source-dir> [--slug <slug>] [--no-eval]
  {program} [--project_root PATH] [--json] draft start [prompt] [--from-file <path>...] [--from-url <url>] [--agent <command>]
  {program} [--project_root PATH] [--json] draft list
  {program} [--project_root PATH] [--json] draft import <draft-id> [--no-eval]
  {program} [--project_root PATH] [--json] draft discard <draft-id>
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
        SkillCommand::Reference { package_id } => {
            let reference = package_service::reference_package(
                package_id,
                Some(project_root.root_path.as_str()),
            )?;
            let mut human = format!(
                "Reference for {} in {}\nPackage: {}\nSKILL.md: {}",
                package_id, project_root.root_path, reference.package_path, reference.skill_md_path
            );
            for item in &reference.items {
                let _ = write!(human, "\n\n{}:\n{}", item.label, item.value);
            }
            Ok(RenderedOutput {
                human,
                json: json!({
                    "command": "reference",
                    "project_root": project_root,
                    "packageId": package_id,
                    "reference": reference,
                }),
            })
        }
        SkillCommand::Import {
            source_path,
            slug,
            no_eval,
        } => {
            let req = PackageImportRequest {
                project_root_id: project_root.id.clone(),
                source_path: source_path.clone(),
                slug: slug.clone(),
                run_eval: Some(!no_eval),
            };
            let imported =
                package_service::import_package(&req, Some(project_root.root_path.as_str()))?;
            Ok(RenderedOutput {
                human: format!(
                    "Imported {} to {}\nEval: {}\nNext: {}",
                    imported.slug,
                    imported.package_path,
                    if imported.eval_report.is_some() {
                        "run"
                    } else {
                        "skipped"
                    },
                    imported.reference_command
                ),
                json: json!({
                    "command": "import",
                    "project_root": project_root,
                    "result": imported,
                }),
            })
        }
        SkillCommand::Draft(command) => match command {
            DraftCommand::Start {
                prompt,
                source_paths,
                source_url,
                agent_command,
            } => {
                let req = DraftStartRequest {
                    project_root_id: project_root.id.clone(),
                    prompt: prompt.clone(),
                    source_paths: Some(source_paths.clone()),
                    source_url: source_url.clone(),
                    preferred_agent_command: agent_command.clone(),
                };
                let draft =
                    draft_service::start_draft(&req, Some(project_root.root_path.as_str()))?;
                Ok(RenderedOutput {
                    human: format!(
                        "Started draft {}\nPath: {}\nRun: {}\nImport: {}",
                        draft.draft_id,
                        draft.draft_path,
                        draft.suggested_command,
                        draft.import_command
                    ),
                    json: json!({
                        "command": "draft.start",
                        "project_root": project_root,
                        "draft": draft,
                    }),
                })
            }
            DraftCommand::List => {
                let drafts = draft_service::list_drafts(Some(project_root.root_path.as_str()))?;
                let mut human = format!("Found {} draft workspace(s).", drafts.len());
                for draft in &drafts {
                    let _ = write!(human, "\n- {} {}", draft.draft_id, draft.draft_path);
                }
                Ok(RenderedOutput {
                    human,
                    json: json!({
                        "command": "draft.list",
                        "project_root": project_root,
                        "drafts": drafts,
                    }),
                })
            }
            DraftCommand::Import { draft_id, no_eval } => {
                let req = DraftImportRequest {
                    project_root_id: project_root.id.clone(),
                    draft_id: draft_id.clone(),
                    run_eval: Some(!no_eval),
                };
                let imported =
                    draft_service::import_draft(&req, Some(project_root.root_path.as_str()))?;
                Ok(RenderedOutput {
                    human: format!(
                        "Imported draft {} as {}\nPackage: {}\nNext: {}",
                        draft_id, imported.slug, imported.package_path, imported.reference_command
                    ),
                    json: json!({
                        "command": "draft.import",
                        "project_root": project_root,
                        "result": imported,
                    }),
                })
            }
            DraftCommand::Discard { draft_id } => {
                let req = DraftDiscardRequest {
                    project_root_id: project_root.id.clone(),
                    draft_id: draft_id.clone(),
                };
                let discarded =
                    draft_service::discard_draft(&req, Some(project_root.root_path.as_str()))?;
                Ok(RenderedOutput {
                    human: format!(
                        "{} draft {}.",
                        if discarded {
                            "Discarded"
                        } else {
                            "Did not find"
                        },
                        draft_id
                    ),
                    json: json!({
                        "command": "draft.discard",
                        "project_root": project_root,
                        "draftId": draft_id,
                        "discarded": discarded,
                    }),
                })
            }
        },
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
        execute, parse_from, DraftCommand, ExportCommand, ParseOutcome, SkillCli, SkillCommand,
        VersionCommand,
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
    fn reference_command_returns_copyable_items_as_json() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let project_root_path = root.to_string_lossy().to_string();
        let cli = match parse_from([
            "skill",
            "--json",
            "--project_root",
            project_root_path.as_str(),
            "reference",
            "pkg-interview",
        ])
        .expect("parse reference")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        let rendered = execute(&cli).expect("reference command should succeed");
        let items = rendered
            .json
            .get("reference")
            .and_then(|value| value.get("items"))
            .and_then(|value| value.as_array())
            .expect("reference items");

        assert!(items
            .iter()
            .any(|item| item.get("id").and_then(|value| value.as_str()) == Some("cli-reference")));

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn parser_recognizes_import_with_slug_and_no_eval() {
        let cli = match parse_from([
            "skill",
            "--json",
            "import",
            "/tmp/existing-skill",
            "--slug",
            "existing-skill",
            "--no-eval",
        ])
        .expect("parse import")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: true,
                command: SkillCommand::Import {
                    source_path: "/tmp/existing-skill".to_string(),
                    slug: Some("existing-skill".to_string()),
                    no_eval: true,
                },
            }
        );
    }

    #[test]
    fn parser_recognizes_draft_start_with_sources() {
        let cli = match parse_from([
            "skill",
            "draft",
            "start",
            "draft a source-backed skill",
            "--from-file",
            "/tmp/source.md",
            "--from-url",
            "https://example.com/brief",
            "--agent",
            "codex --model gpt-5.4",
        ])
        .expect("parse draft start")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: false,
                command: SkillCommand::Draft(DraftCommand::Start {
                    prompt: Some("draft a source-backed skill".to_string()),
                    source_paths: vec!["/tmp/source.md".to_string()],
                    source_url: Some("https://example.com/brief".to_string()),
                    agent_command: Some("codex --model gpt-5.4".to_string()),
                }),
            }
        );
    }

    #[test]
    fn parser_recognizes_draft_import_without_eval() {
        let cli = match parse_from(["skill", "draft", "import", "draft-demo", "--no-eval"])
            .expect("parse draft import")
        {
            ParseOutcome::Run(cli) => cli,
            ParseOutcome::Print(_) => panic!("expected runnable cli"),
        };

        assert_eq!(
            cli,
            SkillCli {
                project_root: None,
                json: false,
                command: SkillCommand::Draft(DraftCommand::Import {
                    draft_id: "draft-demo".to_string(),
                    no_eval: true,
                }),
            }
        );
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
    fn parser_rejects_removed_create_command() {
        let error = parse_from(["skill", "create", "draft a meeting skill"])
            .expect_err("legacy create should be removed");

        assert_eq!(error, "unknown command: create");
    }

    #[test]
    fn parser_rejects_removed_generator_doctor() {
        let error = parse_from(["skill", "--json", "doctor", "generator"])
            .expect_err("legacy generator doctor should be removed");

        assert_eq!(error, "unknown command: doctor");
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
