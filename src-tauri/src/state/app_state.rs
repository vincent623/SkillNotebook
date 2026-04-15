use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::storage::filesystem;

#[derive(Debug, Clone)]
struct WorkspaceSession {
    current_workspace_root: String,
    recent_workspace_roots: Vec<String>,
}

#[derive(Debug)]
pub struct AppState {
    pub app_name: &'static str,
    workspace_session: Mutex<WorkspaceSession>,
}

impl AppState {
    pub fn current_workspace_root(&self) -> Result<String, String> {
        self.workspace_session
            .lock()
            .map_err(|_| "failed to lock workspace session".to_string())
            .map(|session| session.current_workspace_root.clone())
    }

    pub fn recent_workspace_roots(&self) -> Result<Vec<String>, String> {
        self.workspace_session
            .lock()
            .map_err(|_| "failed to lock workspace session".to_string())
            .map(|session| session.recent_workspace_roots.clone())
    }

    pub fn set_current_workspace_root(&self, root_path: &str) -> Result<String, String> {
        let normalized = normalize_workspace_root(root_path);
        let mut session = self
            .workspace_session
            .lock()
            .map_err(|_| "failed to lock workspace session".to_string())?;

        session.current_workspace_root = normalized.clone();
        session
            .recent_workspace_roots
            .retain(|item| item != &normalized);
        session.recent_workspace_roots.insert(0, normalized.clone());
        session.recent_workspace_roots.truncate(10);

        Ok(normalized)
    }
}

impl Default for AppState {
    fn default() -> Self {
        let default_root = normalize_workspace_root(
            filesystem::default_workspace_root()
                .to_string_lossy()
                .as_ref(),
        );

        Self {
            app_name: "Skill Notebook",
            workspace_session: Mutex::new(WorkspaceSession {
                current_workspace_root: default_root.clone(),
                recent_workspace_roots: vec![default_root],
            }),
        }
    }
}

fn normalize_workspace_root(root_path: &str) -> String {
    let path = PathBuf::from(root_path);
    fs::canonicalize(&path)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}
