use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::domain::test::{
    PackageTestCheckResult, PackageTestFileResult, PackageTestReport, PackageTestStatus,
};
use crate::storage::filesystem;
use crate::utils::ids::slugify;
use crate::utils::time::now_iso;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SmokeTestDefinition {
    name: Option<String>,
    package: Option<String>,
    prompt: Option<String>,
    input: Option<Value>,
    #[serde(alias = "expected_output")]
    expected_output: Option<String>,
    #[serde(alias = "expects", alias = "expectations")]
    checks: Vec<String>,
}

#[derive(Debug, Clone)]
struct PackageTestContext {
    package_id: String,
    slug: String,
    name: String,
    corpus_lower: String,
    has_skill_md: bool,
    has_frontmatter: bool,
    has_prompts: bool,
    has_examples: bool,
    has_evals: bool,
    has_tests: bool,
    input_defined: bool,
    output_defined: bool,
    use_trigger_defined: bool,
}

pub fn run_package_test(
    package_id: &str,
    root_path: Option<&str>,
) -> Result<PackageTestReport, String> {
    let scanned = filesystem::scan_project_root(root_path)?;
    let package = scanned
        .packages
        .into_iter()
        .find(|item| item.id == package_id)
        .ok_or_else(|| format!("package not found: {}", package_id))?;
    let package_root = PathBuf::from(&package.root_path);
    let test_files = collect_test_json_files(&package_root)?;
    let created_at = now_iso();
    let report_id = format!(
        "test-{}-{}",
        package.slug,
        slugify(&created_at).trim_matches('-')
    );

    if test_files.is_empty() {
        return Ok(PackageTestReport {
            id: report_id,
            package_id: package.id,
            status: PackageTestStatus::Missing,
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            files: Vec::new(),
            summary: "No smoke test JSON files found under tests/.".to_string(),
            created_at,
        });
    }

    let context = build_context(&package_root, &package.id, &package.slug, &package.name)?;
    let mut file_results = Vec::new();
    for path in test_files {
        file_results.push(run_test_file(&package_root, &path, &context)?);
    }

    let total_tests = file_results.len() as u32;
    let passed_tests = file_results.iter().filter(|item| item.passed).count() as u32;
    let failed_tests = total_tests.saturating_sub(passed_tests);
    let status = if failed_tests == 0 {
        PackageTestStatus::Passed
    } else {
        PackageTestStatus::Failed
    };
    let summary = if matches!(status, PackageTestStatus::Passed) {
        format!("All {} smoke test file(s) passed.", total_tests)
    } else {
        format!(
            "{} of {} smoke test file(s) failed.",
            failed_tests, total_tests
        )
    };

    Ok(PackageTestReport {
        id: report_id,
        package_id: context.package_id,
        status,
        total_tests,
        passed_tests,
        failed_tests,
        files: file_results,
        summary,
        created_at,
    })
}

fn run_test_file(
    package_root: &Path,
    path: &Path,
    context: &PackageTestContext,
) -> Result<PackageTestFileResult, String> {
    let relative = relative_path(package_root, path)?;
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {}", path.display(), error))?;
    let definition = match serde_json::from_str::<SmokeTestDefinition>(&content) {
        Ok(definition) => definition,
        Err(error) => {
            return Ok(PackageTestFileResult {
                path: relative,
                name: path
                    .file_stem()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| "smoke-test".to_string()),
                passed: false,
                checks: vec![PackageTestCheckResult {
                    description: "Test file parses as JSON.".to_string(),
                    passed: false,
                    evidence: format!("Parse error: {}", error),
                }],
            });
        }
    };

    let mut checks = Vec::new();
    checks.push(PackageTestCheckResult {
        description: "Test file parses as JSON.".to_string(),
        passed: true,
        evidence: "JSON loaded successfully.".to_string(),
    });

    let has_input = definition
        .prompt
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || definition
            .input
            .as_ref()
            .map(value_has_content)
            .unwrap_or(false);
    checks.push(PackageTestCheckResult {
        description: "Smoke test defines input or prompt.".to_string(),
        passed: has_input,
        evidence: if has_input {
            "Input material is present.".to_string()
        } else {
            "Add a non-empty `input` or `prompt` field.".to_string()
        },
    });

    let has_expected = definition
        .expected_output
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || !definition.checks.is_empty();
    checks.push(PackageTestCheckResult {
        description: "Smoke test defines expectations.".to_string(),
        passed: has_expected,
        evidence: if has_expected {
            "Expected output or checks are present.".to_string()
        } else {
            "Add `expectedOutput`, `checks`, or `expects`.".to_string()
        },
    });

    if let Some(package_name) = definition.package.as_deref() {
        let normalized = slugify(package_name);
        let matches_package = normalized == context.slug
            || normalized == context.package_id
            || normalized == slugify(&context.name);
        checks.push(PackageTestCheckResult {
            description: "Smoke test targets this package.".to_string(),
            passed: matches_package,
            evidence: if matches_package {
                format!("`{}` matches the selected package.", package_name)
            } else {
                format!(
                    "`{}` does not match `{}` or `{}`.",
                    package_name, context.slug, context.package_id
                )
            },
        });
    }

    let expectations = if definition.checks.is_empty() {
        definition
            .expected_output
            .iter()
            .map(|value| format!("Expected output: {}", value))
            .collect::<Vec<_>>()
    } else {
        definition.checks.clone()
    };

    for expectation in expectations {
        checks.push(evaluate_expectation(&expectation, context));
    }

    let passed = checks.iter().all(|item| item.passed);
    Ok(PackageTestFileResult {
        path: relative,
        name: definition
            .name
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "smoke-test".to_string()),
        passed,
        checks,
    })
}

fn evaluate_expectation(expectation: &str, context: &PackageTestContext) -> PackageTestCheckResult {
    let lowered = expectation.to_lowercase();
    let (passed, evidence) = if contains_any(&lowered, &["frontmatter", "validate", "validates"]) {
        (
            context.has_skill_md && context.has_frontmatter,
            if context.has_frontmatter {
                "SKILL.md starts with YAML frontmatter.".to_string()
            } else {
                "SKILL.md has no YAML frontmatter block.".to_string()
            },
        )
    } else if lowered.contains("prompt")
        && lowered.contains("example")
        && (lowered.contains("eval") || lowered.contains("test"))
    {
        let passed =
            context.has_prompts && context.has_examples && (context.has_evals || context.has_tests);
        (
            passed,
            format!(
                "prompts={}, examples={}, evals={}, tests={}",
                context.has_prompts, context.has_examples, context.has_evals, context.has_tests
            ),
        )
    } else if contains_any(&lowered, &["when to use", "use when", "trigger"]) {
        (
            context.use_trigger_defined,
            if context.use_trigger_defined {
                "The package describes when to use the skill.".to_string()
            } else {
                "Add a `Use when` description or `When to Use` section.".to_string()
            },
        )
    } else if contains_any(&lowered, &["output", "outputs", "deliverable", "result"]) {
        (
            context.output_defined,
            if context.output_defined {
                "The package defines output expectations.".to_string()
            } else {
                "Add an Outputs section or expected output contract.".to_string()
            },
        )
    } else if contains_any(&lowered, &["input", "inputs", "source"]) {
        (
            context.input_defined,
            if context.input_defined {
                "The package defines input expectations.".to_string()
            } else {
                "Add an Inputs section or input contract.".to_string()
            },
        )
    } else {
        let keywords = expectation_keywords(&lowered);
        if keywords.is_empty() {
            (true, "Expectation has no checkable keywords.".to_string())
        } else {
            let missing = keywords
                .iter()
                .filter(|keyword| !keyword_present(&context.corpus_lower, keyword))
                .cloned()
                .collect::<Vec<_>>();
            (
                missing.is_empty(),
                if missing.is_empty() {
                    format!("Matched keywords: {}.", keywords.join(", "))
                } else {
                    format!(
                        "Missing keywords in package content: {}.",
                        missing.join(", ")
                    )
                },
            )
        }
    };

    PackageTestCheckResult {
        description: expectation.to_string(),
        passed,
        evidence,
    }
}

fn build_context(
    package_root: &Path,
    package_id: &str,
    slug: &str,
    name: &str,
) -> Result<PackageTestContext, String> {
    let skill_path = package_root.join("SKILL.md");
    let skill_content = fs::read_to_string(&skill_path).unwrap_or_default();
    let skill_lower = skill_content.to_lowercase();
    let corpus_lower = read_package_corpus(package_root)?.to_lowercase();
    let has_skill_md = skill_path.exists();
    let has_frontmatter = skill_content.trim_start().starts_with("---");
    let has_prompts = contains_regular_file(&package_root.join("prompts"))?;
    let has_examples = contains_regular_file(&package_root.join("examples"))?;
    let has_evals = contains_regular_file(&package_root.join("evals"))?;
    let has_tests = contains_regular_file(&package_root.join("tests"))?;
    let input_defined = contains_any(
        &skill_lower,
        &["## inputs", "input", "inputs", "source material"],
    );
    let output_defined = contains_any(
        &skill_lower,
        &[
            "## outputs",
            "output",
            "outputs",
            "deliverable",
            "result",
            "summary",
        ],
    );
    let use_trigger_defined = contains_any(&skill_lower, &["when to use", "use when"])
        || corpus_lower.contains("use when");

    Ok(PackageTestContext {
        package_id: package_id.to_string(),
        slug: slug.to_string(),
        name: name.to_string(),
        corpus_lower,
        has_skill_md,
        has_frontmatter,
        has_prompts,
        has_examples,
        has_evals,
        has_tests,
        input_defined,
        output_defined,
        use_trigger_defined,
    })
}

fn collect_test_json_files(package_root: &Path) -> Result<Vec<PathBuf>, String> {
    let tests_root = package_root.join("tests");
    if !tests_root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_json_files(&tests_root, &mut files)?;
    files.sort_by(|left, right| left.to_string_lossy().cmp(&right.to_string_lossy()));
    Ok(files)
}

fn collect_json_files(current: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("failed to read directory {}: {}", current.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("failed to inspect directory entry: {}", error))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {}", path.display(), error))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_json_files(&path, files)?;
        } else if metadata.is_file()
            && path
                .extension()
                .map(|value| value.to_string_lossy().eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        {
            files.push(path);
        }
    }

    Ok(())
}

fn read_package_corpus(package_root: &Path) -> Result<String, String> {
    let mut corpus = String::new();
    collect_corpus(package_root, package_root, &mut corpus)?;
    Ok(corpus)
}

fn collect_corpus(package_root: &Path, current: &Path, corpus: &mut String) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("failed to read directory {}: {}", current.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("failed to inspect directory entry: {}", error))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "notebook.json" || name == "tests" {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {}", path.display(), error))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_corpus(package_root, &path, corpus)?;
        } else if metadata.is_file() && is_text_corpus_file(&path) {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read {}: {}", path.display(), error))?;
            corpus.push('\n');
            corpus.push_str(&relative_path(package_root, &path)?);
            corpus.push('\n');
            corpus.push_str(&content);
        }
    }

    Ok(())
}

fn is_text_corpus_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    if name == "SKILL.md" {
        return true;
    }

    matches!(
        path.extension()
            .map(|value| value.to_string_lossy().to_lowercase())
            .as_deref(),
        Some("md" | "txt" | "json" | "sh")
    )
}

fn contains_regular_file(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    if path.is_file() {
        return Ok(true);
    }
    for entry in fs::read_dir(path)
        .map_err(|error| format!("failed to read directory {}: {}", path.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("failed to inspect directory entry: {}", error))?;
        let child = entry.path();
        let metadata = fs::symlink_metadata(&child)
            .map_err(|error| format!("failed to inspect {}: {}", child.display(), error))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_file() {
            return Ok(true);
        }
        if metadata.is_dir() && contains_regular_file(&child)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn relative_path(package_root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(package_root)
        .map(|value| value.to_string_lossy().replace('\\', "/"))
        .map_err(|error| format!("failed to compute relative path: {}", error))
}

fn value_has_content(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn expectation_keywords(value: &str) -> Vec<String> {
    let stop_words = [
        "about", "after", "before", "check", "clear", "define", "defines", "expected", "file",
        "files", "from", "into", "output", "package", "present", "result", "should", "skill",
        "test", "that", "this", "what", "when", "with",
    ];

    value
        .split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|item| item.chars().count() >= 4)
        .filter(|item| !stop_words.contains(item))
        .take(8)
        .map(ToString::to_string)
        .collect()
}

fn keyword_present(corpus: &str, keyword: &str) -> bool {
    if corpus.contains(keyword) {
        return true;
    }
    if keyword.ends_with('s') && keyword.len() > 4 {
        let singular = keyword.trim_end_matches('s');
        if corpus.contains(singular) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::run_package_test;
    use crate::domain::test::PackageTestStatus;
    use crate::storage::filesystem;

    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_project_root_path() -> PathBuf {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "skill-notebook-test-service-{}-{}",
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
        filesystem::copy_directory_recursive(&root, destination).expect("copy project root");
        destination.clone()
    }

    #[test]
    fn runs_package_smoke_tests() {
        let report = run_package_test("pkg-interview", None).expect("test report");

        assert_eq!(report.status, PackageTestStatus::Passed);
        assert_eq!(report.total_tests, 1);
        assert_eq!(report.passed_tests, 1);
        assert!(report.files[0]
            .checks
            .iter()
            .any(|item| item.description.contains("insight")));
    }

    #[test]
    fn reports_missing_when_package_has_no_tests() {
        let report = run_package_test("pkg-meeting", None).expect("test report");

        assert_eq!(report.status, PackageTestStatus::Missing);
        assert_eq!(report.total_tests, 0);
        assert!(report.summary.contains("No smoke test"));
    }

    #[test]
    fn reports_malformed_smoke_test_as_failed_file() {
        let project_root_path = tmp_project_root_path();
        let root = copy_example_project_root(&project_root_path);
        let package_root =
            filesystem::canonical_skills_root(&root).join("meeting-actions-synthesizer");
        filesystem::write_text_file(
            &package_root.join("tests").join("smoke-test.json"),
            "{ not json",
        )
        .expect("write malformed smoke test");

        let report =
            run_package_test("pkg-meeting", Some(root.to_string_lossy().as_ref())).expect("report");

        assert_eq!(report.status, PackageTestStatus::Failed);
        assert_eq!(report.total_tests, 1);
        assert_eq!(report.failed_tests, 1);
        assert!(report.files[0].checks[0].evidence.contains("Parse error"));
    }
}
