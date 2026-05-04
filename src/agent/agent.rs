use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use anyhow::Context;
use serde::Deserialize;
use std::sync::Arc;

use super::mcp;
use crate::tools;
use crate::utils::get_workspace_dir;

// Providers
use adk_rust::model::{ OpenAIClient, OpenAIConfig };
use adk_rust::model::{ OpenRouterClient, OpenRouterConfig };

use adk_rust::tool::AgentTool;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct AppConfig {
    model: ModelConfig,
}

#[derive(Debug, Deserialize)]
struct ModelConfig {
    provider: String,
    model_name: String,
    api_key_env: String,
    #[allow(dead_code)]
    base_url: Option<String>,
}

async fn load_config() -> anyhow::Result<AppConfig> {
    let config_str = tokio::fs::read_to_string("config.toml").await?;
    let config: AppConfig = toml::from_str(&config_str)?;
    Ok(config)
}

pub fn get_compaction_config(model: Arc<dyn Llm>) -> EventsCompactionConfig {
    EventsCompactionConfig {
        compaction_interval: 5,
        overlap_size: 1,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>, String, String)> {
    // Load config
    let app_config = load_config().await.unwrap_or_else(|e| {
        log::warn!("Failed to load config.toml: {}. Using defaults.", e);
        AppConfig {
            model: ModelConfig {
                provider: "gemini".to_string(),
                model_name: "gemini-2.5-flash".to_string(),
                api_key_env: "GOOGLE_API_KEY".to_string(),
                base_url: None,
            },
        }
    });

    let provider = app_config.model.provider.clone();
    let model_name = app_config.model.model_name.clone();

    // Initialize Model based on config
    let api_key = std::env
        ::var(&app_config.model.api_key_env)
        .with_context(|| {
            format!("Environment variable {} not set", app_config.model.api_key_env)
        })?;

    let model: Arc<dyn Llm> = match app_config.model.provider.as_str() {
        "gemini" => Arc::new(GeminiModel::new(&api_key, &app_config.model.model_name)?),
        "openrouter" =>
            Arc::new(
                OpenRouterClient::new(
                    OpenRouterConfig::new(&api_key, &app_config.model.model_name)
                )?
            ),
        "openai" =>
            Arc::new(OpenAIClient::new(OpenAIConfig::new(&api_key, &app_config.model.model_name))?),
        "thaillm" =>
            Arc::new(
                OpenAIClient::new(
                    OpenAIConfig::compatible(
                        &api_key,
                        "https://thaillm.or.th/api/v1",
                        &app_config.model.model_name
                    )
                )?
            ),

        _ => anyhow::bail!("Unsupported provider: {}", app_config.model.provider),
    };

    // Load Persona, User Info, and Memories from files
    let agent_md = tokio::fs
        ::read_to_string("AGENT.md").await
        .unwrap_or_else(|_| "Standard Assistant".to_string());
    let user_md = tokio::fs
        ::read_to_string("USER.md").await
        .unwrap_or_else(|_| "Developer".to_string());
    let memories_md = tokio::fs
        ::read_to_string("MEMORIES.md").await
        .unwrap_or_else(|_| "No previous memories.".to_string());

    // Get the current project root path
    let workspace_dir = get_workspace_dir().await?;

    // --- SUB-AGENTS ---
    let generalist = Arc::new(
        LlmAgentBuilder::new("generalist")
            .description(
                "A high-efficiency agent with access to all tools. Use this for repetitive batch tasks or high-volume data processing to keep the main conversation history lean."
            )
            .instruction(
                "You are a generalist agent. Perform the requested batch tasks or data processing efficiently."
            )
            .model(model.clone())
            .build()?
    );

    // Wrap sub-agents as tools
    let generalist_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(generalist));

    // Create a map of specialized tools for the ParallelWriter
    let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
    specialists.insert("generalist".to_string(), generalist_tool.clone());

    // --- MAIN ORCHESTRATOR ---
    let mut builder = LlmAgentBuilder::new("nami")
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
0. Concise Communication: Be direct. Do NOT repeat the current task or latest prompt at the start of your response unless the task status has changed or you are explicitly asked to summarize the current state.
1. Skill-First Approach: Always prioritize using your specialized **Skills** (from the `.skills/` directory) to perform complex tasks or follow specific workflows. Skills represent your high-level expertise and standardized procedures.
2. Tool Usage: If no specialized skill is applicable, use your built-in **Tools** (google_search, web_fetch, Wiki, FileSystem, Weather, Shell, etc.) to perform actions, retrieve data, or verify information.
3. Delegation: You have specialized sub-agents at your disposal. Use them for complex or turn-intensive tasks:
   - Use 'generalist' for repetitive batch tasks.
   - Use 'parallel_task' to execute multiple tasks or specialist queries simultaneously.
3. Knowledge Management (Wiki): Use the Wiki tools to store and retrieve long-term information. Treat the 'wiki/' directory as your primary memory.
   - To learn/save: Use add_wiki_page.
   - To find: Use search_wiki or list_wiki_pages. Use search_wiki_by_tag for finding specific tags.
   - To read: Use get_wiki_page.
   - To organize: Use summarize_wiki to update the summary index.
   - To explore connections: Use get_wiki_graph to visualize the vault or get_backlinks to see what links to a specific page.
   - To maintain: Use check_broken_links to find dangling wikilinks, and rename_wiki_page to safely rename a page and update all its incoming links.
   - To create structured notes: Use apply_template to apply predefined structures to new or existing pages.
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
        .model(model.clone());

    // Register all specialists as tools for the orchestrator
    for specialist in specialists.values() {
        builder = builder.tool(specialist.clone());
    }

    for t in tools::parallel_tasks::parallel_tasks_tool(specialists) {
        builder = builder.tool(t);
    }

    // add tools to the agent
    let mut tools: Vec<Arc<dyn Tool>> = tools::weather::weather_tools();
    tools.extend(tools::filesystem::filesystem_tools());
    tools.extend(tools::current_datetime::datetime_tools());
    tools.extend(tools::wiki::wiki_tools());
    tools.extend(tools::shell::shell_tools());
    tools.extend(tools::web_fetch::web_fetch_tools());
    tools.extend(tools::system_status::system_status_tools());
    tools.extend(tools::soul::soul_tools());
    tools.extend(tools::search::search_tools());
    tools.extend(tools::todo::todo_tools());
    tools.extend(tools::parallel_tasks::parallel_tasks_tool(specialists));

    // Add tools to the agent builder
    for t in tools {
        builder = builder.tool(t);
    }

    builder = builder.with_skills_from_root(workspace_dir)?;

    // Load MCP tools for the orchestrator (can be further delegated if needed)
    builder = mcp::load_mcp_tools(builder).await?;

    let agent = builder.build()?;

    Ok((Arc::new(agent), model, provider, model_name))
}
