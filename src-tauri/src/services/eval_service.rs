use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::domain::eval::{EvalDetails, EvalOverallStatus, EvalReport};
use crate::domain::package::PackageStatus;
use crate::storage::filesystem;
use crate::utils::time::now_iso;

#[derive(Debug, Clone)]
pub struct EvaluationArtifacts {
    pub report: EvalReport,
    pub suggested_status: PackageStatus,
    pub validation_summary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SkillEvalFile {
    skill_name: String,
    evals: Vec<SkillEvalCase>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SkillEvalCase {
    id: u32,
    prompt: String,
    expected_output: String,
    files: Vec<String>,
    expectations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvalExpectationResult {
    expectation_id: String,
    description: String,
    passed: bool,
    evidence: String,
}

#[derive(Debug, Clone)]
struct ValidationOutcome {
    passed: bool,
    summary: String,
}

pub fn latest_report(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<Option<EvalReport>, String> {
    let report = filesystem::scan_workspace(root_path)?
        .eval_reports
        .into_iter()
        .find(|item| item.package_id == package_id);

    Ok(report)
}

pub fn run_eval(package_id: &str, root_path: Option<&str>) -> Result<EvalReport, String> {
    let scanned = filesystem::scan_workspace(root_path)?;
    let package = scanned
        .packages
        .iter()
        .find(|item| item.id == package_id)
        .cloned()
        .ok_or_else(|| format!("package not found: {}", package_id))?;

    let workspace_root = PathBuf::from(&scanned.workspace.root_path);
    let package_root = PathBuf::from(&package.root_path);
    let mut notebook = filesystem::load_package_notebook(&package_root)?;
    let iteration = notebook.eval_reports.len() as u32 + 1;

    let evaluation = evaluate_package(
        &workspace_root,
        &package_root,
        &package.id,
        &package.slug,
        &package.name,
        &notebook.description,
        iteration,
    )?;

    notebook.last_eval_status = Some(evaluation.report.overall_status.clone());
    notebook.status = if matches!(notebook.status, PackageStatus::Archived) {
        PackageStatus::Archived
    } else {
        evaluation.suggested_status.clone()
    };
    notebook.updated_at = now_iso();
    notebook.eval_reports.insert(0, evaluation.report.clone());

    filesystem::save_package_notebook(&package_root, &notebook)?;

    Ok(evaluation.report)
}

pub fn evaluate_package(
    workspace_root: &Path,
    package_root: &Path,
    package_id: &str,
    slug: &str,
    package_name: &str,
    fallback_description: &str,
    iteration: u32,
) -> Result<EvaluationArtifacts, String> {
    let started = Instant::now();
    let now = now_iso();
    let skill_path = package_root.join("SKILL.md");
    let skill_md = read_optional_text(&skill_path)?;
    let skill_content = skill_md.as_deref().unwrap_or_default();
    let frontmatter = extract_frontmatter(skill_content);
    let frontmatter_description = frontmatter
        .as_deref()
        .and_then(|block| extract_frontmatter_value(block, "description"));
    let effective_description = frontmatter_description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback_description);
    let lower_skill = skill_content.to_lowercase();

    let prompt_files = list_relative_files(package_root, "prompts")?;
    let example_files = list_relative_files(package_root, "examples")?;
    let reference_files = list_relative_files(package_root, "references")?;
    let script_files = list_relative_files(package_root, "scripts")?;
    let test_files = list_relative_files(package_root, "tests")?;
    let eval_definition = load_eval_file(package_root)?;
    let validation = run_quick_validate(package_root);

    let has_skill_md = skill_path.exists();
    let has_examples = !example_files.is_empty();
    let has_prompts = !prompt_files.is_empty();
    let has_scripts = !script_files.is_empty();
    let has_tests = !test_files.is_empty() || !eval_definition.evals.is_empty();
    let input_defined = contains_any(
        &lower_skill,
        &[
            "## inputs",
            "input",
            "inputs",
            "incoming",
            "source material",
        ],
    );
    let output_defined = contains_any(
        &lower_skill,
        &["## outputs", "output", "outputs", "deliverable", "result"],
    ) || eval_definition
        .evals
        .iter()
        .any(|item| !item.expected_output.trim().is_empty());
    let boundaries_clear = contains_any(
        &lower_skill,
        &[
            "don't use",
            "do not use",
            "not for",
            "boundar",
            "when not to use",
        ],
    );
    let description_has_trigger = effective_description.to_lowercase().contains("use when");
    let has_structured_sections =
        lower_skill.contains("## workflow") || lower_skill.contains("## quick reference");
    let has_placeholders = contains_any(
        skill_content,
        &["[trigger condition]", "[task 1]", "[task 2]"],
    );

    let completeness_score = score_ratio(&[
        (has_skill_md, 24),
        (has_prompts, 16),
        (has_examples, 14),
        (has_tests, 14),
        (input_defined, 12),
        (output_defined, 12),
        (boundaries_clear, 8),
    ]);

    let clarity_score = score_ratio(&[
        (description_has_trigger, 25),
        (input_defined, 18),
        (output_defined, 18),
        (boundaries_clear, 15),
        (has_structured_sections, 14),
        (!has_placeholders, 10),
    ]);

    let executability_score = score_ratio(&[
        (validation.passed, 35),
        (has_prompts, 20),
        (has_tests, 20),
        (has_examples, 10),
        (!has_placeholders, 10),
        (has_skill_md, 5),
    ]);

    let overall_average = (completeness_score + clarity_score + executability_score) / 3.0;
    let overall_status = if validation.passed && overall_average >= 0.84 {
        EvalOverallStatus::Usable
    } else if overall_average >= 0.62 {
        EvalOverallStatus::NeedsImprovement
    } else {
        EvalOverallStatus::Problematic
    };

    let suggested_status = if matches!(overall_status, EvalOverallStatus::Usable)
        && has_examples
        && has_prompts
        && has_tests
        && output_defined
    {
        PackageStatus::Validated
    } else if matches!(overall_status, EvalOverallStatus::Problematic) {
        PackageStatus::Draft
    } else {
        PackageStatus::NeedsEval
    };

    let mut suggestions = Vec::new();
    if !validation.passed {
        suggestions.push(validation.summary.clone());
    }
    if !description_has_trigger {
        suggestions.push("Add a frontmatter description that clearly says what the skill does and when to use it.".to_string());
    }
    if !boundaries_clear {
        suggestions.push(
            "Add a short 'When not to use' or boundary section so the skill does not over-trigger."
                .to_string(),
        );
    }
    if !has_examples {
        suggestions
            .push("Add at least one example to demonstrate the expected output shape.".to_string());
    }
    if !has_tests {
        suggestions.push(
            "Add eval or smoke-test files so the package can be re-checked consistently."
                .to_string(),
        );
    }
    if !output_defined {
        suggestions.push(
            "Define the final output contract more explicitly in SKILL.md or eval expectations."
                .to_string(),
        );
    }
    suggestions.truncate(4);

    let mut notes = vec![validation.summary.clone()];
    if !reference_files.is_empty() {
        notes.push(format!(
            "{} reference file(s) are bundled for progressive disclosure.",
            reference_files.len()
        ));
    }
    if has_scripts {
        notes.push(format!(
            "{} script file(s) are available for deterministic execution steps.",
            script_files.len()
        ));
    }
    if has_placeholders {
        notes.push(
            "Template placeholders are still present and should be replaced before publication."
                .to_string(),
        );
    }

    let details = EvalDetails {
        has_skill_md,
        has_examples,
        has_prompts,
        has_scripts,
        input_defined,
        output_defined,
        boundaries_clear,
        notes,
    };

    let report = EvalReport {
        id: format!("eval-{}-v{}", slug, iteration),
        package_id: package_id.to_string(),
        completeness_score,
        clarity_score,
        executability_score,
        overall_status,
        suggestions,
        details,
        created_at: now.clone(),
    };

    let case_results = build_case_results(
        &eval_definition,
        has_skill_md,
        has_prompts,
        has_examples,
        input_defined,
        output_defined,
        boundaries_clear,
        validation.passed,
        description_has_trigger,
    );

    sync_eval_workspace(
        workspace_root,
        package_root,
        package_id,
        slug,
        package_name,
        &now,
        iteration,
        &eval_definition,
        &case_results,
        &report,
        &validation,
        started.elapsed().as_millis() as u64,
    )?;

    Ok(EvaluationArtifacts {
        report,
        suggested_status,
        validation_summary: validation.summary,
    })
}

fn sync_eval_workspace(
    workspace_root: &Path,
    package_root: &Path,
    package_id: &str,
    slug: &str,
    package_name: &str,
    created_at: &str,
    iteration: u32,
    _eval_definition: &SkillEvalFile,
    case_results: &[(SkillEvalCase, Vec<EvalExpectationResult>)],
    report: &EvalReport,
    validation: &ValidationOutcome,
    duration_ms: u64,
) -> Result<(), String> {
    let eval_root = workspace_root.join(".42eval").join(slug);
    let cases_root = eval_root.join("cases");
    let iteration_root = eval_root.join("iterations").join(format!("v{}", iteration));
    let snapshot_root = eval_root.join("skill-snapshot");

    filesystem::ensure_directory(&cases_root)?;
    filesystem::ensure_directory(&iteration_root)?;
    filesystem::ensure_directory(&snapshot_root)?;

    if package_root.join("SKILL.md").exists() {
        let skill_snapshot = snapshot_root.join("SKILL.md");
        let content = fs::read_to_string(package_root.join("SKILL.md"))
            .map_err(|error| format!("failed to read {}: {}", package_root.display(), error))?;
        filesystem::write_text_file(&skill_snapshot, &content)?;
    }
    if package_root.join("references").exists() {
        filesystem::copy_directory_recursive(
            &package_root.join("references"),
            &snapshot_root.join("references"),
        )?;
    }

    let mut benchmark_runs = Vec::new();
    for (case, results) in case_results {
        let case_id = format!("case-{:03}", case.id.max(1));
        let eval_id = format!("eval-{}", case.id.max(1));
        let eval_dir = iteration_root.join(&eval_id);
        let with_skill_dir = eval_dir.join("with_skill");
        filesystem::ensure_directory(&with_skill_dir)?;

        filesystem::write_json_file(
            &cases_root.join(format!("{}.json", case_id)),
            &json!({
                "caseId": case_id,
                "caseName": format!("{} case {}", package_name, case.id.max(1)),
                "prompt": case.prompt,
                "expectations": results.iter().enumerate().map(|(index, item)| {
                    json!({
                        "id": format!("exp-{}", index + 1),
                        "description": item.description,
                        "category": expectation_category(&item.description),
                        "critical": index < 2,
                    })
                }).collect::<Vec<_>>(),
                "files": case.files,
            }),
        )?;

        filesystem::write_json_file(
            &eval_dir.join("eval_metadata.json"),
            &json!({
                "evalId": eval_id,
                "caseId": case_id,
                "caseName": format!("{} case {}", package_name, case.id.max(1)),
                "prompt": case.prompt,
                "expectations": results.iter().enumerate().map(|(index, item)| {
                    json!({
                        "id": format!("exp-{}", index + 1),
                        "description": item.description,
                        "category": expectation_category(&item.description),
                        "critical": index < 2,
                    })
                }).collect::<Vec<_>>(),
                "config": {
                    "skillName": slug,
                    "quadrant": "leverage",
                    "strategy": "structural-review",
                    "iteration": iteration,
                },
                "createdAt": created_at,
            }),
        )?;

        let passed_count = results.iter().filter(|item| item.passed).count();
        let pass_rate = if results.is_empty() {
            0.0
        } else {
            passed_count as f32 / results.len() as f32
        };

        filesystem::write_json_file(
            &with_skill_dir.join("grading.json"),
            &json!({
                "expectations": results,
                "passRate": pass_rate,
                "passedCount": passed_count,
                "totalCount": results.len(),
                "reasoning": format!(
                    "Structural review run for {}. Validation: {}.",
                    slug, validation.summary
                ),
                "gradedAt": created_at,
            }),
        )?;
        filesystem::write_json_file(
            &with_skill_dir.join("metadata.json"),
            &json!({
                "startedAt": created_at,
                "completedAt": created_at,
                "durationMs": duration_ms,
                "totalTokens": 0,
                "toolCalls": 1,
                "model": "local-structural-review",
            }),
        )?;
        filesystem::write_text_file(
            &with_skill_dir.join("output.md"),
            &format!(
                "# Structural Eval\n\n- Package: {}\n- Report: {}\n- Validation: {}\n\n## Suggestions\n{}\n",
                package_name,
                report.id,
                validation.summary,
                if report.suggestions.is_empty() {
                    "- No immediate fixes suggested.".to_string()
                } else {
                    report
                        .suggestions
                        .iter()
                        .map(|item| format!("- {}", item))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            ),
        )?;

        benchmark_runs.push(json!({
            "evalId": eval_id,
            "caseId": case_id,
            "caseName": format!("{} case {}", package_name, case.id.max(1)),
            "withSkill": {
                "expectations": results,
                "passRate": pass_rate,
                "outputLength": fs::read_to_string(with_skill_dir.join("output.md")).map(|content| content.len()).unwrap_or_default(),
                "metadata": {
                    "durationMs": duration_ms,
                    "totalTokens": 0
                }
            },
            "withoutSkill": null
        }));
    }

    filesystem::write_json_file(
        &eval_root.join("config.json"),
        &json!({
            "version": "2.0",
            "skillName": slug,
            "skillPath": package_root.to_string_lossy(),
            "classification": {
                "quadrant": "leverage",
                "strategy": "structural-review",
                "modelCapability": "strong",
                "humanPractice": "emerging",
                "modelScore": (report.clarity_score * 100.0).round() as i32,
                "practiceScore": (report.completeness_score * 100.0).round() as i32,
                "confidence": report.executability_score,
                "signals": [
                    {
                        "dimension": "model",
                        "direction": "strong",
                        "keyword": slug,
                        "weight": 6
                    }
                ]
            },
            "currentIteration": iteration,
            "createdAt": created_at,
            "updatedAt": created_at
        }),
    )?;

    filesystem::write_json_file(
        &iteration_root.join("benchmark.json"),
        &json!({
            "version": "2.0",
            "skillName": slug,
            "packageId": package_id,
            "iteration": iteration,
            "runs": benchmark_runs,
            "runSummary": {
                "withSkill": {
                    "meanPassRate": average_pass_rate(case_results),
                    "totalRuns": case_results.len()
                },
                "withoutSkill": null
            },
            "timing": {
                "withSkill": {
                    "meanDurationMs": duration_ms,
                    "meanTokens": 0,
                    "totalRuns": case_results.len()
                }
            },
            "evaluatedAt": created_at
        }),
    )?;

    filesystem::write_text_file(
        &iteration_root.join("benchmark.md"),
        &format!(
            "# Benchmark Summary\n\n- Package: {}\n- Iteration: v{}\n- Mean pass rate: {:.0}%\n- Validation: {}\n",
            package_name,
            iteration,
            average_pass_rate(case_results) * 100.0,
            validation.summary
        ),
    )?;

    Ok(())
}

fn load_eval_file(package_root: &Path) -> Result<SkillEvalFile, String> {
    let path = package_root.join("evals").join("evals.json");
    if !path.exists() {
        return Ok(SkillEvalFile::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {}", path.display(), error))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("failed to parse {}: {}", path.display(), error))
}

fn build_case_results(
    eval_definition: &SkillEvalFile,
    has_skill_md: bool,
    has_prompts: bool,
    has_examples: bool,
    input_defined: bool,
    output_defined: bool,
    boundaries_clear: bool,
    validation_passed: bool,
    description_has_trigger: bool,
) -> Vec<(SkillEvalCase, Vec<EvalExpectationResult>)> {
    let cases = if eval_definition.evals.is_empty() {
        vec![SkillEvalCase {
            id: 1,
            prompt: "Run the package structural review.".to_string(),
            expected_output: "A draft skill package with prompts, examples, and tests.".to_string(),
            files: Vec::new(),
            expectations: vec![
                "SKILL.md frontmatter validates successfully.".to_string(),
                "The package describes when to use the skill and what it outputs.".to_string(),
                "Prompt, example, and eval files are present.".to_string(),
            ],
        }]
    } else {
        eval_definition.evals.clone()
    };

    cases
        .into_iter()
        .map(|case| {
            let expectations = if case.expectations.is_empty() {
                vec![
                    "SKILL.md frontmatter validates successfully.".to_string(),
                    "The package describes when to use the skill and what it outputs.".to_string(),
                    "Prompt, example, and eval files are present.".to_string(),
                ]
            } else {
                case.expectations.clone()
            };

            let results = expectations
                .into_iter()
                .enumerate()
                .map(|(index, description)| {
                    let lowered = description.to_lowercase();
                    let (passed, evidence) = if lowered.contains("frontmatter")
                        || lowered.contains("validate")
                    {
                        (
                            validation_passed,
                            if validation_passed {
                                "quick_validate.py accepted the skill frontmatter.".to_string()
                            } else {
                                "quick_validate.py reported a frontmatter issue.".to_string()
                            },
                        )
                    } else if lowered.contains("when to use")
                        || lowered.contains("trigger")
                        || lowered.contains("description")
                    {
                        (
                            description_has_trigger,
                            if description_has_trigger {
                                "The description contains an explicit 'Use when' trigger."
                                    .to_string()
                            } else {
                                "The description still needs an explicit trigger clause."
                                    .to_string()
                            },
                        )
                    } else if lowered.contains("output") {
                        (
                            output_defined,
                            if output_defined {
                                "Output expectations are stated in SKILL.md or eval definitions."
                                    .to_string()
                            } else {
                                "No explicit output contract was detected.".to_string()
                            },
                        )
                    } else if lowered.contains("input") {
                        (
                            input_defined,
                            if input_defined {
                                "Input expectations are spelled out in the skill.".to_string()
                            } else {
                                "Input requirements are still implicit.".to_string()
                            },
                        )
                    } else if lowered.contains("example") {
                        (
                            has_examples,
                            if has_examples {
                                "Example files are present under examples/.".to_string()
                            } else {
                                "No example files were found.".to_string()
                            },
                        )
                    } else if lowered.contains("prompt") {
                        (
                            has_prompts,
                            if has_prompts {
                                "Prompt files are present under prompts/.".to_string()
                            } else {
                                "No prompt files were found.".to_string()
                            },
                        )
                    } else if lowered.contains("boundary") || lowered.contains("not to use") {
                        (
                            boundaries_clear,
                            if boundaries_clear {
                                "Boundary language is present in the skill instructions."
                                    .to_string()
                            } else {
                                "The skill still needs clearer boundary language.".to_string()
                            },
                        )
                    } else {
                        (
                            has_skill_md && has_prompts && output_defined,
                            "General structural expectations are satisfied by the current draft."
                                .to_string(),
                        )
                    };

                    EvalExpectationResult {
                        expectation_id: format!("exp-{}", index + 1),
                        description,
                        passed,
                        evidence,
                    }
                })
                .collect::<Vec<_>>();

            (case, results)
        })
        .collect()
}

fn average_pass_rate(case_results: &[(SkillEvalCase, Vec<EvalExpectationResult>)]) -> f32 {
    if case_results.is_empty() {
        return 0.0;
    }

    let sum = case_results
        .iter()
        .map(|(_, results)| {
            if results.is_empty() {
                0.0
            } else {
                results.iter().filter(|item| item.passed).count() as f32 / results.len() as f32
            }
        })
        .sum::<f32>();

    sum / case_results.len() as f32
}

fn run_quick_validate(package_root: &Path) -> ValidationOutcome {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")));
    let script_path = repo_root
        .join("docs")
        .join("skill-creator")
        .join("scripts")
        .join("quick_validate.py");

    if !script_path.exists() {
        return ValidationOutcome {
            passed: false,
            summary: "Validation helper not found; structural review continued without it."
                .to_string(),
        };
    }

    match Command::new("python3")
        .arg(&script_path)
        .arg(package_root)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let summary = if stdout.is_empty() { stderr } else { stdout };

            ValidationOutcome {
                passed: output.status.success(),
                summary: if summary.is_empty() {
                    "Validation completed.".to_string()
                } else {
                    summary
                },
            }
        }
        Err(error) => ValidationOutcome {
            passed: false,
            summary: format!("Validation helper could not run: {}", error),
        },
    }
}

fn list_relative_files(package_root: &Path, subdir: &str) -> Result<Vec<String>, String> {
    let dir = package_root.join(subdir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_relative_files(package_root, &dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_relative_files(
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
            collect_relative_files(package_root, &path, files)?;
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

fn read_optional_text(path: &Path) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }

    fs::read_to_string(path)
        .map(Some)
        .map_err(|error| format!("failed to read {}: {}", path.display(), error))
}

fn extract_frontmatter(content: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }

    let mut frontmatter = Vec::new();
    for line in lines {
        if line == "---" {
            return Some(frontmatter.join("\n"));
        }
        frontmatter.push(line.to_string());
    }

    None
}

fn extract_frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
    let lines = frontmatter.lines().collect::<Vec<_>>();
    let prefix = format!("{}:", key);
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let value = rest.trim();
            if value == "|" || value == ">" {
                index += 1;
                let mut block = Vec::new();
                while index < lines.len() {
                    let current = lines[index];
                    if current.starts_with(' ') || current.starts_with('\t') {
                        block.push(current.trim().to_string());
                        index += 1;
                    } else {
                        break;
                    }
                }
                return Some(block.join(" "));
            }

            return Some(value.trim_matches('"').trim_matches('\'').to_string());
        }

        index += 1;
    }

    None
}

fn score_ratio(items: &[(bool, u32)]) -> f32 {
    let total_weight = items.iter().map(|(_, weight)| *weight).sum::<u32>() as f32;
    if total_weight == 0.0 {
        return 0.0;
    }

    let earned = items
        .iter()
        .filter(|(passed, _)| *passed)
        .map(|(_, weight)| *weight)
        .sum::<u32>() as f32;

    (earned / total_weight * 100.0).round() / 100.0
}

fn contains_any(content: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| content.contains(pattern))
}

fn expectation_category(description: &str) -> &'static str {
    let lowered = description.to_lowercase();
    if lowered.contains("output") || lowered.contains("input") {
        "structure"
    } else if lowered.contains("example") || lowered.contains("prompt") {
        "content"
    } else if lowered.contains("trigger") || lowered.contains("frontmatter") {
        "quality"
    } else {
        "differential"
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::{env, fs};

    use crate::domain::package::PackageNotebookDocument;
    use crate::storage::filesystem;
    use crate::utils::time::now_iso;

    use super::{evaluate_package, PackageStatus, SkillEvalFile};

    fn make_temp_workspace(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("skill-notebook-{}-{}", name, std::process::id()));
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

    #[test]
    fn evaluates_a_valid_package_as_usable() {
        let workspace_root = make_temp_workspace("eval-usable");
        let package_root = workspace_root.join("packages").join("draft-skill");
        fs::create_dir_all(package_root.join("prompts")).expect("prompts");
        fs::create_dir_all(package_root.join("examples")).expect("examples");
        fs::create_dir_all(package_root.join("evals")).expect("evals");

        filesystem::write_text_file(
            &package_root.join("SKILL.md"),
            "---\nname: draft-skill\ndescription: Structures a reusable workflow for turning notes into actions. Use when the user asks to summarize meetings or extract action items.\n---\n\n# Draft Skill\n\n## Inputs\n- meeting notes\n\n## Outputs\n- action list\n\n## Workflow\n1. Read the notes.\n2. Extract owners and deadlines.\n\n## When not to use\n- raw transcription tasks\n",
        )
        .expect("skill");
        filesystem::write_text_file(
            &package_root.join("prompts").join("task.md"),
            "Summarize the notes into actions.",
        )
        .expect("task");
        filesystem::write_text_file(
            &package_root.join("examples").join("example-01.md"),
            "Input: weekly sync notes\nOutput: owner, deadline, risk",
        )
        .expect("example");
        filesystem::write_text_file(
            &package_root.join("evals").join("evals.json"),
            &serde_json::to_string_pretty(&SkillEvalFile {
                skill_name: "draft-skill".to_string(),
                evals: vec![super::SkillEvalCase {
                    id: 1,
                    prompt: "Turn weekly notes into clear actions.".to_string(),
                    expected_output: "A structured action list.".to_string(),
                    files: Vec::new(),
                    expectations: vec![
                        "SKILL.md frontmatter validates successfully.".to_string(),
                        "The package describes when to use the skill and what it outputs."
                            .to_string(),
                    ],
                }],
            })
            .expect("eval json"),
        )
        .expect("eval file");

        let notebook = PackageNotebookDocument {
            id: "pkg-draft-skill".to_string(),
            name: "Draft Skill".to_string(),
            description: "Structures a reusable workflow for turning notes into actions."
                .to_string(),
            tags: vec!["meeting".to_string()],
            status: PackageStatus::Draft,
            current_version: 0,
            last_eval_status: None,
            related_skills: Vec::new(),
            bundle_candidates: Vec::new(),
            created_at: now_iso(),
            updated_at: now_iso(),
            versions: Vec::new(),
            eval_reports: Vec::new(),
        };
        filesystem::save_package_notebook(&package_root, &notebook).expect("notebook");

        let evaluation = evaluate_package(
            &workspace_root,
            &package_root,
            "pkg-draft-skill",
            "draft-skill",
            "Draft Skill",
            &notebook.description,
            1,
        )
        .expect("evaluation");

        assert!(matches!(
            evaluation.report.overall_status,
            crate::domain::eval::EvalOverallStatus::Usable
        ));
        assert!(matches!(
            evaluation.suggested_status,
            PackageStatus::Validated
        ));

        fs::remove_dir_all(workspace_root).ok();
    }
}
