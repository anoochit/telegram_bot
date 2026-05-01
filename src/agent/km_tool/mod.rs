use std::sync::Arc;
use adk_rust::serde::Deserialize;
use adk_tool::{tool, AdkError};
use adk_rust::Tool;
use schemars::JsonSchema;
use serde_json::{json, Value};
use crate::agent::database::DbManager;

#[derive(Deserialize, JsonSchema)]
struct KmArgs {
    /// The query to search for in local Knowledge Management system
    query: String,
}

#[derive(Deserialize, JsonSchema)]
struct AddKnowledgeArgs {
    /// The content to add to the Knowledge Management system
    content: String,
}

#[derive(Deserialize, JsonSchema)]
struct GetKmStatsArgs {}

/// Retrieves statistics about the local Knowledge Management system.
#[tool]
async fn get_km_stats(_args: GetKmStatsArgs) -> std::result::Result<Value, AdkError> {
    let db_manager = DbManager::new().await
        .map_err(|e| AdkError::tool(format!("Failed to connect to database: {}", e)))?;
    
    let conn = db_manager.conn.lock().unwrap();

    let mut stmt = conn.prepare("SELECT COUNT(*) FROM knowledge_items")
        .map_err(|e| AdkError::tool(format!("Database error: {}", e)))?;

    let count: i64 = stmt.query_row([], |row| row.get(0))
        .map_err(|e| AdkError::tool(format!("Query error: {}", e)))?;

    Ok(json!({
        "total_documents": count,
    }))
}

/// Searches local Knowledge Management system for information using vector similarity.
#[tool]
async fn search_km(args: KmArgs) -> std::result::Result<Value, AdkError> {
    let db_manager = DbManager::new().await
        .map_err(|e| AdkError::tool(format!("Failed to connect to database: {}", e)))?;
    
    let conn = db_manager.conn.lock().unwrap();

    // Perform vector search
    let mut stmt = conn.prepare(
        "SELECT content FROM knowledge_items WHERE content LIKE ? LIMIT 5"
    ).map_err(|e| AdkError::tool(format!("Database error: {}", e)))?;

    let query_param = format!("%{}%", args.query);
    let results: Vec<String> = stmt.query_map([query_param], |row| {
        row.get(0)
    })
    .map_err(|e| AdkError::tool(format!("Query error: {}", e)))?
    .filter_map(Result::ok)
    .collect();
        
    if !results.is_empty() {
        Ok(json!({"results": results}))
    } else {
        Ok(json!({"message": "Information not found in knowledge management system."}))
    }
}

/// Adds new information to the Knowledge Management system.
#[tool]
async fn add_knowledge(args: AddKnowledgeArgs) -> std::result::Result<Value, AdkError> {
    let db_manager = DbManager::new().await
        .map_err(|e| AdkError::tool(format!("Failed to connect to database: {}", e)))?;
    
    let conn = db_manager.conn.lock().unwrap();

    // Placeholder: Inserting empty blob for embedding until an embedding model is integrated.
    conn.execute(
        "INSERT INTO knowledge_items (content, embedding) VALUES (?, ?)",
        (&args.content, vec![0u8; 768 * 4]), // Assuming float32 embedding
    ).map_err(|e| AdkError::tool(format!("Failed to add knowledge: {}", e)))?;
        
    Ok(json!({"message": "Successfully added content to knowledge management system."}))
}

pub fn km_tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(SearchKm), Arc::new(AddKnowledge), Arc::new(GetKmStats)]
}
