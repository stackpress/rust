//! Finite synchronous SSR after authorized generated loaders; one request-local Owner.
use leptos::prelude::*;
use stackpress_framework_admin::pages::Snapshot;
use stackpress_framework_ui::components::{
    ADVANCED_STYLES, BASE_STYLES, BREADTH_STYLES, CORE_STYLES,
};

/// Render trusted catalogue component output with the same styles in every host.
/// The module URL is host-selected; Leptos escapes it as an attribute, never HTML.
pub fn roles_document(body: &str, module: &str) -> String {
    let script = view! {<script type="module" src=module.to_owned()></script>}.to_html();
    let catalogue = generated_admin_shared::ROLE_CATALOGUE_STYLES;
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Generated roles</title><style>{BASE_STYLES}{CORE_STYLES}{ADVANCED_STYLES}{BREADTH_STYLES}{catalogue}body{{margin:0}}</style></head><body><div id=\"app\">{body}</div>{script}</body></html>"
    )
}

/// Render a safe static document failure without any protected seed or bootstrap.
/// Only the public status and compiled list URL enter the markup; causes stay private.
pub fn failure_document(status: u16, list: &str) -> String {
    let (heading, message) = match status {
        401 | 403 => (
            "Access denied",
            "You do not have permission. Ask an administrator for access.",
        ),
        404 => (
            "Page not found",
            "This page or record is unavailable. Return to the list.",
        ),
        _ => (
            "Unable to load this page",
            "Reload this page or return to the list. No changes were made by this page request.",
        ),
    };
    // Rendering escapes all dynamic text/attributes. GET recovery never retries a write.
    let body = view! {
        <div class="sp-root" data-theme="light"><main class="sp-blank">
            <p>{format!("Error {status}")}</p><h1>{heading}</h1><p>{message}</p>
            <a href=list.to_owned()>"Return to list"</a>
        </main></div>
    }
    .to_html();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{heading}</title><style>{BASE_STYLES}body{{margin:0}}</style></head><body>{body}</body></html>"
    )
}

/// Render matching public state for SSR or CSR without executable inline bootstrap data.
/// # Errors
/// Refuses oversized/browser-inexact public seed data before document assembly.
pub fn document(
    snapshot: Snapshot,
    csr: bool,
) -> stackpress_framework_schema::SchemaResult<String> {
    let wire = stackpress_framework_schema::to_bounded_value(&snapshot)?;
    stackpress_framework_schema::check_browser_json(&wire)?;
    let seed = wire
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let assets = format!("{}assets/start.js", snapshot.mount);
    let owner = Owner::new();
    let body = if csr {
        String::new()
    } else {
        owner.with(||view!{<generated_admin_shared::crud::Admin snapshot transport=Callback::new(|_|{})/>}.to_html())
    };
    owner.cleanup();
    Ok(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Generated admin</title><style>{CORE_STYLES}{ADVANCED_STYLES}{BREADTH_STYLES}*{{box-sizing:border-box}}body{{font:16px system-ui;margin:1rem auto;padding:1rem;max-width:70rem}}section,fieldset{{min-width:0}}.admin-actions{{display:flex;flex-wrap:wrap;align-items:center;gap:.75rem;margin-block:1rem}}nav{{display:flex;flex-wrap:wrap;gap:1rem;margin-block:1rem}}label{{display:block}}input,select,textarea{{max-width:100%;padding:.4rem}}.sp-formatted input[type=text]{{width:42ch;max-width:100%}}.sp-formatted .sp-edit>p[id$=\"-help\"]{{display:none}}.sp-formatted:focus-within .sp-edit>p[id$=\"-help\"]{{display:block}}#import-file-name{{overflow-wrap:anywhere}}.admin-table-scroll{{max-width:100%;overflow-x:auto}}.table-scroll-hint{{display:none}}.import-preview-scroll{{max-width:100%;overflow:auto}}#import-preview-table{{border-collapse:collapse;min-width:32rem}}#import-preview-table th,#import-preview-table td{{padding:.5rem;border:1px solid #bbb;vertical-align:top}}#import-preview-table pre{{max-width:24rem}}pre{{white-space:pre-wrap;overflow-wrap:anywhere}}@media(max-width:40rem){{.table-scroll-hint{{display:block}}#import-preview-table{{min-width:0;width:100%}}#import-preview-table thead{{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}}#import-preview-table tbody,#import-preview-table tr,#import-preview-table td,#import-preview-table th[scope=row]{{display:block;width:100%}}#import-preview-table td::before{{content:attr(data-label);display:block;font-weight:bold}}#import-preview-table pre{{max-width:100%}}}}.sp-role{{margin-block:1rem}}.sp-role-value{{display:flex;flex-wrap:wrap;gap:.5rem}}#operation-errors:empty{{display:none}}</style></head><body><div id=\"app\">{body}</div><script id=\"admin-seed\" type=\"application/json\">{seed}</script><script type=\"module\" src=\"{assets}\"></script></body></html>"
    ))
}
