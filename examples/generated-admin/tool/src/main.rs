//! Finite generation CLI; uses the same registry and marked output root in check/install mode.
use std::{path::PathBuf, process::ExitCode};

/// Parse explicit local generator options; no ambient plugin configuration changes output.
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()?;
    let mut input = root.join("schema.idea");
    let mut output = root.join("shared/src/generated");
    let mut check = false;
    let mut plugin = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "emit" | "generate" => {}
            "--check" => check = true,
            "--plugin" => plugin = true,
            "--input" => input = args.next().ok_or("missing input path")?.into(),
            "--output" => output = args.next().ok_or("missing output path")?.into(),
            _ => return Err("unknown generator option".into()),
        }
    }
    let report = generated_admin_tool::generate(&input, &output, check, plugin)?;
    if check && !report.is_current() {
        return Err("generated admin output is stale".into());
    }
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

/// Report a finite failure through exit status while preserving the private diagnostic chain.
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:?}");
            ExitCode::FAILURE
        }
    }
}
