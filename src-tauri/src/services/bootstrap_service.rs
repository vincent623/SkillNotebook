use crate::domain::common::AppBootstrap;
use crate::domain::package::PackageStatus;
use crate::services::skill_create_service;
use crate::storage::filesystem;

pub fn build_bootstrap(root_path: Option<&str>) -> Result<AppBootstrap, String> {
    let scanned = filesystem::scan_project_root(root_path)?;
    let cleaned_previews =
        skill_create_service::cleanup_stale_package_previews(&scanned.project_root.root_path)
            .unwrap_or_default();
    let needs_eval_count = scanned
        .packages
        .iter()
        .filter(|item| matches!(item.status, PackageStatus::Draft | PackageStatus::NeedsEval))
        .count();

    let mut activity_log = vec![
        format!(
            "Loaded project root from {}.",
            scanned.project_root.root_path
        ),
        format!(
            "Discovered {} skill package(s) under .skills/.",
            scanned.packages.len()
        ),
    ];

    if needs_eval_count > 0 {
        activity_log.push(format!(
            "{} package(s) currently need evaluation before the next formal save.",
            needs_eval_count
        ));
    }
    if cleaned_previews > 0 {
        activity_log.push(format!(
            "Removed {} stale create preview workspace(s).",
            cleaned_previews
        ));
    }

    Ok(AppBootstrap {
        selected_package_id: scanned.packages.first().map(|item| item.id.clone()),
        project_root: scanned.project_root,
        packages: scanned.packages,
        eval_reports: scanned.eval_reports,
        versions: scanned.versions,
        previews: scanned.previews,
        activity_log,
    })
}
