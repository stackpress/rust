//! Browser builds, or an explicitly requested live development loop for the copied blog.

/// Dispatch an explicit build or live loop without linking application/generated source.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(wrapper) = std::env::var_os("BLOG_DEV_COMPILER_WRAPPER") {
        return blog_xtask::compiler(&wrapper, &std::env::args_os().skip(1).collect::<Vec<_>>());
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask has no blog directory")?;
    let args: Vec<_> = std::env::args().skip(1).collect();
    let target = std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from);
    match args.first().map(String::as_str) {
        Some("build-browser") if args.len() == 1 => {
            blog_xtask::build_browser(root, target.as_deref())
        }
        Some("dev") => blog_xtask::dev::run(
            root,
            target.as_deref(),
            blog_xtask::dev::Options::parse(&args[1..])?,
        ),
        _ => Err("usage: cargo xtask build-browser | dev [--port N] [--reload-port M]".into()),
    }
}
