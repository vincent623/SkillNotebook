use serde::{Deserialize, Serialize};

use super::eval::EvalReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftSourceKind {
    Text,
    Files,
    Url,
    Empty,
}

impl Default for DraftSourceKind {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftStartRequest {
    pub project_root_id: String,
    pub prompt: Option<String>,
    pub source_paths: Option<Vec<String>>,
    pub source_url: Option<String>,
    pub preferred_agent_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftImportRequest {
    pub project_root_id: String,
    pub draft_id: String,
    pub run_eval: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftDiscardRequest {
    pub project_root_id: String,
    pub draft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftWorkspace {
    pub draft_id: String,
    pub project_root_id: String,
    pub draft_path: String,
    pub brief_path: String,
    pub intended_slug: String,
    pub source_kind: DraftSourceKind,
    pub source_summary: String,
    pub suggested_command: String,
    pub import_command: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftImportResponse {
    pub draft_id: String,
    pub package_id: String,
    pub slug: String,
    pub package_path: String,
    pub eval_report: Option<EvalReport>,
    pub eval_command: String,
    pub version_command: String,
    pub reference_command: String,
    pub imported_at: String,
}
