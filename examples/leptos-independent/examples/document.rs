//! One finite document on stdout; no transport, storage, router or listener.
use leptos_independent::{Initial, native};
use stackpress_leptos::render::{RenderLimits, initialize_ssr};

/// Render explicit choices supplied by the proof; unknown input fails before output.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("expected <hydrate|csr> <light|dark> <en|fr>".into());
    }
    if !matches!(args[0].as_str(), "hydrate" | "csr") {
        return Err("unknown document mode".into());
    }
    let initial = Initial::new(&args[1], &args[2], "Seeded draft".into())?;
    initialize_ssr()?;
    println!(
        "{}",
        native::render(initial, args[0] == "csr", RenderLimits::default()).await?
    );
    Ok(())
}
