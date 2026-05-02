use adk_rust::Agent;
use adk_rust::Launcher;
use std::sync::Arc;

pub(crate) async fn run_serve(agent: Arc<dyn Agent>, port: u16) -> anyhow::Result<()> {
    Launcher::new(agent).run_serve_directly(port).await?;
    Ok(())
}
