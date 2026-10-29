//! Deliberately hostile examples, clearly separated from ordinary product fixtures.
use leptos::prelude::*;
use stackpress_leptos::{
    components::{FieldControl, Textarea},
    rich::{RichContent, SafeHtml},
};

/// Security corpus: RCDATA breakout, entities, quotes, Unicode and leading newline.
pub const TEXTAREA_PAYLOAD: &str = "\n</textarea><script>globalThis.pwned=1</script>&\"'\u{2028}";

/// One corpus exercises the exact same sanitizer on native SSR and actual WASM.
pub const CORPUS: &[&str] = &[
    "<p onclick='globalThis.pwned=1'>Safe <strong>content</strong></p><script>globalThis.pwned=1</script>",
    "<a href='jav&#x61;script:alert(1)'>Rejected encoded JavaScript</a> <a href='//evil.invalid'>Rejected protocol-relative URL</a> <a href='https://example.invalid' title='safe'>Allowed HTTPS link</a>",
    "<svg><g onload='alert(1)'><foreignObject><img src='https://evil.invalid' onerror='alert(1)'></foreignObject></g></svg>",
    "<math><mtext><table><mglyph><style><!--</style><img title='--><img src=x onerror=alert(1)>'>",
    "<style>body{display:none}</style><form><input autofocus><iframe src='https://evil.invalid'></iframe></form><p>Visible fallback</p>",
    "<a href='data:text/html,evil'>Rejected data URL</a> <a href='/local'>Rejected relative URL</a> <p>&lt;script&gt;literal&lt;/script&gt;</p>",
];

/// Keep controlled textarea escaping and sanitizer samples in one labelled region.
#[component]
pub fn SafetySamples(
    /// Verbatim native edit; the default hostile seed is owned by the caller.
    initial: String,
    /// The root retains ownership of its Resource and replay identity.
    children: Children,
) -> impl IntoView {
    // Adopt the exact pre-hydration property; the root still owns Resource replay.
    let notes = RwSignal::new(initial);
    // Publish only on explicit inspection so early edits do not repair SSR text.
    let inspected = RwSignal::new(String::new());
    view! {
        <section id="safety-samples" aria-labelledby="safety-title">
            <h2 id="safety-title">"Safety samples"</h2>
            <p>"Deliberately hostile text tests escaping and sanitization. It is displayed as data; rejected links cannot navigate."</p>
            {children()}
            <form id="safety-form" on:submit=move |event| event.prevent_default()>
                <FieldControl for_id="safety-notes" label="Textarea escaping sample" help="Literal markup, quotes, Unicode and a leading newline">
                    <Textarea id="safety-notes" name="safety-notes" value=notes described_by="safety-notes-help safety-notes-error"/>
                </FieldControl>
                <button id="safety-inspect" type="button" on:click=move |_| inspected.set(notes.get())>"Inspect safety draft"</button>
                <output id="safety-state">{move || inspected.get()}</output>
            </form>
            <h3>"Sanitized rich content"</h3>
            {CORPUS.iter().enumerate().map(|(index,input)| view! { <section data-corpus=index><RichContent content=SafeHtml::html(input).expect("bounded fixed corpus")/></section> }).collect_view()}
            <RichContent content=SafeHtml::markdown("**Markdown** with `safe code` and <img src='https://evil.invalid'>").expect("bounded fixed Markdown")/>
        </section>
    }
}
