//! Emit one finite inert SSR/CSR fixture; this process never opens a listener.
use leptos::prelude::*;
use leptos_independent::realtime::RealtimeRoot;
use stackpress_leptos::render::{RenderLimits, Renderer, document_shell, initialize_ssr};
use std::convert::Infallible;
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_ssr()?;
    let csr = std::env::args().nth(1).as_deref() == Some("csr");
    let rendered=Renderer::try_prepare(RenderLimits::default(),move ||Ok::<_,Infallible>(if csr {().into_any()}else{view! {<RealtimeRoot/>}.into_any()}),
        |body|Ok(document_shell("lang=\"en\"","<meta charset=\"utf-8\">",&format!("<div id=\"realtime-root\">{body}</div><script type=\"module\" src=\"/assets/start.js\"></script>"))))?.finish().await?;
    println!("{rendered}");
    Ok(())
}
