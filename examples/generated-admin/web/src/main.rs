//! Finite SSR process: write one document to stdout; never bind a listener.
use leptos::prelude::*;

/// Render the same generated component tree later hydrated by the WASM client.
fn main() {
    if std::env::args().any(|arg| arg == "--request") {
        finite();
        return;
    }
    let csr = std::env::args().any(|arg| arg == "--csr");
    let owner = Owner::new();
    let body = if csr {
        String::new()
    } else {
        owner.with(|| view! { <generated_admin_shared::Admin/> }.to_html())
    };
    owner.cleanup();
    println!(
        "{}",
        generated_admin_web::render::roles_document(&body, "./assets/start.js")
    );
}

/// Consume exactly one bounded trusted protocol object; never start a server or worker loop.
fn finite() {
    use std::io::Read;
    let mut bytes = Vec::new();
    let result = (|| -> Result<String, Box<dyn std::error::Error>> {
        std::io::stdin().take(8_388_609).read_to_end(&mut bytes)?;
        if bytes.len() > 8_388_608 {
            return Err("finite input budget exceeded".into());
        }
        let input = serde_json::from_slice(&bytes)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(serde_json::to_string(
            &runtime.block_on(generated_admin_web::finite::run(input))?,
        )?)
    })();
    match result {
        Ok(output) => println!("{output}"),
        Err(_) => {
            eprintln!("Finite admin request failed");
            std::process::exit(1);
        }
    }
}
