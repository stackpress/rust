//! Finite SSR process: write one document to stdout; never bind a listener.
use leptos::prelude::*;
use stackpress_framework_ui::components::{ADVANCED_STYLES, BREADTH_STYLES, CORE_STYLES};

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
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Generated admin</title><style>{CORE_STYLES}{ADVANCED_STYLES}{BREADTH_STYLES}*{{box-sizing:border-box}}body{{font:16px system-ui;max-width:1100px;margin:2rem auto;padding:1rem}}section{{border:1px solid #ccc;padding:1rem;margin:1rem 0;min-width:0}}fieldset{{min-width:0}}input:not([type=radio]):not([type=checkbox]),select{{max-width:100%}}.sp-field{{margin:.5rem 0}}.sp-role-value{{display:flex;flex-wrap:wrap;gap:.5rem;margin:.25rem 0}}label{{display:block}}input,select{{padding:.4rem}}output{{overflow-wrap:anywhere}}h3{{margin-bottom:.4rem}}.sp-error:empty,.sp-help:empty{{display:none}}</style></head><body><div id=\"app\">{body}</div><script type=\"module\" src=\"./assets/start.js\"></script></body></html>"
    );
}

/// Consume exactly one bounded trusted protocol object; never start a server or worker loop.
fn finite() {
    use std::io::Read;
    let mut bytes = Vec::new();
    let result = (|| -> Result<String, Box<dyn std::error::Error>> {
        std::io::stdin().take(1_048_577).read_to_end(&mut bytes)?;
        if bytes.len() > 1_048_576 {
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
