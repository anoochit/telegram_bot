use crate::agent::get_compaction_config;
use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::SessionService;
use futures::StreamExt;
use std::sync::Arc;

pub(crate) async fn run_direct(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
    prompt: &str,
) -> anyhow::Result<()> {
    let app_name = "cli";
    let user_id = "default_user";
    let session_id = "cli_session";

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions.clone())
        .compaction_config(get_compaction_config(model))
        .build()?;

    let content = Content::new("user").with_text(prompt);
    let mut stream = runner.run_str(user_id, session_id, content).await?;

    let mut response_buffer = String::new();
    while let Some(result) = stream.next().await {
        if let Ok(event) = result
            && let Some(content) = &event.llm_response.content
        {
            for part in &content.parts {
                if let Some(text) = part.text() {
                    response_buffer.push_str(text);
                }
            }
        }
    }

    termimad::print_text(&response_buffer);
    Ok(())
}
