use std::fs;
use std::path::Path;
use std::sync::Arc;

use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
struct CreateSkillArgs {
    /// The name of the skill (e.g., 'capt-jack')
    name: String,
    /// A short description of what the skill does
    description: String,
}

/// Creates a new skill directory and SKILL.md file in the .skills directory.
#[tool]
async fn create_skill(args: CreateSkillArgs) -> std::result::Result<Value, AdkError> {
    let name = args.name.trim().replace(" ", "-").to_lowercase();
    let description = args.description;

    let path = Path::new(".skills").join(&name);
    if path.exists() {
        return Err(AdkError::tool(format!("Skill '{}' already exists.", name)));
    }

    fs::create_dir_all(&path)
        .map_err(|e| AdkError::tool(format!("Failed to create directory: {e}")))?;

    let content = format!(
        "---\nname: {}\ndescription: {}\n---\n\n# {}\n\n## Description\n{}\n",
        name, description, name, description
    );

    fs::write(path.join("SKILL.md"), content)
        .map_err(|e| AdkError::tool(format!("Failed to write SKILL.md: {e}")))?;

    Ok(json!({
        "status": "success",
        "message": format!("Successfully created skill '{}' at .skills/{}", name, name)
    }))
}

pub fn create_skill_tool() -> Vec<Arc<dyn Tool>> {
    // The #[tool] macro generates a struct named after the function (CreateSkill)
    vec![Arc::new(CreateSkill)]
}
