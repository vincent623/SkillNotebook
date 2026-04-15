use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageVersion {
    pub id: String,
    pub package_id: String,
    pub version_number: u32,
    pub note: Option<String>,
    pub snapshot_path: String,
    pub eval_report_id: Option<String>,
    pub is_pinned: bool,
    pub created_at: String,
}
