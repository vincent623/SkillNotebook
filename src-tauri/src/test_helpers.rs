use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::filesystem;

pub fn tmp_project_root_path() -> PathBuf {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "skill-notebook-test-{}-{}",
        std::process::id(),
        seed
    ))
}

pub fn copy_example_project_root(destination: &Path) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
        .join("project-root");
    filesystem::copy_directory_recursive(&root, destination).expect("copy project root");
    destination.to_path_buf()
}
