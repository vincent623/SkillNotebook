use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub target_platform: String,
    pub handoff: HandoffConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HandoffConfig {
    pub terminal_command: Option<String>,
    pub editor_command: Option<String>,
    pub agent_command: Option<String>,
    pub global_claude_skills_dir: Option<String>,
    pub project_claude_skills_dir_name: Option<String>,
}

impl Default for HandoffConfig {
    fn default() -> Self {
        Self {
            terminal_command: Some("open -a Terminal".to_string()),
            editor_command: None,
            agent_command: Some("codex".to_string()),
            global_claude_skills_dir: Some("~/.claude/skills".to_string()),
            project_claude_skills_dir_name: Some(".claude/skills".to_string()),
        }
    }
}

pub fn app_settings_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("SKILL_NOTEBOOK_SETTINGS_FILE") {
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
            .join("settings.json"),
    )
}

pub fn load_app_config() -> AppConfig {
    let Some(path) = app_settings_path() else {
        return AppConfig::default();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return AppConfig::default();
    };

    serde_json::from_str::<AppConfig>(&content).unwrap_or_default()
}

pub fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let path =
        app_settings_path().ok_or_else(|| "failed to resolve app settings path".to_string())?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("invalid app settings path: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create settings directory: {}", error))?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|error| format!("failed to serialize settings: {}", error))?;
    fs::write(&path, format!("{}\n", content))
        .map_err(|error| format!("failed to write settings {}: {}", path.display(), error))?;

    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&path)
            .map_err(|error| format!("failed to read settings metadata: {}", error))?
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&path, permissions)
            .map_err(|error| format!("failed to secure settings permissions: {}", error))?;
    }

    Ok(())
}

pub fn update_handoff_from_payload(payload: &serde_json::Value) -> Result<AppConfig, String> {
    let mut config = load_app_config();
    let Some(handoff) = payload.get("handoff").and_then(|value| value.as_object()) else {
        return Ok(config);
    };

    if let Some(value) = string_field(handoff, "terminalCommand") {
        config.handoff.terminal_command = value;
    }
    if let Some(value) = string_field(handoff, "editorCommand") {
        config.handoff.editor_command = value;
    }
    if let Some(value) = string_field(handoff, "agentCommand") {
        config.handoff.agent_command = value;
    }
    if let Some(value) = string_field(handoff, "globalClaudeSkillsDir") {
        config.handoff.global_claude_skills_dir = value;
    }
    if let Some(value) = string_field(handoff, "projectClaudeSkillsDirName") {
        config.handoff.project_claude_skills_dir_name = value;
    }

    save_app_config(&config)?;
    Ok(config)
}

fn string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<Option<String>> {
    let value = object.get(key)?;
    if value.is_null() {
        return Some(None);
    }
    let Some(raw) = value.as_str() else {
        return Some(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Some(None)
    } else {
        Some(Some(trimmed.to_string()))
    }
}
