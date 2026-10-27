//! Unchanged finite pipe entry point; reusable registration lives in the library.
/// Handle one pipe request and exit without listening.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    stackpress_example_ui_gallery_web::run().await
}
