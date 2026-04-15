use crate::domain::workspace::Workspace;
use crate::storage::filesystem;
use crate::utils::ids::slugify;
use crate::utils::time::now_iso;
use serde_json::json;

pub fn default_workspace() -> Result<Workspace, String> {
    Ok(filesystem::scan_workspace(None)?.workspace)
}

pub fn open_workspace(root_path: &str) -> Result<Workspace, String> {
    Ok(filesystem::scan_workspace(Some(root_path))?.workspace)
}

pub fn recent_workspaces(root_paths: Vec<String>) -> Result<Vec<Workspace>, String> {
    let mut workspaces = Vec::new();

    for root_path in root_paths {
        if let Ok(workspace) = open_workspace(&root_path) {
            workspaces.push(workspace);
        }
    }

    if workspaces.is_empty() {
        workspaces.push(default_workspace()?);
    }

    Ok(workspaces)
}

pub fn draft_workspace(name: &str) -> Workspace {
    let base_root = filesystem::default_workspace_root()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(filesystem::default_workspace_root);
    let slug = slugify(name);

    Workspace {
        id: format!("workspace-{}", slug),
        name: name.to_string(),
        root_path: base_root.join(&slug).to_string_lossy().to_string(),
        created_at: "pending".to_string(),
        updated_at: "pending".to_string(),
        last_opened_at: None,
    }
}

pub fn create_workspace(name: &str) -> Result<Workspace, String> {
    let mut draft = draft_workspace(name);
    if draft.root_path.trim().is_empty() {
        draft.root_path = filesystem::default_workspace_root()
            .to_string_lossy()
            .to_string();
    }

    let root_path = std::path::PathBuf::from(&draft.root_path);
    if root_path.exists()
        && root_path
            .join(".skill-notebook")
            .join("config.json")
            .exists()
    {
        return Err(format!("workspace already exists: {}", root_path.display()));
    }

    filesystem::ensure_directory(&root_path.join("packages"))?;
    filesystem::ensure_directory(&root_path.join(".skill-notebook").join("snapshots"))?;
    filesystem::ensure_directory(&root_path.join(".skill-notebook").join("logs"))?;
    filesystem::ensure_directory(&root_path.join(".skill-notebook").join("cache"))?;

    let now = now_iso();
    let workspace = Workspace {
        id: format!("workspace-{}", slugify(name)),
        name: name.to_string(),
        root_path: root_path.to_string_lossy().to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
        last_opened_at: Some(now.clone()),
    };

    filesystem::write_json_file(
        &root_path.join(".skill-notebook").join("config.json"),
        &json!({
            "id": workspace.id,
            "name": workspace.name,
            "createdAt": workspace.created_at,
            "updatedAt": workspace.updated_at,
            "lastOpenedAt": workspace.last_opened_at,
        }),
    )?;

    Ok(workspace)
}
