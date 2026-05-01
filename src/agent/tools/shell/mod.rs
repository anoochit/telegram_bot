use std::sync::Arc;
use adk_rust::serde::Deserialize;
use adk_tool::{tool, AdkError};
use adk_rust::Tool;
use schemars::JsonSchema;
use serde_json::{json, Value};
use tokio::process::Command;

#[derive(Deserialize, JsonSchema)]
struct ShellArgs {
    /// Command to execute
    command: String,
}

/// Executes allowed system commands safely.
#[tool]
async fn execute_shell(args: ShellArgs) -> std::result::Result<Value, AdkError> {
    // Basic security: only allow specific commands
    let allowed_commands = vec!["git", "ls", "grep"];
    if !allowed_commands.iter().any(|&cmd| args.command.starts_with(cmd)) {
        return Err(AdkError::tool(format!("Command not allowed: {}", args.command)));
    }

    let output = Command::new("powershell")
        .arg("-Command")
        .arg(&args.command)
        .output()
        .await
        .map_err(|e| AdkError::tool(format!("Execution failed: {}", e)))?;

    if output.status.success() {
        Ok(json!({"stdout": String::from_utf8_lossy(&output.stdout)}))
    } else {
        Err(AdkError::tool(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

pub fn shell_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ExecuteShell)]
}
