use futures::StreamExt;
use std::sync::Arc;

use adk_runner::EventsCompactionConfig;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

pub struct AgentRunner {
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    app_name: String,
    model: Arc<dyn Llm>,
}

impl AgentRunner {
    pub fn new(
        agent: Arc<dyn Agent>,
        sessions: Arc<dyn SessionService>,
        app_name: impl Into<String>,
        model: Arc<dyn Llm>,
    ) -> Self {
        Self {
            agent,
            sessions,
            app_name: app_name.into(),
            model,
        }
    }

    pub async fn run(
        &self,
        user_id: &str,
        session_id: &str,
        input: &str,
    ) -> anyhow::Result<String> {
        // ensure session exists
        if self
            .sessions
            .get(GetRequest {
                app_name: self.app_name.clone(),
                user_id: user_id.to_string(),
                session_id: session_id.to_string(),
                num_recent_events: None,
                after: None,
            })
            .await
            .is_err()
        {
            self.sessions
                .create(CreateRequest {
                    app_name: self.app_name.clone(),
                    user_id: user_id.to_string(),
                    session_id: Some(session_id.to_string()),
                    state: Default::default(),
                })
                .await?;
        }

        let summarizer = Arc::new(LlmEventSummarizer::new(self.model.clone()));
        let compaction_config = EventsCompactionConfig {
            compaction_interval: 10,
            overlap_size: 2,
            summarizer,
        };

        let runner = Runner::builder()
            .app_name(&self.app_name)
            .agent(self.agent.clone())
            .session_service(self.sessions.clone())
            .compaction_config(compaction_config)
            .build()?;

        let content = Content::new("user").with_text(input);

        let mut stream = runner.run_str(user_id, session_id, content).await?;

        let mut response = String::new();

        while let Some(result) = stream.next().await {
            let event = result?;

            if let Some(content) = &event.llm_response.content {
                for part in &content.parts {
                    if let Some(text) = part.text() {
                        response.push_str(text);
                    }
                }
            }
        }

        if response.is_empty() {
            response = "Sorry, I couldn't generate a response.".to_string();
        }

        Ok(response)
    }
}
