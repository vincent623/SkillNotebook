use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageTestStatus {
    Passed,
    Failed,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageTestCheckResult {
    pub description: String,
    pub passed: bool,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageTestFileResult {
    pub path: String,
    pub name: String,
    pub passed: bool,
    pub checks: Vec<PackageTestCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageTestReport {
    pub id: String,
    pub package_id: String,
    pub status: PackageTestStatus,
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub files: Vec<PackageTestFileResult>,
    pub summary: String,
    pub created_at: String,
}
