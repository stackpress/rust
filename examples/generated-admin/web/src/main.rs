//! Finite SSR process: write one document to stdout; never bind a listener.
use leptos::prelude::*;

/// Render the same generated component tree later hydrated by the WASM client.
fn main() {
    let csr = std::env::args().any(|arg| arg == "--csr");
    let owner = Owner::new();
    let body = if csr {
        String::new()
    } else {
        owner.with(|| view! { <generated_admin_shared::Admin/> }.to_html())
    };
    owner.cleanup();
    println!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Generated admin</title><style>body{{font:16px system-ui;max-width:1100px;margin:2rem auto;padding:1rem}}section{{border:1px solid #ccc;padding:1rem;margin:1rem 0}}.sp-field{{margin:.5rem 0}}.sp-role-value{{display:flex;gap:.5rem;margin:.25rem 0}}label{{display:block}}input,select{{padding:.4rem}}h3{{margin-bottom:.4rem}}.sp-error:empty,.sp-help:empty{{display:none}}</style></head><body><div id=\"app\">{body}</div><script type=\"module\" src=\"/assets/start.js\"></script></body></html>"
    );
}
