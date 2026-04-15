use crate::domain::common::AppBootstrap;
use crate::domain::package::PackageStatus;
use crate::storage::filesystem;

pub fn build_bootstrap(root_path: Option<&str>) -> Result<AppBootstrap, String> {
    let scanned = filesystem::scan_workspace(root_path)?;
    let needs_eval_count = scanned
        .packages
        .iter()
        .filter(|item| matches!(item.status, PackageStatus::Draft | PackageStatus::NeedsEval))
        .count();

    let mut activity_log = vec![
        format!("Loaded workspace from {}.", scanned.workspace.root_path),
        format!("Discovered {} skill package(s).", scanned.packages.len()),
    ];

    if needs_eval_count > 0 {
        activity_log.push(format!(
            "{} package(s) currently need evaluation before the next formal save.",
            needs_eval_count
        ));
    }

    Ok(AppBootstrap {
        selected_package_id: scanned.packages.first().map(|item| item.id.clone()),
        workspace: scanned.workspace,
        packages: scanned.packages,
        eval_reports: scanned.eval_reports,
        versions: scanned.versions,
        previews: scanned.previews,
        activity_log,
    })
}
