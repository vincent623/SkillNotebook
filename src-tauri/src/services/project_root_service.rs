use crate::domain::project_root::ProjectRoot;
use crate::storage::filesystem;
use crate::utils::ids::slugify;
use crate::utils::time::now_iso;
use serde_json::json;

pub fn default_project_root() -> Result<ProjectRoot, String> {
    Ok(filesystem::scan_project_root(None)?.project_root)
}

pub fn open_project_root(root_path: &str) -> Result<ProjectRoot, String> {
    Ok(filesystem::scan_project_root(Some(root_path))?.project_root)
}

pub fn recent_project_roots(root_paths: Vec<String>) -> Result<Vec<ProjectRoot>, String> {
    let mut project_roots = Vec::new();

    for root_path in root_paths {
        if let Ok(project_root) = open_project_root(&root_path) {
            project_roots.push(project_root);
        }
    }

    if project_roots.is_empty() {
        project_roots.push(default_project_root()?);
    }

    Ok(project_roots)
}

pub fn draft_project_root(name: &str) -> ProjectRoot {
    let base_root = filesystem::default_project_root()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(filesystem::default_project_root);
    let slug = slugify(name);

    ProjectRoot {
        id: format!("project-root-{}", slug),
        name: name.to_string(),
        root_path: base_root.join(&slug).to_string_lossy().to_string(),
        created_at: "pending".to_string(),
        updated_at: "pending".to_string(),
        last_opened_at: None,
    }
}

pub fn create_project_root(name: &str) -> Result<ProjectRoot, String> {
    let mut draft = draft_project_root(name);
    if draft.root_path.trim().is_empty() {
        draft.root_path = filesystem::default_project_root()
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
        return Err(format!(
            "project root already exists: {}",
            root_path.display()
        ));
    }

    filesystem::ensure_directory(&filesystem::canonical_skills_root(&root_path))?;
    filesystem::ensure_directory(&root_path.join(".skill-notebook").join("snapshots"))?;
    filesystem::ensure_directory(&root_path.join(".skill-notebook").join("logs"))?;
    filesystem::ensure_directory(&root_path.join(".skill-notebook").join("cache"))?;

    let now = now_iso();
    let project_root = ProjectRoot {
        id: format!("project-root-{}", slugify(name)),
        name: name.to_string(),
        root_path: root_path.to_string_lossy().to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
        last_opened_at: Some(now.clone()),
    };

    filesystem::write_json_file(
        &root_path.join(".skill-notebook").join("config.json"),
        &json!({
            "id": project_root.id,
            "name": project_root.name,
            "createdAt": project_root.created_at,
            "updatedAt": project_root.updated_at,
            "lastOpenedAt": project_root.last_opened_at,
        }),
    )?;

    Ok(project_root)
}
