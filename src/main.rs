mod agent;
mod bot;
mod cli;
mod runner;
mod serve;

use std::sync::Arc;

use adk_session::SqliteSessionService;
use clap::{Parser, Subcommand};
use runner::AgentRunner;

#[derive(Parser)]
#[command(name = "agent-app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bot,    // telegram bot
    Cli,    // command line interface
    Server, // http server
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    if !matches!(cli.command, Commands::Server) {
        pretty_env_logger::init();
    }

    log::info!("Application starting...");

    // shared setup
    log::info!("Building agent...");
    let (agent, model) = agent::build_agent().await?;
    log::info!("Agent built successfully.");
    let sessions = SqliteSessionService::new("sessions.db?mode=rwc").await?;
    sessions.migrate().await?;
    let sessions = Arc::new(sessions);

    match cli.command {
        Commands::Bot => {
            log::info!("Running in Bot mode");
            let runner = Arc::new(AgentRunner::new(agent, sessions.clone(), "telegram", model));
            bot::run_bot(runner, sessions.clone()).await?;
        }
        Commands::Cli => {
            cli::run_cli(agent, sessions, model).await?;
        }
        Commands::Server => {
            serve::run_serve(agent).await?;
        }
    }

    Ok(())
}
