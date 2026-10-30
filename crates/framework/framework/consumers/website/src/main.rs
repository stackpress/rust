//! Finite website application; no listener or runtime generation.
use stackpress::framework;
#[allow(dead_code)]
#[path = "../../support/generated.rs"]
mod generated;
#[path = "../../support/host.rs"]
mod host;
#[path = "../../support/native.rs"]
mod native;
#[allow(dead_code)]
#[path = "../../generated/ui/pages_host.rs"]
mod pages_host;

/// The application owns all IO and closes its SQLite file and source task groups.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    stackpress::framework_view::leptos::render::initialize_ssr()?;
    let inventory = native::prove(false).await?;
    println!("INVENTORY {}", serde_json::to_string(&inventory)?);
    println!("slice11 website consumer passed");
    Ok(())
}
