//! Finite SSR document producer; it never starts a transport or listener.
use leptos::prelude::*;
/// Render and dispose one explicit owner without retaining an execution host.
fn main() {
    let owner = Owner::new();
    let html = owner.with(|| source_roles_consumer::app().to_html());
    owner.cleanup();
    println!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><style>{}</style></head><body class=\"sp-root\" data-theme=\"light\"><div id=\"app\">{html}</div><script type=\"module\" src=\"/start.js\"></script></body></html>",
        [
            stackpress_leptos::components::BASE_STYLES,
            stackpress_leptos::components::CORE_STYLES,
            stackpress_leptos::components::BREADTH_STYLES,
            stackpress_leptos::components::source::SOURCE_STYLES
        ]
        .concat()
    );
}
