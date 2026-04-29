use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::storage::filesystem;

#[derive(Debug, Clone)]
struct ProjectRootSession {
    current_project_root: String,
    recent_project_roots: Vec<String>,
}

#[derive(Debug)]
pub struct AppState {
    pub app_name: &'static str,
    project_root_session: Mutex<ProjectRootSession>,
}

impl AppState {
    pub fn current_project_root(&self) -> Result<String, String> {
        self.project_root_session
            .lock()
            .map_err(|_| "failed to lock project_root session".to_string())
            .map(|session| session.current_project_root.clone())
    }

    pub fn recent_project_roots(&self) -> Result<Vec<String>, String> {
        self.project_root_session
            .lock()
            .map_err(|_| "failed to lock project_root session".to_string())
            .map(|session| session.recent_project_roots.clone())
    }

    pub fn set_current_project_root(&self, root_path: &str) -> Result<String, String> {
        let normalized = normalize_project_root(root_path);
        let mut session = self
            .project_root_session
            .lock()
            .map_err(|_| "failed to lock project_root session".to_string())?;

        session.current_project_root = normalized.clone();
        session
            .recent_project_roots
            .retain(|item| item != &normalized);
        session.recent_project_roots.insert(0, normalized.clone());
        session.recent_project_roots.truncate(10);

        Ok(normalized)
    }
}

impl Default for AppState {
    fn default() -> Self {
        let default_root = normalize_project_root(
            filesystem::default_project_root()
                .to_string_lossy()
                .as_ref(),
        );

        Self {
            app_name: "Skill Notebook",
            project_root_session: Mutex::new(ProjectRootSession {
                current_project_root: default_root.clone(),
                recent_project_roots: vec![default_root],
            }),
        }
    }
}

fn normalize_project_root(root_path: &str) -> String {
    let path = PathBuf::from(root_path);
    fs::canonicalize(&path)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}
