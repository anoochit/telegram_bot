use adk_rust::prelude::*;
use adk_rust::tool::AgentTool;
use std::collections::HashMap;
use std::sync::Arc;

use crate::tools;

pub fn get_specialists(model: Arc<dyn Llm>) -> anyhow::Result<HashMap<String, Arc<dyn Tool>>> {
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

    let generalist = {
        let mut builder = LlmAgentBuilder::new("generalist")
            .description("Versatile problem-solver for all tasks including engineering, ops, and general utility.")
            .instruction(
                "You are a versatile Generalist Agent. You excel at multi-disciplinary problem solving, code development, system operations, and information synthesis. You have access to a full suite of tools for filesystem management, shell execution, web interaction, search, and general utilities. Prioritize accuracy, security, and speed. MANDATE: Provide a concise 'Summary + Result' only."
            )
            .model(model.clone());
        for t in &fs_tools {
            builder = builder.tool(t.clone());
        }
        for t in &shell_tools {
            builder = builder.tool(t.clone());
        }
        for t in &search_tools {
            builder = builder.tool(t.clone());
        }
        for t in &web_tools {
            builder = builder.tool(t.clone());
        }
        for t in &wiki_tools {
            builder = builder.tool(t.clone());
        }
        for t in &utility_tools {
            builder = builder.tool(t.clone());
        }
        Arc::new(builder.build()?)
    };

    let mut specialists: HashMap<String, Arc<dyn Tool>> = HashMap::new();
    specialists.insert("generalist".to_string(), Arc::new(AgentTool::new(generalist)));

    Ok(specialists)
}
