//! Finite blog generator CLI; no application runtime, database, watcher or listener starts.

use std::path::Path;

/// Parse explicit generation options and turn a stale check report into a nonzero exit.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Derive the app directory without `..`: the managed writer rejects parent traversal.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("blog tool has no application directory")?;
    let mut input = root.join("schema.idea");
    let mut check = false;
    let mut args = std::env::args().skip(1);
    // Select a finite emitted graph; neither branch starts the native application runtime.
    let application = match args.next().as_deref() {
        Some("generate") => false,
        Some("generate-app") => true,
        _ => {
            return Err(
                "usage: blog-tool generate|generate-app [--input PATH] [--output PATH] [--check]"
                    .into(),
            );
        }
    };
    // Keep the optional native-only proof tree separate from the application's full partitions.
    let mut output = root.join(if application {
        ".build/app-generated"
    } else {
        ".build/generated"
    });
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = args.next().ok_or("--input needs a path")?.into(),
            "--output" => output = args.next().ok_or("--output needs a path")?.into(),
            "--check" => check = true,
            _ => return Err(format!("unknown generator argument: {arg}").into()),
        }
    }

    // Install reports what changed; only check mode treats differences as failure.
    let report = if application {
        blog_tool::generate_application(&input, &output, check)?
    } else {
        blog_tool::generate(&input, &output, check)?
    };
    if check && !report.is_current() {
        return Err(format!("blog generated output is stale: {:?}", report.differences).into());
    }
    println!(
        "blog native source {}",
        if check { "fresh" } else { "generated" }
    );
    Ok(())
}
