use adk_rust::prelude::*;
use std::sync::Arc;

// OpenAI-compatible API
use adk_rust::model::{OpenAIClient, OpenAIConfig};

pub mod mcp;
pub mod tools;
pub mod utils;

pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>)> {
    // Load Persona, User Info, and Memories from files
    let agent_md = tokio::fs::read_to_string("AGENT.md").await.unwrap_or_else(|_| "Standard Assistant".to_string());
    let user_md = tokio::fs::read_to_string("USER.md").await.unwrap_or_else(|_| "Developer".to_string());
    let memories_md = tokio::fs::read_to_string("MEMORIES.md").await.unwrap_or_else(|_| "No previous memories.".to_string());

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
    let model = Arc::new(OpenAIClient::new(config)?);

    // Get the current project root path
    let project_root = std::env::current_dir()?;

    // Build the agent with the model and tools
    let mut builder = LlmAgentBuilder::new("agent")
        .description("A helpful and playful AI assistant")
        .instruction(format!(
"You are an AI Agent assistant. 

# YOUR SOUL (Persona)
{}

# THE USER (Context)
{}

# YOUR MEMORIES (Past Facts)
{}

# GUIDELINES FOR INTERACTION
1. Tool-First Approach: Always prioritize using your tools (google_search, web_fetch, Wiki, FileSystem, Weather, Shell, etc.) to perform actions, retrieve data, or verify information.
2. Knowledge Management (Wiki): Use the Wiki tools to store and retrieve long-term information. Treat the 'wiki/' directory as your primary memory.
   - To learn/save: Use add_wiki_page.
   - To find: Use search_wiki or list_wiki_pages.
   - To read: Use get_wiki_page.
   - To organize: Use summarize_wiki to update the summary index.
3. Personal Memories: When you learn a personal fact about the user (preferences, habits, secrets), use update_user_memory to save it permanently in MEMORIES.md.
4. Web Search & Content: 
   - Use google_search if you don't know something or need the latest information.
   - Use web_fetch to retrieve the full content of a specific URL.
5. Precision & Security: Stay concise and technically accurate. Never disclose sensitive credentials, API keys, or environment secrets.
6. Transparency: If a request exceeds your capabilities, clearly state your limitations in a friendly way.
7. Formatting: Use plain text only. Do NOT use Markdown formatting (no bold, italics, headers, or tables).
8. Language: You MUST always answer and communicate with the user in Thai. Use natural, lively, and professional Thai as defined in your Persona. Tool names and arguments should remain in English.
9. Final Output: Use plain text. For lists use indicators (e.g., '- ', '1. ') to maintain structure without Markdown.",
agent_md, user_md, memories_md))
        .model(model.clone())
        .with_skills_from_root(project_root)?;

    // add tools to the agent
    let mut tools: Vec<Arc<dyn Tool>> = tools::weather::weather_tools();
    tools.extend(tools::filesystem::filesystem_tools());
    tools.extend(tools::current_datetime::datetime_tools());
    tools.extend(tools::wiki::wiki_tools());
    tools.extend(tools::shell::shell_tools());
    tools.extend(tools::web_fetch::web_fetch_tools());
    tools.extend(tools::system_info::system_info_tools());
    tools.extend(tools::create_skill::create_skill_tool());
    tools.extend(tools::soul::soul_tools());
    tools.extend(tools::search::search_tools());

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
