use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use anyhow::Context;
use serde::Deserialize;
use std::sync::Arc;

use super::mcp;
use super::specialists;
use crate::tools;
use crate::utils::get_workspace_dir;

// Providers
use adk_rust::model::{OpenAIClient, OpenAIConfig};

/// Application configuration structure loaded from `config.toml`.
#[derive(Debug, Deserialize)]
struct AppConfig {
    model: ModelConfig,
}

/// Configuration details for the LLM provider and specific model.
#[derive(Debug, Deserialize)]
struct ModelConfig {
    provider: String,
    model_name: String,
    api_key_env: String,
    #[allow(dead_code)]
    base_url: Option<String>,
}

/// Attempts to load the application configuration from `config.toml`.
async fn load_config() -> anyhow::Result<AppConfig> {
    let config_str = tokio::fs::read_to_string("config.toml").await?;
    let config: AppConfig = toml::from_str(&config_str)?;
    Ok(config)
}

/// Generates the compaction configuration for managing agent history events.
pub fn get_compaction_config(model: Arc<dyn Llm>) -> EventsCompactionConfig {
    EventsCompactionConfig {
        compaction_interval: 5,
        overlap_size: 1,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

/// Orchestrates the building of the main AI agent, loading configuration, persona context,
/// and setting up tools, skills, and MCP servers.
///
/// Returns a tuple containing the built agent, the model instance, the provider name,
/// and the model name.
pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>, String, String)> {
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

    let (provider, model_name) = (
        app_config.model.provider.clone(),
        app_config.model.model_name.clone(),
    );
    let model = load_model(&app_config.model).await?;
    let context = load_persona_context().await?;
    let workspace_dir = get_workspace_dir().await?;

    let specialists = specialists::get_specialists(model.clone());
    let mut builder = LlmAgentBuilder::new("nami")
        .description("A helpful and playful AI assistant")
        .instruction(format_persona(&context.0, &context.1, &context.2))
        .model(model.clone());

    builder = configure_agent_tools(builder, specialists);
    builder = builder.with_skills_from_root(workspace_dir)?;
    builder = mcp::load_mcp_tools(builder).await?;

    let agent = builder.build()?;
    Ok((Arc::new(agent), model, provider, model_name))
}

async fn load_model(model_config: &ModelConfig) -> anyhow::Result<Arc<dyn Llm>> {
    let api_key = std::env::var(&model_config.api_key_env)
        .with_context(|| format!("Environment variable {} not set", model_config.api_key_env))?;

    match model_config.provider.as_str() {
        "gemini" => Ok(Arc::new(GeminiModel::new(&api_key, &model_config.model_name)?)),
        "openrouter" => Ok(Arc::new(OpenRouterClient::new(OpenRouterConfig::new(&api_key, &model_config.model_name))?)),
        "openai" => Ok(Arc::new(OpenAIClient::new(OpenAIConfig::new(&api_key, &model_config.model_name))?)),
        "thaillm" => Ok(Arc::new(OpenAIClient::new(OpenAIConfig::compatible(&api_key, "https://thaillm.or.th/api/v1", &model_config.model_name))?)),
        _ => anyhow::bail!("Unsupported provider: {}", model_config.provider),
    }
}

async fn load_persona_context() -> anyhow::Result<(String, String, String)> {
    let agent_md = tokio::fs::read_to_string("AGENT.md")
        .await
        .unwrap_or_else(|_| "Standard Assistant".to_string());
    let user_md = tokio::fs::read_to_string("USER.md")
        .await
        .unwrap_or_else(|_| "Developer".to_string());
    let memories_md = tokio::fs::read_to_string("MEMORIES.md")
        .await
        .unwrap_or_else(|_| "No previous memories.".to_string());
    Ok((agent_md, user_md, memories_md))
}

fn format_persona(agent_md: &str, user_md: &str, memories_md: &str) -> String {
    format!(
        "You are an AI Agent assistant.\n\n# YOUR SOUL (Persona)\n{}\n\n# THE USER (Context)\n{}\n\n# YOUR MEMORIES (Past Facts)\n{}\n\n# GUIDELINES FOR INTERACTION\n0. Concise Communication: Be direct. Do NOT repeat the current task or latest prompt at the start of your response unless the task status has changed or you are explicitly asked to summarize the current state.\n1. Skill-First Approach: Always prioritize using your specialized **Skills** (from the `.skills/` directory) to perform complex tasks or follow specific workflows. Skills represent your high-level expertise and standardized procedures.\n2. Tool Usage: If no specialized skill is applicable, use your built-in **Tools** (google_search, web_fetch, Wiki, FileSystem, Weather, Shell, etc.) to perform actions, retrieve data, or verify information.\n3. Delegation: You have specialized sub-agents at your disposal. Use them for complex or turn-intensive tasks:\n   - Use 'generalist' for repetitive batch tasks.\n   - Use 'parallel_task' to execute multiple tasks or specialist queries simultaneously.\n3. Knowledge Management (Wiki): Use the Wiki tools to store and retrieve long-term information. Treat the 'wiki/' directory as your primary memory.\n   - To learn/save: Use add_wiki_page.\n   - To find: Use search_wiki or list_wiki_pages. Use search_wiki_by_tag for finding specific tags.\n   - To read: Use get_wiki_page.\n   - To organize: Use summarize_wiki to update the summary index.\n   - To explore connections: Use get_wiki_graph to visualize the vault or get_backlinks to see what links to a specific page.\n   - To maintain: Use check_broken_links to find dangling wikilinks, and rename_wiki_page to safely rename a page and update all its incoming links.\n   - To create structured notes: Use apply_template to apply predefined structures to new or existing pages.\n4. Personal Memories: When you learn a personal fact about the user (preferences, habits, secrets), use update_user_memory to save it permanently in MEMORIES.md.\n5. Web Search & Content:\n   - Use google_search if you don't know something or need the latest information.\n   - Use web_fetch to retrieve the full content of a specific URL.\n6. Task Management & Decomposition: Use the TODO tools to manage complex workflows.\n   - Decomposition: For large or multi-step requests, first split the goal into smaller, actionable sub-tasks and add them using add_todo.\n   - Tracking: Keep the TODO list updated. Use list_todos to see what's left.\n   - Execution: You can execute multiple tools in a single response to complete several sub-tasks if appropriate.\n7. Precision & Security: Stay concise and technically accurate. Never disclose sensitive credentials, API keys, or environment secrets.\n8. Transparency: If a request exceeds your capabilities, clearly state your limitations in a friendly way.\n9. Formatting: Do NOT use any Markdown formatting (no bold, italics, headers, or tables). Output responses as plain text only.\n10. Language: You MUST always answer and communicate with the user in Thai. Use natural, lively, and professional Thai as defined in your Persona. Tool names and arguments should remain in English.\n11. Final Output: Use plain text only. For lists, use simple dashes ('-') or numbers followed by a space, and ensure each item is on a new line. Avoid any characters that might be interpreted as Markdown by Telegram if possible, but prioritize clarity in plain text.",
        agent_md, user_md, memories_md
    )
}

fn configure_agent_tools(
    mut builder: LlmAgentBuilder,
    specialists: std::collections::HashMap<String, Arc<dyn Tool>>,
) -> LlmAgentBuilder {
    let mut tools: Vec<Arc<dyn Tool>> = tools::weather::weather_tools();
    tools.extend(tools::filesystem::filesystem_tools());
    tools.extend(tools::current_datetime::datetime_tools());
    tools.extend(tools::wiki::wiki_tools());
    // tools.extend(tools::shell::shell_tools());
    tools.extend(tools::web_fetch::web_fetch_tools());
    tools.extend(tools::system_status::system_status_tools());
    tools.extend(tools::soul::soul_tools());
    tools.extend(tools::search::search_tools());
    tools.extend(tools::todo::todo_tools());
    tools.extend(tools::parallel_tasks::parallel_tasks_tool(specialists));

    for t in tools {
        builder = builder.tool(t);
    }
    builder
}
