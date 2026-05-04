use rustyline::DefaultEditor;
use termimad::{mad_print_inline, MadSkin};
use std::fs::File;
use std::io::Write;


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
    write_file("AGENT.md", "# Agent System Prompt\nDefine your agent's personality and goals here.")?;

    // 4. MEMORIES.md
    write_file("MEMORIES.md", "# Agent Memories\nThis file stores long-term context and past interactions.")?;

    // 5. USER.md
    write_file("USER.md", "# User Profile\nInformation about the user the agent should remember.")?;

    mad_print_inline!(&skin, "\n**Success!** Files initialized: `config.toml`, `.env`, `AGENT.md`, `MEMORIES.md`, `USER.md` \n");

    Ok(())
}

fn write_file(name: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}