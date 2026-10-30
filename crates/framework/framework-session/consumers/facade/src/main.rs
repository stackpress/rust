//! The explicitly selected facade exposes the same direct library under a Rust-style name.
use stackpress::framework_session as identity;
#[path = "../../support/native.rs"]
mod native;

/// Reuse the direct proof so facade selection cannot substitute a different identity contract.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _request = stackpress::Request::new();
    native::prove().await?;
    println!("slice9a facade consumer passed");
    Ok(())
}
