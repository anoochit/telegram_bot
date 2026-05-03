use crate::agent::utils::get_workspace_dir;
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use regex::Regex;
use chrono::{Utc, Datelike};

#[derive(Deserialize, JsonSchema)]
struct WikiPageArgs {
    /// The title of the wiki page (e.g., 'project-notes'). This will be used as the filename.
    title: String,
}

#[derive(Deserialize, JsonSchema)]
struct AddWikiArgs {
    /// The title of the wiki page.
    title: String,
    /// The content in Markdown format.
    content: String,
    /// If true, appends to the existing page instead of overwriting.
    append: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct SearchWikiArgs {
    /// The keyword or phrase to search for across all wiki pages.
    query: String,
}

#[derive(Deserialize, JsonSchema)]
struct SearchWikiByTagArgs {
    /// The tag to search for (e.g., 'rust', 'project-ideas'). Do not include the '#' symbol.
    tag: String,
}

#[derive(Deserialize, JsonSchema)]
struct GetWikiGraphArgs {}

#[derive(Deserialize, JsonSchema)]
struct CreateDailyNoteArgs {
    /// Optional content to pre-fill the daily note with.
    content: Option<String>,
}

/// Helper to get the wiki directory path.
async fn get_wiki_dir() -> std::result::Result<PathBuf, AdkError> {
    let root = get_workspace_dir().await?;
    let wiki_dir = root.join("wiki");
    if !wiki_dir.exists() {
        fs::create_dir_all(&wiki_dir)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to create wiki directory: {}", e)))?;
    }
    Ok(wiki_dir)
}

/// Helper to sanitize title into a filename.
fn sanitize_title(title: &str) -> String {
    title.trim().replace(" ", "-").to_lowercase()
}

/// Adds or updates a wiki page in the 'wiki/' directory.
#[tool]
async fn add_wiki_page(args: AddWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let filename = format!("{}.md", sanitize_title(&args.title));
    let path = wiki_dir.join(filename);

    if args.append.unwrap_or(false) && path.exists() {
        let mut existing = fs::read_to_string(&path).await.unwrap_or_default();
        existing.push_str("\n\n");
        existing.push_str(&args.content);
        fs::write(&path, existing)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to append to wiki page: {}", e)))?;
        Ok(
            json!({"status": "success", "message": format!("Appended to wiki page '{}'", args.title)}),
        )
    } else {
        fs::write(&path, &args.content)
            .await
            .map_err(|e| AdkError::tool(format!("Failed to write wiki page: {}", e)))?;
        Ok(json!({"status": "success", "message": format!("Saved wiki page '{}'", args.title)}))
    }
}

/// Retrieves the content of a specific wiki page.
#[tool]
async fn get_wiki_page(args: WikiPageArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let filename = format!("{}.md", sanitize_title(&args.title));
    let path = wiki_dir.join(filename);

    if !path.exists() {
        return Err(AdkError::tool(format!(
            "Wiki page '{}' not found.",
            args.title
        )));
    }

    let content = fs::read_to_string(&path)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to read wiki page: {}", e)))?;

    Ok(json!({ "title": args.title, "content": content }))
}

/// Lists all available wiki pages.
#[tool]
async fn list_wiki_pages(_args: serde_json::Value) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut dir = fs::read_dir(&wiki_dir)
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;
    let mut pages = Vec::new();

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".md") {
            pages.push(name.replace(".md", ""));
        }
    }

    Ok(json!({ "pages": pages }))
}

/// Searches for a keyword across all wiki pages.
#[tool]
async fn search_wiki(args: SearchWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut dir = fs::read_dir(&wiki_dir)
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;
    let mut matches = Vec::new();
    let query = args.query.to_lowercase();

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            if content.to_lowercase().contains(&query) {
                matches.push(entry.file_name().to_string_lossy().replace(".md", ""));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": "No matches found in wiki." }))
    } else {
        Ok(json!({ "matches": matches }))
    }
}

/// Searches all wiki pages for a specific tag (e.g., '#rust').
#[tool]
async fn search_wiki_by_tag(args: SearchWikiByTagArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut dir = fs::read_dir(&wiki_dir)
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;
    let mut matches = Vec::new();
    let tag_pattern = format!("#{}", args.tag.to_lowercase()); // Construct regex pattern for tag

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            if content.to_lowercase().contains(&tag_pattern) {
                matches.push(entry.file_name().to_string_lossy().replace(".md", ""));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": format!("No pages found with tag \'#{}\'.", args.tag) }))
    } else {
        Ok(json!({ "tag": args.tag, "matches": matches }))
    }
}

/// Scans all wiki pages for [[wikilink]] references and builds a knowledge graph.
#[tool]
async fn get_wiki_graph(_args: GetWikiGraphArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut dir = fs::read_dir(&wiki_dir)
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;
    
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let wikilink_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let filename = entry.file_name().to_string_lossy().to_string();
            let title = filename.replace(".md", "");
            
            nodes.push(json!({"id": &title, "label": &title}));

            let content = fs::read_to_string(&path).await.unwrap_or_default();
            for cap in wikilink_regex.captures_iter(&content) {
                let target_title = cap[1].to_string();
                edges.push(json!({"source": &title, "target": &target_title}));
            }
        }
    }

    Ok(json!({ "nodes": nodes, "edges": edges }))
}

/// Creates a new wiki page for the current date (e.g., 'YYYY-MM-DD.md').
#[tool]
async fn create_daily_note(args: CreateDailyNoteArgs) -> std::result::Result<Value, AdkError> {
    let today = Utc::now();
    let title = format!("{}-{:02}-{:02}", today.year(), today.month(), today.day());
    let filename = format!("{}.md", &title);
    let wiki_dir = get_wiki_dir().await?;
    let path = wiki_dir.join(&filename);

    if path.exists() {
        return Err(AdkError::tool(format!("Daily note for {} already exists.", title)));
    }

    let initial_content = args.content.unwrap_or_else(|| format!("# {}\n\n", title));
    fs::write(&path, initial_content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to create daily note: {}", e)))?;

    Ok(json!({"status": "success", "message": format!("Created daily note for '{}'", title), "title": title}))
}

/// Generates a 'SUMMARY.md' file that indexes all available wiki pages with a brief overview.
#[tool]
async fn summarize_wiki(_args: serde_json::Value) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut dir = fs::read_dir(&wiki_dir)
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?;
    let mut summary_content =
        "# Wiki Summary Index\n\nGenerated automatically by Nami.\n\n".to_string();

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AdkError::tool(e.to_string()))?
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let filename = entry.file_name().to_string_lossy().to_string();
            let title = filename.replace(".md", "");
            if title == "SUMMARY" {
                continue;
            }

            let content = fs::read_to_string(&path).await.unwrap_or_default();
            let first_line = content
                .lines()
                .next()
                .unwrap_or("No content")
                .trim_start_matches('#')
                .trim();
            summary_content.push_str(&format!("- **{}**: {}\n", title, first_line));
        }
    }

    let summary_path = wiki_dir.join("SUMMARY.md");
    fs::write(&summary_path, summary_content)
        .await
        .map_err(|e| AdkError::tool(format!("Failed to write SUMMARY.md: {}", e)))?;

    Ok(json!({"status": "success", "message": "Wiki summary (SUMMARY.md) has been updated!"}))
}

pub fn wiki_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(AddWikiPage),
        Arc::new(GetWikiPage),
        Arc::new(ListWikiPages),
        Arc::new(SearchWiki),
        Arc::new(SummarizeWiki),
        Arc::new(SearchWikiByTag),
        Arc::new(GetWikiGraph),
        Arc::new(CreateDailyNote),
    ]
}
