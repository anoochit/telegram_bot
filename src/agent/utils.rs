use adk_rust::prelude::*;
use std::path::PathBuf;
use tokio::fs;

const WORKSPACE_NAME: &str = "workspace";

/// Returns the absolute path to the sandbox directory.
/// Ensures the directory exists on disk.
pub async fn get_workspace_root() -> std::result::Result<PathBuf, AdkError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AdkError::tool("Failed to get home directory"))?;

    let root = home.join(WORKSPACE_NAME);

    if !root.exists() {
        fs::create_dir_all(&root)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create workspace: {}", e)))?;
    }

    // Canonicalize for security checks
    Ok(fs::canonicalize(&root)
        .await
        .unwrap_or(root))
}
