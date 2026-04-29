use std::env;
use std::fmt::Write as _;
use std::process;

use serde_json::{json, Value};

use crate::domain::package::{CreatePackageFromNlRequest, PackageStatus};
use crate::services::{
    eval_service, package_service, search_service, skill_create_service, test_service,
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
    Create {
        prompt: String,
        context: Option<String>,
    },
    Eval {
        package_id: String,
    },
    Test {
        package_id: String,
    },
    Version(VersionCommand),
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
        "find" => {
            let query = argv.get(index + 1).cloned();
            if argv.len() > index + 2 {
                return Err("find accepts at most one optional query".to_string());
            }
            SkillCommand::Find { query }
        }
        "create" => {
            let prompt = argv
                .get(index + 1)
                .cloned()
                .ok_or_else(|| "create requires a prompt".to_string())?;
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

            SkillCommand::Create { prompt, context }
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
  {program} [--project_root PATH] [--json] create <prompt> [--context <text>]
  {program} [--project_root PATH] [--json] eval <package-id>
  {program} [--project_root PATH] [--json] test <package-id>
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
        SkillCommand::Create { prompt, context } => {
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
    use super::{execute, parse_from, ParseOutcome, SkillCli, SkillCommand, VersionCommand};
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
                command: SkillCommand::Create {
                    prompt: "draft a meeting skill".to_string(),
                    context: Some("output markdown".to_string()),
                },
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
