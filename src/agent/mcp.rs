use adk_rust::prelude::*;
use adk_tool::mcp::McpServerManager;
use std::sync::Arc;
use crate::agent::utils;

/// Loads MCP tools from `mcp.json` if it exists and attaches them to the agent builder.
/// It checks the workspace directory first, then the current directory.
pub async fn load_mcp_tools(mut builder: LlmAgentBuilder) -> anyhow::Result<LlmAgentBuilder> {
    let workspace_root = utils::get_workspace_root().await?;
    let workspace_mcp = workspace_root.join("mcp.json");
    
    let mcp_config_path = if workspace_mcp.exists() {
        Some(workspace_mcp)
    } else if std::path::Path::new("mcp.json").exists() {
        Some(std::path::PathBuf::from("mcp.json"))
    } else {
        None
    };

    if let Some(path) = mcp_config_path {
        let mcp_manager = McpServerManager::from_json_file(path.to_str().unwrap_or("mcp.json"))?;
        let mcp_manager = Arc::new(mcp_manager);

        // Start all servers and handle potential failures per server
        let results = mcp_manager.start_all().await;
        for (name, res) in results {
            if let Err(e) = res {
                log::error!("Failed to start MCP server '{}': {}", name, e);
            } else {
                log::info!("Started MCP server '{}'", name);
            }
        }

        builder = builder.toolset(mcp_manager).into();
    }

    Ok(builder)
}
