use adk_session::SqliteSessionService;
use std::fs;
use std::path::Path;

pub async fn initialize_project() -> anyhow::Result<()> {
    let files = [
        (
            "AGENT.md",
            "# Agent Configuration\n\nDefine agent personality and capabilities here.",
        ),
        (
            "MEMORIES.md",
            "# Agent Memories\n\nPersistent memories and context for the agent.",
        ),
        (
            "USER.md",
            "# User Information\n\nUser profile and preferences.",
        ),
        (
            "config.toml",
            "[model]\nprovider = \"gemini\"\nmodel_name = \"gemini-2.5-flash\"\napi_key_env = \"GOOGLE_API_KEY\"\n",
        ),
    ];

    for (filename, content) in files {
        if !Path::new(filename).exists() {
            fs::write(filename, content)?;
            println!("Created {}", filename);
        } else {
            println!("{} already exists, skipping.", filename);
        }
    }

    let db_path = "sessions.db";
    println!("Initializing database at {}...", db_path);
    let sessions = SqliteSessionService::new(&format!("{}?mode=rwc", db_path)).await?;
    sessions.migrate().await?;
    println!("Database initialized successfully.");

    Ok(())
}
