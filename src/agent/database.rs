use crate::agent::utils::get_workspace_root;
use rusqlite::Connection;
use sqlite_vec::sqlite3_vec_init;
use std::sync::{Arc, Mutex};

pub struct DbManager {
    pub conn: Arc<Mutex<Connection>>,
}

impl DbManager {
    pub async fn new() -> anyhow::Result<Self> {
        let workspace_root = get_workspace_root().await?;
        let db_path = workspace_root.join("knowledge.db");

        // Register sqlite-vec extension
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_vec_init as *const (),
            )));
        }

        let conn = Connection::open(db_path)?;

        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS knowledge_items (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS vec_knowledge USING vec0(
                embedding float[768] -- Assuming 768 dimensions for common embedding models
            );",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}
