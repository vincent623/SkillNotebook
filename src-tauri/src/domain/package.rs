use serde::{Deserialize, Serialize};

use super::eval::EvalOverallStatus;
use super::eval::EvalReport;
use super::version::PackageVersion;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageStatus {
    Draft,
    Evaluating,
    Validated,
    NeedsEval,
    Archived,
}

impl Default for PackageStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPackage {
    pub id: String,
    pub workspace_id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub status: PackageStatus,
    pub root_path: String,
    pub current_version: u32,
    pub last_eval_status: Option<EvalOverallStatus>,
    pub related_skills: Vec<String>,
    pub bundle_candidates: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PackageNotebookDocument {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub status: PackageStatus,
    pub current_version: u32,
    pub last_eval_status: Option<EvalOverallStatus>,
    pub related_skills: Vec<String>,
    pub bundle_candidates: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub versions: Vec<PackageVersion>,
    pub eval_reports: Vec<EvalReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub package_id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub updated_at: String,
    pub status: PackageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePackageFromNlRequest {
    pub workspace_id: String,
    pub prompt: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePackageFromNlResponse {
    pub package_id: String,
    pub name: String,
    pub slug: String,
    pub root_path: String,
    pub eval_workspace_path: String,
    pub draft_created: bool,
    pub auto_eval_started: bool,
    pub validation_summary: String,
    pub generator_used: String,
    pub generation_summary: String,
}
