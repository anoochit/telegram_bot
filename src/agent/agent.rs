use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use adk_rust::tool::AgentTool;
use std::sync::Arc;
use std::collections::HashMap;

use crate::utils::get_workspace_dir;
use crate::tools;
use super::mcp;

// OpenAI-compatible API
// use adk_rust::model::{OpenAIClient, OpenAIConfig};


pub fn get_compaction_config(model: Arc<dyn Llm>) -> EventsCompactionConfig {
    EventsCompactionConfig {
        compaction_interval: 10,
        overlap_size: 1,
        summarizer: Arc::new(LlmEventSummarizer::new(model)),
    }
}

pub async fn build_agent() -> anyhow::Result<(Arc<dyn Agent>, Arc<dyn Llm>)> {
    // Load Persona, User Info, and Memories from files
    let agent_md = tokio::fs::read_to_string("AGENT.md")
        .await
        .unwrap_or_else(|_| "Standard Assistant".to_string());
    let user_md = tokio::fs::read_to_string("USER.md")
        .await
        .unwrap_or_else(|_| "Developer".to_string());
    let memories_md = tokio::fs::read_to_string("MEMORIES.md")
        .await
        .unwrap_or_else(|_| "No previous memories.".to_string());

    let api_key =
        std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY environment variable not set");
    let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.5-flash")?);

    // Get the current project root path
    let workspace_dir = get_workspace_dir().await?;

    // --- TOOL GROUPS ---
    let fs_tools = tools::filesystem::filesystem_tools();
    let shell_tools = tools::shell::shell_tools();
    let search_tools = tools::search::search_tools();
    let web_tools = tools::web_fetch::web_fetch_tools();
    let wiki_tools = tools::wiki::wiki_tools();
    let utility_tools: Vec<Arc<dyn Tool>> = {
        let mut t = tools::weather::weather_tools();
        t.extend(tools::current_datetime::datetime_tools());
        t.extend(tools::system_info::system_info_tools());
        t.extend(tools::soul::soul_tools());
        t.extend(tools::todo::todo_tools());
        t
    };

    // --- SUB-AGENTS ---

    let investigator = {
        let mut builder = LlmAgentBuilder::new("codebase_investigator")
            .description("Deep codebase analysis, mapping, and dependency tracking.")
            .instruction("You are a codebase investigator. Analyze context/code to identify root causes or plan refactors. MANDATE: Return a concise 'Summary + Actionable Conclusion' only. Avoid verbose step-by-step logs.")
            .model(model.clone());
        for t in &fs_tools { builder = builder.tool(t.clone()); }
        for t in &shell_tools { builder = builder.tool(t.clone()); }
        for t in &search_tools { builder = builder.tool(t.clone()); }
        Arc::new(builder.build()?)
    };

    let generalist = {
        let mut builder = LlmAgentBuilder::new("generalist")
            .description("High-efficiency agent for batch tasks or general utility.")
            .instruction("You are a generalist agent. Perform requested tasks efficiently using your utility tools. MANDATE: Return a concise 'Summary + Result' only.")
            .model(model.clone());
        for t in &utility_tools { builder = builder.tool(t.clone()); }
        for t in &wiki_tools { builder = builder.tool(t.clone()); }
        for t in &search_tools { builder = builder.tool(t.clone()); }
        Arc::new(builder.build()?)
    };

    let web_developer = {
        let mut builder = LlmAgentBuilder::new("web_developer")
            .description("Full-stack web development: components, APIs, and styling.")
            .instruction("You are a web development expert. Focus on modular, accessible code. MANDATE: Return a concise 'Summary + Code/Implementation' only.")
            .model(model.clone());
        for t in &fs_tools { builder = builder.tool(t.clone()); }
        for t in &web_tools { builder = builder.tool(t.clone()); }
        for t in &search_tools { builder = builder.tool(t.clone()); }
        Arc::new(builder.build()?)
    };

    let devops_engineer = {
        let mut builder = LlmAgentBuilder::new("devops_engineer")
            .description("DevOps, CI/CD, and cloud infrastructure.")
            .instruction("You are a DevOps engineer. Prioritize security and reliability. MANDATE: Return a concise 'Summary + Configuration/Plan' only.")
            .model(model.clone());
        for t in &shell_tools { builder = builder.tool(t.clone()); }
        for t in &fs_tools { builder = builder.tool(t.clone()); }
        Arc::new(builder.build()?)
    };

    let quality_assurance = {
        let mut builder = LlmAgentBuilder::new("quality_assurance")
            .description("Testing, quality control, and edge case analysis.")
            .instruction("You are a QA specialist. Ensure code quality through testing. MANDATE: Return a concise 'Summary + Test Results/Observations' only.")
            .model(model.clone());
        for t in &shell_tools { builder = builder.tool(t.clone()); }
        for t in &fs_tools { builder = builder.tool(t.clone()); }
        Arc::new(builder.build()?)
    };

    let data_specialist = {
        let mut builder = LlmAgentBuilder::new("data_specialist")
            .description("Database design, data models, and analytics.")
            .instruction("You are a data specialist. Focus on normalization and integrity. MANDATE: Return a concise 'Summary + Schema/Query' only.")
            .model(model.clone());
        for t in &fs_tools { builder = builder.tool(t.clone()); }
        Arc::new(builder.build()?)
    };

    let documentation_architect = {
        let mut builder = LlmAgentBuilder::new("documentation_architect")
            .description("Technical documentation and project knowledge bases.")
            .instruction("You are a documentation expert. Ensure clarity and accuracy. MANDATE: Return a concise 'Summary + Documentation' only.")
            .model(model.clone());
        for t in &fs_tools { builder = builder.tool(t.clone()); }
        for t in &wiki_tools { builder = builder.tool(t.clone()); }
        Arc::new(builder.build()?)
    };

    // Wrap sub-agents as tools for the Orchestrator
    let investigator_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(investigator));
    let generalist_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(generalist));
    let web_developer_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(web_developer));
    let devops_engineer_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(devops_engineer));
    let quality_assurance_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(quality_assurance));
    let data_specialist_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(data_specialist));
    let documentation_architect_tool: Arc<dyn Tool> = Arc::new(AgentTool::new(documentation_architect));

    let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
    specialists.insert("codebase_investigator".to_string(), investigator_tool.clone());
    specialists.insert("generalist".to_string(), generalist_tool.clone());
    specialists.insert("web_developer".to_string(), web_developer_tool.clone());
    specialists.insert("devops_engineer".to_string(), devops_engineer_tool.clone());
    specialists.insert("quality_assurance".to_string(), quality_assurance_tool.clone());
    specialists.insert("data_specialist".to_string(), data_specialist_tool.clone());
    specialists.insert("documentation_architect".to_string(), documentation_architect_tool.clone());

    // --- MAIN ORCHESTRATOR ---
    let mut builder = LlmAgentBuilder::new("nami")
        .description("Nami: Orchestrator and Helpful AI Assistant")
        .instruction(
            format!(
                "You are Nami, a high-level Orchestrator. 

# CONTEXT
- **Persona:** {}
- **User:** {}
- **Memories:** {}

# ORCHESTRATION STRATEGY
1. **Delegate by Default:** You are the brain, the sub-agents are the hands. Always delegate specialized tasks (coding, research, analysis) to the appropriate sub-agent.
2. **Minimalist Execution:** Do NOT perform technical work yourself if a sub-agent can do it. Use `parallel_writer` for multiple tasks.
3. **Communication:** Always respond in natural, lively **Thai**.
4. **Token Efficiency:** Be extremely concise. Plain text only. No Markdown.

# SUB-AGENTS
- `codebase_investigator`: Use for deep code analysis/bug hunting.
- `generalist`: Use for general utility (weather, wiki, date, system info, simple tasks).
- `web_developer`: Use for all web-related tasks.
- `devops_engineer`: Use for CI/CD, Docker, and infrastructure.
- `quality_assurance`: Use for testing and validation.
- `data_specialist`: Use for database and data modeling.
- `documentation_architect`: Use for keeping documentation updated.",
                agent_md,
                user_md,
                memories_md
            )
        )
        .model(model.clone())
        .tool(investigator_tool)
        .tool(generalist_tool)
        .tool(web_developer_tool)
        .tool(devops_engineer_tool)
        .tool(quality_assurance_tool)
        .tool(data_specialist_tool)
        .tool(documentation_architect_tool);

    for t in tools::parallel_writer::parallel_writer_tool(specialists) {
        builder = builder.tool(t);
    }

    builder = builder.with_skills_from_root(workspace_dir)?;

    // Load MCP tools for the orchestrator (can be further delegated if needed)
    builder = mcp::load_mcp_tools(builder).await?;

    let agent = builder.build()?;
    Ok((Arc::new(agent), model))
}
