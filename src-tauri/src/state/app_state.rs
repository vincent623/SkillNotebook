use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::storage::filesystem;

#[derive(Debug, Clone)]
struct ProjectRootSession {
    current_project_root: String,
    recent_project_roots: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRootSessionFile {
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
        persist_project_root_session(&session);

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
            project_root_session: Mutex::new(load_project_root_session(default_root)),
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

fn load_project_root_session(default_root: String) -> ProjectRootSession {
    let Some(path) = project_root_session_path() else {
        return ProjectRootSession {
            current_project_root: default_root.clone(),
            recent_project_roots: vec![default_root],
        };
    };

    let Ok(content) = fs::read_to_string(&path) else {
        return ProjectRootSession {
            current_project_root: default_root.clone(),
            recent_project_roots: vec![default_root],
        };
    };
    let Ok(session_file) = serde_json::from_str::<ProjectRootSessionFile>(&content) else {
        return ProjectRootSession {
            current_project_root: default_root.clone(),
            recent_project_roots: vec![default_root],
        };
    };

    let current = normalize_project_root(&session_file.current_project_root);
    let mut recent = Vec::new();
    for root in session_file.recent_project_roots {
        let normalized = normalize_project_root(&root);
        if normalized.is_empty() || recent.contains(&normalized) {
            continue;
        }
        recent.push(normalized);
        if recent.len() >= 10 {
            break;
        }
    }
    if !recent.contains(&current) {
        recent.insert(0, current.clone());
    }
    if recent.is_empty() {
        recent.push(default_root);
    }

    ProjectRootSession {
        current_project_root: current,
        recent_project_roots: recent,
    }
}

fn persist_project_root_session(session: &ProjectRootSession) {
    let Some(path) = project_root_session_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }

    let payload = ProjectRootSessionFile {
        current_project_root: session.current_project_root.clone(),
        recent_project_roots: session.recent_project_roots.clone(),
    };
    let Ok(content) = serde_json::to_string_pretty(&payload) else {
        return;
    };
    fs::write(path, format!("{}\n", content)).ok();
}

fn project_root_session_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("SKILL_NOTEBOOK_SESSION_FILE") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    let home = env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Skill Notebook")
            .join("session.json"),
    )
}
