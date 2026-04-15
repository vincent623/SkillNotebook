use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewModel {
    pub package_id: String,
    pub name: String,
    pub has_skill_md: bool,
    pub prompt_files: Vec<String>,
    pub example_files: Vec<String>,
    pub reference_files: Vec<String>,
    pub script_files: Vec<String>,
    pub test_files: Vec<String>,
    pub skill_md_preview: String,
    pub example_preview: String,
    pub final_preview: String,
}
