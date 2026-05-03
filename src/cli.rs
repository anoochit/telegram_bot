use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use crossterm::style::Stylize;
use crossterm::{cursor, execute, style, terminal};
use futures::StreamExt;
use futures_util::FutureExt;
use rustyline::DefaultEditor;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use termimad::MadSkin;

use crate::agent::get_compaction_config;
use adk_rust::Agent;
use adk_rust::prelude::*;
use adk_session::{CreateRequest, GetRequest, SessionService};

pub(crate) async fn run_cli(
    agent: Arc<dyn Agent>,
    sessions: Arc<dyn SessionService>,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    println!(
        "{}",
        style::style(
            r#"
 _____  ___        __       ___      ___   __     
(\"   \|"  \      /""\     |"  \    /"  | |" \    
|.\\   \    |    /    \     \   \  //   | ||  |   
|: \.   \\  |   /' /\  \    /\\  \/.    | |:  |   
|.  \    \. |  //  __'  \  |: \.        | |.  |   
|    \    \ | /   /  \\  \ |.  \    /:  | /\  |\  
 \___|\____\)(___/    \___)|___|\__/|___|(__\_|_) 
                                                                                               
"#
        )
        .magenta()
    );
    println!("{}", style::style("Nami CLI v0.2.0").bold().magenta());
    println!(
        "\n{}",
        "Type /exit to quit, /clear to wipe terminal, /new to start a new chat."
    );
    println!("Press ESC during a request to cancel it.\n");

    let app_name = "cli";
    let user_id = "default_user";
    let session_id = "cli_session";

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

    let runner = Runner::builder()
        .app_name(app_name)
        .agent(agent)
        .session_service(sessions.clone())
        .compaction_config(get_compaction_config(model))
        .build()?;

    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(".cli_history");

    let mut nami_skin = MadSkin::default();
    nami_skin
        .paragraph
        .set_fg(termimad::crossterm::style::Color::White);
    nami_skin
        .bullet
        .set_fg(termimad::crossterm::style::Color::Magenta);

    loop {
        let readline = rl.readline("You > ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "/exit" {
                    break;
                }
                if trimmed == "/clear" {
                    execute!(
                        stdout,
                        terminal::Clear(terminal::ClearType::All),
                        cursor::MoveTo(0, 0)
                    )?;
                    continue;
                }
                if trimmed == "/new" {
                    let _ = sessions
                        .delete(adk_session::DeleteRequest {
                            app_name: app_name.to_string(),
                            user_id: user_id.to_string(),
                            session_id: session_id.to_string(),
                        })
                        .await;
                    sessions
                        .create(CreateRequest {
                            app_name: app_name.to_string(),
                            user_id: user_id.to_string(),
                            session_id: Some(session_id.to_string()),
                            state: Default::default(),
                        })
                        .await?;
                    println!("\n{}\n", style::style("--- Session reset ---").dim());
                    continue;
                }

                let _ = rl.add_history_entry(trimmed);
                rl.save_history(".cli_history")?;

                let mut response_buffer = String::new();
                let is_thinking = Arc::new(AtomicBool::new(true));
                let indicator = is_thinking.clone();
                let handle = tokio::spawn(async move {
                    let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                    let mut i = 0;
                    while indicator.load(Ordering::Relaxed) {
                        print!(
                            "\r{} Thinking...",
                            style::style(spinner[i % 10]).with(style::Color::Magenta)
                        );
                        io::stdout().flush().ok();
                        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
                        i += 1;
                    }
                });

                let content = Content::new("user").with_text(trimmed);
                let mut stream = runner.run_str(user_id, session_id, content).await?;

                let mut reader = EventStream::new();
                let mut cancelled = false;

                terminal::enable_raw_mode()?;

                loop {
                    tokio::select! {
                        result = stream.next() => {
                            match result {
                                Some(Ok(event)) => {
                                    if let Some(content) = &event.llm_response.content {
                                        for part in &content.parts {
                                            if let Some(text) = part.text() {
                                                response_buffer.push_str(text);
                                            }

                                            if let Part::FunctionCall { name, .. } = part {
                                                print!("\r\x1B[K{} {}\r\n", style::style(" 🛠️ Calling:").dim(), style::style(name).cyan().bold());
                                                io::stdout().flush().ok();
                                            }
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    log::error!("Stream error: {:?}", e);
                                    break;
                                }
                                None => break,
                            }
                        }
                        maybe_event = reader.next().fuse() => {
                            if let Some(Ok(Event::Key(key))) = maybe_event {
                                if key.code == KeyCode::Esc && key.kind == KeyEventKind::Press {
                                    cancelled = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                terminal::disable_raw_mode()?;
                is_thinking.store(false, Ordering::Relaxed);
                handle.await?;
                print!("\r\x1B[K");
                io::stdout().flush().ok();

                if cancelled {
                    println!("\n{}", style::style("--- Request cancelled ---").dim());
                } else {
                    println!("\n{}", style::style("Nami").bold().magenta());
                    nami_skin.print_text(&response_buffer);
                }
                println!();
            }
            Err(_) => break,
        }
    }
    Ok(())
}
