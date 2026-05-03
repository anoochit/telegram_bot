use adk_rust::prelude::*;
use adk_rust::Tool;
use adk_rust::serde::Deserialize;
use schemars::JsonSchema;
use std::sync::Arc;
use std::collections::HashMap;
use futures::future::join_all;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
pub struct WritingTask {
    /// The specific task or prompt for this writing job.
    pub prompt: String,
    /// The name of the specialized agent to handle this task (e.g. 'documentation_architect', 'web_developer').
    pub specialist: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ParallelWriterArgs {
    /// A list of writing tasks to execute simultaneously.
    pub tasks: Vec<WritingTask>,
}

pub struct ParallelWriter {
    specialists: HashMap<String, Arc<dyn Tool>>,
}

impl ParallelWriter {
    pub fn new(specialists: HashMap<String, Arc<dyn Tool>>) -> Self {
        Self { specialists }
    }
}

#[async_trait::async_trait]
impl Tool for ParallelWriter {
    fn name(&self) -> &str {
        "parallel_writer"
    }

    fn description(&self) -> &str {
        "Executes multiple writing tasks in parallel using specialized sub-agents. Use this for complex document generation or multi-file analysis."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(json!({
            "type": "object",
            "properties": {
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "prompt": { "type": "string", "description": "The prompt or instructions for the sub-agent." },
                            "specialist": { "type": "string", "description": "The name of the sub-agent to use (e.g., 'documentation_architect', 'web_developer', 'generalist')." }
                        },
                        "required": ["prompt", "specialist"]
                    }
                }
            },
            "required": ["tasks"]
        }))
    }

    async fn execute(&self, ctx: Arc<dyn ToolContext>, args: Value) -> std::result::Result<Value, AdkError> {
        let args: ParallelWriterArgs = serde_json::from_value(args)
            .map_err(|e| AdkError::tool(format!("Invalid arguments: {}", e)))?;

        let mut futures = Vec::new();

        for task in args.tasks {
            if let Some(tool) = self.specialists.get(&task.specialist) {
                let tool = tool.clone();
                let prompt = task.prompt.clone();
                let ctx = ctx.clone();
                let specialist_name = task.specialist.clone();
                
                futures.push(tokio::spawn(async move {
                    // Call the tool (which is an AgentTool wrapping the sub-agent)
                    // AgentTool expects the input in an 'input' field.
                    match tool.execute(ctx, json!({ "input": prompt })).await {
                        Ok(res) => format!("[{}] success: {}", specialist_name, res),
                        Err(e) => format!("[{}] error: {}", specialist_name, e),
                    }
                }));
            } else {
                let specialist = task.specialist.clone();
                futures.push(tokio::spawn(async move {
                    format!("Error: Specialist '{}' not found", specialist)
                }));
            }
        }

        let results = join_all(futures).await;
        let mut final_results = Vec::new();
        
        for res in results {
            match res {
                Ok(r) => final_results.push(r),
                Err(e) => final_results.push(format!("Internal error: {}", e)),
            }
        }

        Ok(json!({ 
            "status": "success", 
            "tasks_executed": final_results.len(), 
            "outputs": final_results 
        }))
    }
}

pub fn parallel_writer_tool(specialists: HashMap<String, Arc<dyn Tool>>) -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(ParallelWriter::new(specialists))]
}
