use std::sync::Arc;
use adk_rust::prelude::*;

// OpenAI-compatible API
use adk_rust::model::{OpenAIClient, OpenAIConfig};

pub mod utils;
pub mod database;
pub mod current_datetime_tool;
pub mod filesystem_tool;
pub mod km_tool;
pub mod shell_tool;
pub mod weather_tool;
pub mod web_fetch_tool;
pub mod system_info_tool;
pub mod mcp;


pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>)> {

    // Sample for ThaiLLM OpenAI-compatible API
    // Load the API key from an environment variable
    let api_key = std::env::var("THAILLM_API_KEY")?;

    // Create the OpenAI client with the custom configuration
    let config = OpenAIConfig::compatible(
        &api_key,
        "https://thaillm.or.th/api/v1",
        "typhoon-s-thaillm-8b-instruct",
    );

    // Create the OpenAI client with the custom configuration
    let model =  Arc::new(OpenAIClient::new(config)?);

    // Sample for Gemini
    // let api_key = std::env::var("GOOGLE_API_KEY")?;
    // let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.5-flash")?);

    // Get the current project root path
    let project_root = std::env::current_dir()?;
    
    // Get the workspace root
    let workspace_root = utils::get_workspace_root().await?;

    // Build the agent with the model and tools
    let mut builder = LlmAgentBuilder::new("agent")
        .description("A helpful AI assistant")
        .instruction("You are a professional, secure, and proactive AI Agent assistant. 
Your goal is to assist the user by executing tasks accurately using your available tools.

Guidelines for Interaction:
1. Tool-First Approach: Always prioritize using your tools (FileSystem, Weather, Shell, KM, etc.) to perform actions, retrieve data, or verify information.
2. Knowledge Retrieval: Check internal knowledge (memory, KM tool, and local docs) before relying on general training data.
3. Precision & Security: Be concise and technically accurate. Never disclose sensitive credentials, API keys, or environment secrets.
4. Transparency: If a request exceeds your capabilities or toolset, clearly state your limitations. Never hallucinate.
5. Formatting: Use Markdown for structure. Present structured data in tables when it improves readability.
6. Language: You MUST always answer and communicate with the user language.
7. Final Output: Provide response messages in clear, direct text, table.")
        .model(model.clone())
        .with_skills_from_root(project_root)?
        .with_skills_from_root(workspace_root)?;

    // add tools to the agent
    let mut tools = weather_tool::weather_tools();
    tools.extend(filesystem_tool::filesystem_tools());
    tools.extend(current_datetime_tool::datetime_tools());
    tools.extend(km_tool::km_tools());
    tools.extend(shell_tool::shell_tools());
    tools.extend(web_fetch_tool::web_fetch_tools());
    tools.extend(system_info_tool::system_info_tools());
 
    // Add tools to the agent builder
    for t in tools {
        builder = builder.tool(t).into();
    }

    // Load MCP tools from mcp.json if it exists
    builder = mcp::load_mcp_tools(builder).await?;

    // Build and return the agent
    let agent = builder.build()?;
    
    Ok((Arc::new(agent), model))
}
