//! Direct JWT/permission mechanisms need neither the facade nor a plugin/server.
use stackpress_framework_session as identity;
#[path = "../../support/native.rs"]
mod native;

/// Execute the direct mechanism proof on an application-selected runtime.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    native::prove().await?;
    println!("slice9a direct consumer passed");
    Ok(())
}
