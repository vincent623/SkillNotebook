use crate::domain::package::SearchResult;
use crate::storage::filesystem;
use crate::utils::errors::AppError;

pub fn search_packages(query: &str, root_path: Option<&str>) -> Result<Vec<SearchResult>, AppError> {
    let lowered_query = query.trim().to_lowercase();
    if lowered_query.is_empty() {
        return Ok(Vec::new());
    }
    let results = filesystem::scan_project_root(root_path)?
        .packages
        .into_iter()
        .filter(|item| {
            item.name.to_lowercase().contains(&lowered_query)
                || item.description.to_lowercase().contains(&lowered_query)
                || item.tags.iter().any(|tag| tag.to_lowercase().contains(&lowered_query))
        })
        .map(|item| SearchResult {
            package_id: item.id,
            name: item.name,
            description: item.description,
            tags: item.tags,
            updated_at: item.updated_at,
            status: item.status,
        })
        .collect::<Vec<_>>();

    Ok(results)
}
