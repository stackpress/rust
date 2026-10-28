//! Finite proof-only SSR document; no registration or listener enters the ordinary gallery.
use leptos::prelude::*;
use stackpress_framework_ui::components::{BASE_STYLES, BREADTH_STYLES, CORE_STYLES};

/// Write matching collection markup or an empty CSR root for intercepted browser proofs.
fn main() {
    let owner = Owner::new();
    let body = if std::env::args().any(|value| value == "--csr") {
        String::new()
    } else {
        owner.with(|| {
            view! {<stackpress_example_ui_gallery_shared::collection_proof::CollectionProof/>}
                .to_html()
        })
    };
    owner.cleanup();
    println!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><style>{BASE_STYLES}{CORE_STYLES}{BREADTH_STYLES}body{{margin:0}}main{{padding:1rem}}output{{display:block;overflow-wrap:anywhere}}</style></head><body><div id=\"app\">{body}</div><script type=\"module\" src=\"./assets/collections.js\"></script></body></html>"
    );
}
