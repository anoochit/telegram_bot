use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use adk_rust::tool::AgentTool;
use std::sync::Arc;

// OpenAI-compatible API
// use adk_rust::model::{OpenAIClient, OpenAIConfig};

pub mod mcp;
pub mod tools;
pub mod utils;

pub fn get_compaction_config(model: Arc<dyn Llm>) -> EventsCompactionConfig {
    EventsCompactionConfig {
        compaction_interval: 5,
        overlap_size: 2,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>)> {
    // Load Persona, User Info, and Memories from files
    let agent_md = tokio::fs::read_to_string("AGENT.md")
        .await
        .unwrap_or_else(|_| "Standard Assistant".to_string());
    // If USER.md or MEMORIES.md don't exist, use default values
    let user_md = tokio::fs::read_to_string("USER.md")
        .await
        .unwrap_or_else(|_| "Developer".to_string());
    // MEMORIES.md is for personal facts about the user that the agent should remember long-term. It can be updated by the agent using the update_user_memory tool.
    let memories_md = tokio::fs::read_to_string("MEMORIES.md")
        .await
        .unwrap_or_else(|_| "No previous memories.".to_string());

    // Sample 1: for ThaiLLM OpenAI-compatible API
    // Load the API key from an environment variable
    // let api_key = std::env
    //     ::var("THAILLM_API_KEY")
    //     .expect("THAILLM_API_KEY environment variable not set");

    // Create the OpenAI client with the custom configuration
    // let config = OpenAIConfig::compatible(
    //     &api_key,
    //     "https://thaillm.or.th/api/v1",
    //     "typhoon-s-thaillm-8b-instruct",
    // );

    // Create the OpenAI client with the custom configuration
    // let model = Arc::new(OpenAIClient::new(config)?);

    // Sample 2: for Gemini Model
    let api_key =
        std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY environment variable not set");
    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.5-flash")?);

    // Get the current project root path
    let project_root = std::env::current_dir()?;

    // Define specialized sub-agents
    let investigator = LlmAgentBuilder::new("codebase_investigator")
        .description(
            "Specialized in deep codebase analysis, architectural mapping, and understanding system-wide dependencies. Use this for bug root-cause analysis or planning large refactors."
        )
        .instruction(
            "You are a codebase investigator. Analyze the provided context, code, and logs to identify root causes of bugs or plan architectural improvements."
        )
        .model(model.clone())
        .build()?;

    let generalist = LlmAgentBuilder::new("generalist")
        .description(
            "A high-efficiency agent with access to all tools. Use this for repetitive batch tasks or high-volume data processing to keep the main conversation history lean."
        )
        .instruction(
            "You are a generalist agent. Perform the requested batch tasks or data processing efficiently."
        )
        .model(model.clone())
        .build()?;

    // Build the agent with the model and tools
    let mut builder = LlmAgentBuilder::new("agent")
        .description("A helpful and playful AI assistant")
        .instruction(
            format!(
                "You are an AI Agent assistant. 

# YOUR SOUL (Persona)
{}

# THE USER (Context)
{}

# YOUR MEMORIES (Past Facts)
{}

# GUIDELINES FOR INTERACTION
1. Tool-First Approach: Always prioritize using your tools (google_search, web_fetch, Wiki, FileSystem, Weather, Shell, etc.) to perform actions, retrieve data, or verify information.
2. Delegation: You have specialized sub-agents at your disposal. Use them for complex or turn-intensive tasks:
   - Use 'codebase_investigator' for deep code analysis or bug hunting.
   - Use 'generalist' for repetitive batch tasks or when processing large amounts of data.
3. Knowledge Management (Wiki): Use the Wiki tools to store and retrieve long-term information. Treat the 'wiki/' directory as your primary memory.
   - To learn/save: Use add_wiki_page.
   - To find: Use search_wiki or list_wiki_pages.
   - To read: Use get_wiki_page.
   - To organize: Use summarize_wiki to update the summary index.
4. Personal Memories: When you learn a personal fact about the user (preferences, habits, secrets), use update_user_memory to save it permanently in MEMORIES.md.
5. Web Search & Content: 
   - Use google_search if you don't know something or need the latest information.
   - Use web_fetch to retrieve the full content of a specific URL.
6. Task Management & Decomposition: Use the TODO tools to manage complex workflows.
   - Decomposition: For large or multi-step requests, first split the goal into smaller, actionable sub-tasks and add them using add_todo.
   - Tracking: Keep the TODO list updated. Use list_todos to see what's left.
   - Execution: You can execute multiple tools in a single response to complete several sub-tasks if appropriate.
7. Precision & Security: Stay concise and technically accurate. Never disclose sensitive credentials, API keys, or environment secrets.
8. Transparency: If a request exceeds your capabilities, clearly state your limitations in a friendly way.
9. Formatting: Do NOT use any Markdown formatting (no bold, italics, headers, or tables). Output responses as plain text only.
10. Language: You MUST always answer and communicate with the user in Thai. Use natural, lively, and professional Thai as defined in your Persona. Tool names and arguments should remain in English.
11. Final Output: Use plain text only. For lists, use simple dashes ('-') or numbers followed by a space, and ensure each item is on a new line. Avoid any characters that might be interpreted as Markdown by Telegram if possible, but prioritize clarity in plain text.",
                agent_md,
                user_md,
                memories_md
            )
        )
        .model(model.clone())
        .tool(Arc::new(AgentTool::new(Arc::new(investigator))))
        .tool(Arc::new(AgentTool::new(Arc::new(generalist))))
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
    tools.extend(tools::todo::todo_tools());

    // Add tools to the agent builder
    for t in tools {
        builder = builder.tool(t);
    }

    // Load MCP tools from mcp.json if it exists
    builder = mcp::load_mcp_tools(builder).await?;

    // Build and return the agent
    let agent = builder.build()?;

    Ok((Arc::new(agent), model))
}
