use crate::agent::utils::get_workspace_dir;
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use adk_tool::{AdkError, tool};
use schemars::JsonSchema;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use regex::Regex;
use chrono::{Utc, Datelike};
use walkdir::WalkDir;
use std::collections::HashMap;

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
struct SearchWikiByDateArgs {
    /// The date to search for in 'YYYY-MM-DD' format (e.g., '2026-05-03').
    date: String,
}

#[derive(Deserialize, JsonSchema)]
struct GetWikiGraphArgs {}

#[derive(Deserialize, JsonSchema)]
struct CreateDailyNoteArgs {
    /// Optional content to pre-fill the daily note with.
    content: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct SanitizeWikiVaultArgs {}

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

/// Helper to convert a string to Title Case (Obsidian-style with spaces).
/// Converts dashes (except in dates), underscores, and multiple spaces into single spaces and capitalizes each word.
fn to_title_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    let mut last_was_space = false;
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        let c = chars[i];
        
        // Special case: Preserve dashes if they look like a date (Digit-Dash-Digit)
        let is_date_dash = c == '-' 
            && i > 0 
            && i < chars.len() - 1 
            && chars[i-1].is_ascii_digit() 
            && chars[i+1].is_ascii_digit();

        if (c == '-' && !is_date_dash) || c == '_' || c == ' ' {
            if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
            last_was_space = false;
        } else {
            result.push(c);
            last_was_space = false;
        }
    }
    result.trim().to_string()
}

/// Helper to sanitize title into a filename while enforcing Obsidian style (Title Case with spaces).
/// Removes characters that are invalid in Windows/Unix filenames, except for forward slashes.
fn sanitize_title(title: &str) -> String {
    // Convert backslashes to forward slashes for cross-platform folder support
    let sanitized_path = title.trim().replace("\\", "/");
    
    // Process each component of the path separately to preserve folder structure
    let parts: Vec<String> = sanitized_path.split('/')
        .map(|part| {
            let mut p = part.to_string();
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            p.retain(|c| !invalid_chars.contains(&c));
            to_title_case(&p)
        })
        .collect();
    
    parts.join("/")
}

/// Helper to get the relative title from a file path
fn get_relative_title(wiki_dir: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(wiki_dir)
        .unwrap_or(file_path)
        .with_extension("")
        .to_string_lossy()
        .replace("\\", "/") // Normalize to forward slashes for cross-platform consistency
}

/// Adds or updates a wiki page in the 'wiki/' directory. Supports nested folders (e.g., 'Projects/Notes').
#[tool]
async fn add_wiki_page(args: AddWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let sanitized_title = sanitize_title(&args.title);
    let filename = format!("{}.md", sanitized_title);
    let path = wiki_dir.join(&filename);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AdkError::tool(format!("Failed to create parent directories: {}", e)))?;
        }
    }

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

/// Lists all available wiki pages recursively.
#[tool]
async fn list_wiki_pages(_args: serde_json::Value) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut pages = Vec::new();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            pages.push(get_relative_title(&wiki_dir, path));
        }
    }

    Ok(json!({ "pages": pages }))
}

/// Searches for a keyword across all wiki pages recursively.
#[tool]
async fn search_wiki(args: SearchWikiArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut matches = Vec::new();
    let query = args.query.to_lowercase();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            if content.to_lowercase().contains(&query) {
                matches.push(get_relative_title(&wiki_dir, path));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": "No matches found in wiki." }))
    } else {
        Ok(json!({ "matches": matches }))
    }
}

/// Searches all wiki pages for a specific tag (e.g., '#rust') recursively.
#[tool]
async fn search_wiki_by_tag(args: SearchWikiByTagArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut matches = Vec::new();
    let tag_pattern = format!("#{}", args.tag.to_lowercase());

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            if content.to_lowercase().contains(&tag_pattern) {
                matches.push(get_relative_title(&wiki_dir, path));
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": format!("No pages found with tag \'#{}\'.", args.tag) }))
    } else {
        Ok(json!({ "tag": args.tag, "matches": matches }))
    }
}

/// Searches for wiki pages created on a specific date (e.g., '2026-05-03'), especially Daily Notes.
#[tool]
async fn search_wiki_by_date(args: SearchWikiByDateArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut matches = Vec::new();
    let target_date = args.date.trim();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative_title = get_relative_title(&wiki_dir, path);
            
            // Check if the filename contains the date
            if relative_title.contains(target_date) {
                matches.push(relative_title);
            }
        }
    }

    if matches.is_empty() {
        Ok(json!({ "message": format!("No notes found for date '{}'.", target_date) }))
    } else {
        Ok(json!({ "date": target_date, "matches": matches }))
    }
}

/// Scans all wiki pages recursively for [[wikilink]] references and builds a knowledge graph.
#[tool]
async fn get_wiki_graph(_args: GetWikiGraphArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let wikilink_regex = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let title = get_relative_title(&wiki_dir, path);
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

/// Creates a new wiki page for the current date (e.g., 'Daily Notes/YYYY-MM-DD.md').
#[tool]
async fn create_daily_note(args: CreateDailyNoteArgs) -> std::result::Result<Value, AdkError> {
    let today = Utc::now();
    let title = format!("Daily Notes/{}-{:02}-{:02}", today.year(), today.month(), today.day());
    
    // We can just reuse add_wiki_page logic here to handle the folder creation naturally
    let add_args = AddWikiArgs {
        title: title.clone(),
        content: args.content.unwrap_or_else(|| format!("# {}\n\n", title)),
        append: Some(false),
    };
    
    add_wiki_page(add_args).await?;

    Ok(json!({"status": "success", "message": format!("Created daily note for '{}'", title), "title": title}))
}

/// Generates a 'SUMMARY.md' file that indexes all available wiki pages recursively with a brief overview.
#[tool]
async fn summarize_wiki(_args: serde_json::Value) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut summary_content =
        "# Wiki Summary Index\n\nGenerated automatically by Nami.\n\n".to_string();

    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let title = get_relative_title(&wiki_dir, path);
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

/// Cleans up the wiki vault by renaming old dash-cased files to Title Case with spaces, and updating wikilinks.
#[tool]
async fn sanitize_wiki_vault(_args: SanitizeWikiVaultArgs) -> std::result::Result<Value, AdkError> {
    let wiki_dir = get_wiki_dir().await?;
    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut files_to_process = Vec::new();

    // Pass 1: Identify files that need renaming
    for entry in WalkDir::new(&wiki_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let relative_title = get_relative_title(&wiki_dir, path);
            files_to_process.push(path.to_path_buf());
            
            // Only process the actual filename, keep the parent path
            let file_stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let new_stem = to_title_case(&file_stem);
            
            if file_stem != new_stem {
                let mut new_title = relative_title.clone();
                // Replace just the filename part in the relative path
                if let Some(pos) = new_title.rfind(&file_stem) {
                    new_title.replace_range(pos..pos + file_stem.len(), &new_stem);
                }
                rename_map.insert(relative_title, new_title);
            }
        }
    }

    let mut renamed_count = 0;
    let mut links_updated_count = 0;

    // Pass 2: Rename files
    for (old_title, new_title) in &rename_map {
        let old_path = wiki_dir.join(format!("{}.md", old_title));
        let new_path = wiki_dir.join(format!("{}.md", new_title));
        
        if old_path.exists() && !new_path.exists() {
            fs::rename(&old_path, &new_path).await.map_err(|e| AdkError::tool(format!("Failed to rename {:?} to {:?}: {}", old_path, new_path, e)))?;
            renamed_count += 1;
        }
    }

    // Pass 3: Update contents (wikilinks) in ALL files
    // Use the updated file paths if they were renamed
    let current_files: Vec<PathBuf> = files_to_process.into_iter().map(|p| {
        let relative = get_relative_title(&wiki_dir, &p);
        if let Some(new_rel) = rename_map.get(&relative) {
            wiki_dir.join(format!("{}.md", new_rel))
        } else {
            p
        }
    }).collect();

    for path in current_files {
        if path.exists() {
            let content = fs::read_to_string(&path).await.unwrap_or_default();
            let mut new_content = content.clone();
            
            for (old_title, new_title) in &rename_map {
                // Match exact wikilinks e.g., [[old-title]] -> [[New Title]]
                let old_link = format!("[[{}]]", old_title);
                let new_link = format!("[[{}]]", new_title);
                if new_content.contains(&old_link) {
                    new_content = new_content.replace(&old_link, &new_link);
                    links_updated_count += 1;
                }
            }

            if content != new_content {
                fs::write(&path, new_content).await.map_err(|e| AdkError::tool(format!("Failed to update links in {:?}: {}", path, e)))?;
            }
        }
    }

    Ok(json!({
        "status": "success", 
        "message": "Vault cleanup complete.",
        "files_renamed": renamed_count,
        "links_updated": links_updated_count,
        "renamed_mapping": rename_map
    }))
}

pub fn wiki_tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(AddWikiPage),
        Arc::new(GetWikiPage),
        Arc::new(ListWikiPages),
        Arc::new(SearchWiki),
        Arc::new(SummarizeWiki),
        Arc::new(SearchWikiByTag),
        Arc::new(SearchWikiByDate),
        Arc::new(GetWikiGraph),
        Arc::new(CreateDailyNote),
        Arc::new(SanitizeWikiVault),
    ]
}
