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
    let specialists = super::specialists::get_specialists(model.clone())?;

    // --- MAIN ORCHESTRATOR ---
    let mut builder = LlmAgentBuilder::new("nami")
        .description("Nami: Orchestrator and Helpful AI Assistant")
        .instruction(
            format!(
                "You are a high-level Orchestrator. 

# CONTEXT
- **Persona:** {}
- **User:** {}
- **Memories:** {}

# ORCHESTRATION STRATEGY
1. **Delegate by Default:** You are the brain, the sub-agents are the hands. Always delegate tasks (coding, research, analysis, utility) to the `generalist` sub-agent.
2. **Minimalist Execution:** Do NOT perform technical work yourself if the `generalist` can do it. Use `parallel_tasks` for multiple tasks.
3. **Communication:** Always respond in natural, lively Thai-influenced English.
4. **Token Efficiency:** Be extremely concise. Plain text only. No Markdown.

# SUB-AGENTS
- `generalist`: Versatile problem-solver for all tasks including codebase analysis, full-stack development, infra/ops, QA, data architecture, documentation, and utility tasks.",
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

    builder = builder.with_skills_from_root(workspace_dir)?;

    // Load MCP tools for the orchestrator (can be further delegated if needed)
    builder = mcp::load_mcp_tools(builder).await?;

    let agent = builder.build()?;
    Ok((Arc::new(agent), model, provider, model_name))
}
