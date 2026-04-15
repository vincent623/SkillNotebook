use crate::domain::package::SearchResult;
use crate::storage::filesystem;

pub fn search_packages(query: &str, root_path: Option<&str>) -> Result<Vec<SearchResult>, String> {
    let lowered_query = query.trim().to_lowercase();
    let results = filesystem::scan_workspace(root_path)?
        .packages
        .into_iter()
        .filter(|item| {
            let haystack = format!("{} {} {}", item.name, item.description, item.tags.join(" "))
                .to_lowercase();
            haystack.contains(&lowered_query)
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
