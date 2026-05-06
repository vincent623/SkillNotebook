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
    pub agent_runtime: AgentRuntimeConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AgentRuntimeConfig {
    pub mode: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub node_binary: Option<String>,
    pub sidecar_script: Option<String>,
    pub timeout_secs: Option<u64>,
    pub retry_attempts: Option<u64>,
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

pub fn update_agent_runtime_from_payload(payload: &serde_json::Value) -> Result<AppConfig, String> {
    let mut config = load_app_config();
    let Some(agent_runtime) = payload
        .get("agentRuntime")
        .and_then(|value| value.as_object())
    else {
        return Ok(config);
    };

    if let Some(value) = string_field(agent_runtime, "mode") {
        config.agent_runtime.mode = value;
    }
    if let Some(value) = string_field(agent_runtime, "provider") {
        config.agent_runtime.provider = value;
    }
    if let Some(value) = string_field(agent_runtime, "baseUrl") {
        config.agent_runtime.base_url = value;
    }
    if let Some(value) = string_field(agent_runtime, "model") {
        config.agent_runtime.model = value;
    }
    if let Some(value) = string_field(agent_runtime, "nodeBinary") {
        config.agent_runtime.node_binary = value;
    }
    if let Some(value) = string_field(agent_runtime, "sidecarScript") {
        config.agent_runtime.sidecar_script = value;
    }
    if let Some(value) = positive_u64_field(agent_runtime, "timeoutSecs") {
        config.agent_runtime.timeout_secs = value;
    }
    if let Some(value) = positive_u64_field(agent_runtime, "retryAttempts") {
        config.agent_runtime.retry_attempts = value;
    }

    let clear_api_key = agent_runtime
        .get("clearApiKey")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if clear_api_key {
        config.agent_runtime.api_key = None;
    } else if let Some(value) = string_field(agent_runtime, "apiKey") {
        config.agent_runtime.api_key = value;
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

fn positive_u64_field(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<Option<u64>> {
    let value = object.get(key)?;
    if value.is_null() {
        return Some(None);
    }
    if let Some(number) = value.as_u64().filter(|number| *number > 0) {
        return Some(Some(number));
    }
    if let Some(raw) = value.as_str() {
        return Some(raw.trim().parse::<u64>().ok().filter(|number| *number > 0));
    }
    Some(None)
}
