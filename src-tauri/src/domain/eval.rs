use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalOverallStatus {
    Usable,
    NeedsImprovement,
    Problematic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalDetails {
    pub has_skill_md: bool,
    pub has_examples: bool,
    pub has_prompts: bool,
    pub has_scripts: bool,
    pub input_defined: bool,
    pub output_defined: bool,
    pub boundaries_clear: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalReport {
    pub id: String,
    pub package_id: String,
    pub completeness_score: f32,
    pub clarity_score: f32,
    pub executability_score: f32,
    pub overall_status: EvalOverallStatus,
    pub suggestions: Vec<String>,
    pub details: EvalDetails,
    pub created_at: String,
}
