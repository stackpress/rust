//! Finite development commands for the copied blog; never starts a listener.

/// Select the browser build without loading the application or its generated source.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask has no blog directory")?;
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args != ["build-browser"] {
        return Err("usage: cargo xtask build-browser".into());
    }
    let target = std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from);
    blog_xtask::build_browser(root, target.as_deref())
}
