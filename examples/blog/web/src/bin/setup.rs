//! Explicit source install command; installation is never a side effect of app startup.

/// Install only the explicitly selected SQLite file, then await its connection shutdown.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| ".build/blog.sqlite".into());
    blog_web::plugins::store::setup(std::path::Path::new(&path)).await?;
    Ok(())
}
