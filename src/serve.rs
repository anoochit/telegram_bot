use adk_rust::Agent;
use adk_rust::Launcher;
use std::sync::Arc;

pub(crate) async fn run_serve(agent: Arc<dyn Agent>) -> anyhow::Result<()> {
    Launcher::new(agent).run_serve_directly(8080).await?;
    Ok(())
}
