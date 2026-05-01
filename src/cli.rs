use futures::StreamExt;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use termimad::MadSkin;
use rustyline::{DefaultEditor};

use adk_runner::EventsCompactionConfig;
use adk_rust::Agent;
use adk_rust::agent::LlmEventSummarizer;
use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

pub(crate) async fn run_cli(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    println!(
        r#"
Type a message to chat. /exit to quit.
"#
    );

    let app_name = "cli";
    let user_id = "default_user";
    let session_id = "cli_session";

    // ensure session exists
    if sessions
        .get(GetRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            num_recent_events: None,
            after: None,
        })
        .await
        .is_err()
    {
        sessions
            .create(CreateRequest {
                app_name: app_name.to_string(),
                user_id: user_id.to_string(),
                session_id: Some(session_id.to_string()),
                state: Default::default(),
            })
            .await?;
    }

    let summarizer = Arc::new(LlmEventSummarizer::new(model.clone()));
    let compaction_config = EventsCompactionConfig {
        compaction_interval: 10,
        overlap_size: 2,
        summarizer,
    };

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions.clone())
        .compaction_config(compaction_config)
        .build()?;

    let mut rl = DefaultEditor::new()?;
    // Optional: persist history
    let _ = rl.load_history(".cli_history");

    let mut response_buffer = String::new();
    let skin = MadSkin::default();
    loop {
        let readline = rl.readline("You> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "/exit" || trimmed == "exit" {
                    break;
                }
                if trimmed == "/clean" {
                    print!("\x1B[2J\x1B[1;1H");
                    io::stdout().flush().ok();
                    continue;
                }
                if trimmed == "/clear" {
                    let _ = sessions.delete(adk_session::DeleteRequest {
                        app_name: app_name.to_string(),
                        user_id: user_id.to_string(),
                        session_id: session_id.to_string(),
                    }).await;
                    let _ = sessions.create(CreateRequest {
                        app_name: app_name.to_string(),
                        user_id: user_id.to_string(),
                        session_id: Some(session_id.to_string()),
                        state: Default::default(),
                    }).await;
                    println!("Session cleared.");
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);
                let _ = rl.save_history(".cli_history");

                let content = Content::new("user").with_text(trimmed);
                let mut stream = runner.run_str(user_id, session_id, content).await?;

                response_buffer.clear();
                print!("Agent> ");
                
                // Thinking indicator
                let is_thinking = Arc::new(AtomicBool::new(true));
                let indicator = is_thinking.clone();
                let handle = tokio::spawn(async move {
                    let spinner = ['|', '/', '-', '\\'];
                    let mut i = 0;
                    while indicator.load(Ordering::Relaxed) {
                        print!("\rAgent> [{}]", spinner[i % 4]);
                        io::stdout().flush().ok();
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        i += 1;
                    }
                    // Clear the line
                    print!("\r\x1B[K");
                    io::stdout().flush().ok();
                });

                while let Some(result) = stream.next().await {
                    let event = result?;

                    if let Some(content) = &event.llm_response.content {
                        for part in &content.parts {
                            if let Some(text) = part.text() {
                                response_buffer.push_str(text);
                            }
                        }
                    }
                }
                is_thinking.store(false, Ordering::Relaxed);
                handle.await?;
                println!();
                skin.print_text(&response_buffer);
                println!();
            }
            Err(rustyline::error::ReadlineError::Interrupted) | Err(rustyline::error::ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
