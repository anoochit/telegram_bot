# README

ADK-Rust doesn't have a built-in Telegram integration, but you can build a Telegram bot on top of it by combining `adk-rust` with the `teloxide` crate (the standard Telegram bot library for Rust). Here's a complete guide:

## Edit `.env`

```text
THAILLM_API_KEY=your-api-key-here
GOOGLE_API_KEY=your-google-api-key-here
TELOXIDE_TOKEN=your_telegram_bot_token
```

Get your bot token from [@BotFather](https://t.me/BotFather) on Telegram

## Run the application

The application supports multiple modes:

### Telegram Bot

```bash
cargo run -- bot
```

### Console CLI

```bash
cargo run -- cli
```

### HTTP Server

```bash
cargo run -- server
```

## How it Works

The architecture is straightforward:

```
Telegram User
     │  (sends message)
     ▼
  teloxide  ──────────────────────────────────▶  Telegram API
     │  (routes update)                              ▲
     ▼                                               │
handle_message()                          bot.send_message()
     │
     ▼
 adk-rust Runner
  (per-user session via SqliteSessionService)
     │
     ▼
  LlmAgent  ──▶  Gemini / Anthropic / OpenAI
```

- **`teloxide`** handles all the Telegram polling and message routing
- **`adk-rust`** handles the AI agent logic and conversation memory
- **`SqliteSessionService`** keyed by `chat_id` gives each user their own persistent conversation history in `sessions.db`

## Workspace Sandbox

The bot includes a filesystem tool that operates within a "workspace" directory. By default, this is located at:

- **`~/workspace`** (in your home directory)

The bot will automatically create this directory if it doesn't exist. Files created by the agent will be stored here.

## Skills System

The bot supports a directory-based skills system. Skills are automatically loaded from the `.skills/` directory in both the **project root** and the **workspace directory**.

- **`greeting`**: Provides warm and professional greetings.
- **`joke-generator`**: Generates appropriate jokes to lighten the mood.
- **`system_info`**: Retrieves machine statistics (CPU, memory, disk).
- **`create-skill`**: Helps the agent create and scaffold new skills.

You can add new skills by creating a new directory in `.skills/` with a `SKILL.md` file in either location.

## Model Context Protocol (MCP) Integration

This bot supports loading external tools via MCP. To use it:

1. Create an `mcp.json` file in the **workspace directory** (recommended) or the **root directory** (see `mcp.json.example`). The workspace version takes precedence.
2. Define your MCP servers in the following format:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/path/to/allowed/dir"
      ]
    }
  }
}
```

3. Restart the bot. It will automatically detect the file, start the servers, and register the tools with the agent.

## Key Tips

**Switch LLM providers** — just change the client configuration in `src/agent/mod.rs`.

**Example: Gemini (Default)**

```rust
let api_key = std::env::var("GOOGLE_API_KEY")?;
let model = Arc::new(GeminiModel::new(&api_key, "gemini-2.5-flash")?);
```

**Example: ThaiLLM (OpenAI-compatible)**

```rust
let config = OpenAIConfig::compatible(
    &api_key,
    "https://thaillm.or.th/api/v1",
    "typhoon-s-thaillm-8b-instruct",
);
let model = OpenAIClient::new(config)?;
```

**Add tools** — attach ADK function tools to your agent in `src/agent/mod.rs`:

```rust
let mut tools = weather_tool::weather_tools();
tools.extend(web_fetch_tool::web_fetch_tools());
// add more tools...

let mut builder = LlmAgentBuilder::new("agent")
    .model(Arc::new(model));

for t in tools {
    builder = builder.tool(t).into();
}
let agent = builder.build()?;
```

**Persistent sessions** — The bot uses `SqliteSessionService` by default so conversations survive restarts. The database is stored in `sessions.db`.

**Webhooks vs polling** — `teloxide` defaults to long polling (easiest for dev). For production, switch to webhooks using `teloxide`'s `axum_no_setup` feature.
