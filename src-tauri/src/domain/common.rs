use serde::{Deserialize, Serialize};

use super::eval::EvalReport;
use super::package::SkillPackage;
use super::preview::PreviewModel;
use super::version::PackageVersion;
use super::workspace::Workspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<AppErrorPayload>,
}

impl<T> AppResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(AppErrorPayload {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrap {
    pub workspace: Workspace,
    pub packages: Vec<SkillPackage>,
    pub eval_reports: Vec<EvalReport>,
    pub versions: Vec<PackageVersion>,
    pub previews: Vec<PreviewModel>,
    pub selected_package_id: Option<String>,
    pub activity_log: Vec<String>,
}
