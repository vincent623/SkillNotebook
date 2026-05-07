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
    pub project_root_id: String,
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
pub struct PackageImportRequest {
    pub project_root_id: String,
    pub source_path: String,
    pub slug: Option<String>,
    pub run_eval: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageImportResponse {
    pub package_id: String,
    pub slug: String,
    pub package_path: String,
    pub eval_report: Option<EvalReport>,
    pub eval_command: String,
    pub version_command: String,
    pub reference_command: String,
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PackageUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<PackageStatus>,
    pub related_skills: Option<Vec<String>>,
    pub bundle_candidates: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageFileEntry {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<PackageFileEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageFileContent {
    pub path: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageExportArtifact {
    pub package_id: String,
    pub zip_path: String,
    pub size_bytes: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageReferenceItemKind {
    Path,
    Snippet,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageReferenceItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub kind: PackageReferenceItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageReferenceResponse {
    pub package_id: String,
    pub slug: String,
    pub package_path: String,
    pub skill_md_path: String,
    pub items: Vec<PackageReferenceItem>,
}
