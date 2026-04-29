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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionDiffChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionDiffEntry {
    pub path: String,
    pub change_type: VersionDiffChangeType,
    pub diff_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageVersionDiff {
    pub version_id: String,
    pub package_id: String,
    pub version_number: u32,
    pub snapshot_path: String,
    pub entries: Vec<VersionDiffEntry>,
}
