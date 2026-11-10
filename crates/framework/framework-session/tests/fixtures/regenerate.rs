//! Regenerate owned source models; never built into the library.
mod generation;
/// A separate example can repair missing output before the integration tests compile.
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut input = None;
    let mut output = None;
    let mut check = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => check = true,
            "--input" => {
                input = Some(std::path::PathBuf::from(
                    args.next().ok_or("--input needs a path")?,
                ))
            }
            "--output" => {
                output = Some(std::path::PathBuf::from(
                    args.next().ok_or("--output needs a path")?,
                ))
            }
            _ => return Err("unknown generation argument".into()),
        }
    }
    match (input, output) {
        (Some(input), Some(output)) => generation::generate(&input, &output, check),
        (None, None) => generation::fixtures(check),
        _ => Err("--input and --output must be supplied together".into()),
    }
}
