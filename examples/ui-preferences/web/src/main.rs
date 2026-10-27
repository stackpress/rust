//! Unchanged finite pipe entry point.
/// Execute exactly one finite request without listening.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    stackpress_example_ui_preferences_web::run().await
}
