use rustyline::DefaultEditor;
use termimad::{mad_print_inline, MadSkin};
use std::fs::File;
use std::io::Write;
use adk_session::SqliteSessionService;


pub async fn initialize_project() -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;
    let skin = MadSkin::default();

    skin.print_text("# AI Agent Initializer\n");

    // 1. Choose LLM Provider
    skin.print_text("*Choose LLM Provider (gemini, openai, openrouter, thaillm):*");
    let provider = rl.readline(">> ")?.trim().to_lowercase();

    // 2. Choose Model
    skin.print_text("\n*Enter Model Name (e.g., gemini-3-flash-preview):*");
    let model_name = rl.readline(">> ")?.trim().to_string();

    // 3. Enter LLM API Key
    skin.print_text("\n*Enter LLM API Key:*");
    let api_key = rl.readline(">> ")?.trim().to_string();

    // 4. Enter Telegram API Key (Optional)
    skin.print_text("\n*Enter Telegram API Key (Optional, press Enter to skip):*");
    let telegram_key = rl.readline(">> ")?.trim().to_string();

    // Determine Env Var Name based on provider
    let api_key_env = match provider.as_str() {
        "gemini" => "GOOGLE_API_KEY",
        "openai" => "OPENAI_API_KEY",
        "thaillm" => "THAILLM_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        _ => "API_KEY",
    };

    // --- File Generation ---

    // 1. config.toml
    let config_content = format!(
r#"[model]
# Provider type: "gemini", "openai", "openrouter" or "thaillm"
provider = "{provider}"
# The specific model identifier
model_name = "{model_name}"
# The environment variable name that holds the API key
api_key_env = "{api_key_env}"
"#);
    write_file("config.toml", &config_content)?;

    // 2. .env
    let env_content = format!(
r#"{api_key_env}={api_key}
TELOXIDE_TOKEN={telegram_key}
SERPER_API_KEY=your_serper_api_key
"#);
    write_file(".env", &env_content)?;

    // 3. AGENT.md
    write_file("AGENT.md", "# Agent Persona (The Soul)\n\n## Name\n\nNami (นามิ)\n\n## Personality\n\n- Friendly, playful, and energetic.\n- Uses polite but lively.\n- Proactive and helpful, always trying to anticipate what the user needs.\n- Technically sharp but explains things in a simple, fun way.\n\n## Tone of Voice\n\n- High energy, positive, and encouraging.\n- Professional when handling security or system tasks, but warm when chatting.\n- ALWAYS use proper Markdown formatting. When making lists, use newlines between list items to ensure they render correctly.\n- Be concise and direct. Avoid repeating the current task or latest prompt back to the user unless it has changed or you are explicitly asked to summarize the state.\n\n## Evolution\n\nName: Nami\nPersonality: Friendly, playful, energetic, polite, proactive, technically sharp.\nTone: High energy, positive, encouraging, professional for tasks, plain text only.\n\n## Evolution\n\nLanguage: Always answer and communicate in English.")?;

    // 4. MEMORIES.md
    write_file("MEMORIES.md", "# User Memories\n\n- User's name is Noel and lives in Bangkok, Thailand.\n- Noel is the Creator/Developer of this bot.\n- Noel prefers clear, direct technical explanations and proactive project organization.")?;

    // 5. USER.md
    write_file("USER.md", "# User Information\n\n## Identity\n\n- I 'am Noel\n- Live in Bangkok, Thailand.\n- The user is the Creator/Developer of this bot.\n\n## Preferences\n\n- Prefers clear, direct technical explanations.\n- Likes the bot to be proactive about project organization.\n- Uses Thai for daily communication but English for technical terms.\n")?;

    mad_print_inline!(&skin, "\n**Success!** Files initialized: `config.toml`, `.env`, `AGENT.md`, `MEMORIES.md`, `USER.md` \n");

    // 6. Session Management
    let db_path = "sessions.db";
    mad_print_inline!(&skin, "Initializing database at {}...", db_path);
    let sessions = SqliteSessionService::new(&format!("{}?mode=rwc", db_path)).await?;
    sessions.migrate().await?;
    mad_print_inline!(&skin, "Database initialized successfully.");


    Ok(())
}

fn write_file(name: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}