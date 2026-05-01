use futures::StreamExt;
use rustyline::DefaultEditor;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use termimad::MadSkin;
use crossterm::{execute, terminal, cursor, style};
use crossterm::style::Stylize;

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
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;

    println!(
        r#"
      _   _                _ 
     | \ | |              (_)
     |  \| | __ _ _ __ ___ _ 
     | . ` |/ _` | '_ ` _ \ |
     | |\  | (_| | | | | | | |
     |_| \_|\__,_|_| |_| |_|_|
     
     Nami CLI :: Your Playful Agent
     Type /exit, /clear, or /new.
"#
    );

    let app_name = "cli";
    let user_id = "default_user";
    let session_id = "cli_session";

    if sessions.get(GetRequest {
        app_name: app_name.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        num_recent_events: None,
        after: None,
    }).await.is_err() {
        sessions.create(CreateRequest {
            app_name: app_name.to_string(),
            user_id: user_id.to_string(),
            session_id: Some(session_id.to_string()),
            state: Default::default(),
        }).await?;
    }

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions.clone())
        .compaction_config(EventsCompactionConfig {
            compaction_interval: 10,
            overlap_size: 2,
            summarizer: Arc::new(LlmEventSummarizer::new(model.clone())),
        })
        .build()?;

    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(".cli_history");

    let mut nami_skin = MadSkin::default();
    nami_skin.bold.set_fg(termimad::crossterm::style::Color::Magenta);
    // Remove the paragraph color override, as it can sometimes affect how `termimad` renders complex block elements like lists.
    // nami_skin.paragraph.set_fg(termimad::crossterm::style::Color::White);

    loop {
        let readline = rl.readline("You> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if trimmed == "/exit" { break; }
                if trimmed == "/clear" {
                    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;
                    continue;
                }
                if trimmed == "/new" {
                    let _ = sessions.delete(adk_session::DeleteRequest {
                        app_name: app_name.to_string(),
                        user_id: user_id.to_string(),
                        session_id: session_id.to_string(),
                    }).await;
                    sessions.create(CreateRequest {
                        app_name: app_name.to_string(),
                        user_id: user_id.to_string(),
                        session_id: Some(session_id.to_string()),
                        state: Default::default(),
                    }).await?;
                    println!("--- New Session Started ---");
                    continue;
                }

                let _ = rl.add_history_entry(trimmed);
                rl.save_history(".cli_history")?;

                println!("\n{}", style::style("─".repeat(50)).with(style::Color::DarkGrey));
                
                let mut response_buffer = String::new();
                let is_thinking = Arc::new(AtomicBool::new(true));
                let indicator = is_thinking.clone();
                let handle = tokio::spawn(async move {
                    let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                    let mut i = 0;
                    while indicator.load(Ordering::Relaxed) {
                        print!("\r{} Nami is thinking... {}", style::style(spinner[i % 10]).with(style::Color::Magenta), spinner[i % 10]);
                        io::stdout().flush().ok();
                        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
                        i += 1;
                    }
                    print!("\r\x1B[K");
                    io::stdout().flush().ok();
                });

                let content = Content::new("user").with_text(trimmed);
                let mut stream = runner.run_str(user_id, session_id, content).await?;

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(event) => {
                            if let Some(content) = &event.llm_response.content {
                                for part in &content.parts {
                                    if let Some(text) = part.text() { 
                                        response_buffer.push_str(text); 
                                    }
                                }
                            }
                        }
                        Err(e) => log::error!("Stream error: {:?}", e),
                    }
                }
                is_thinking.store(false, Ordering::Relaxed);
                handle.await?;

                // Apply formatting to the full buffer
                let mut formatted = response_buffer
                    // Normalize: Ensure space after markers
                    .replace("-", "\n- ")
                    .replace("*", "\n* ");
                
                for i in 0..=9 {
                    let pattern = format!("{}.", i);
                    // Replace with newline + "i. " and ensure exactly one space
                    formatted = formatted.replace(&pattern, &format!("\n{}. ", i));
                }

                // Cleanup: Remove excessive newlines that might have been created
                let formatted = formatted.split('\n')
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");

                print!("\n");
                print!("Nami> ");
                nami_skin.print_text(&formatted);
                println!("\n{}", style::style("─".repeat(50)).with(style::Color::DarkGrey));
            }
            Err(_) => break,
        }
    }
    Ok(())
}
