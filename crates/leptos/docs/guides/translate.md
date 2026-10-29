# Translate and pluralize phrases

Keep translation as checked data, then choose text slots or deliberate view slots. Start
without a reactive root to verify the dictionary and integer grammar.

```rust
use stackpress_leptos::language::Language;
use std::collections::BTreeMap;

fn main() -> Result<(), &'static str> {
    let language = Language::new(
        "fr-CA".into(),
        BTreeMap::from([
            ("%s profile".into(), "%s profil".into()),
            ("%s profiles".into(), "%s profils".into()),
        ]),
        "%s".into(),
    )?;
    assert_eq!(
        language.plural_text(0, "%s profile", "%s profiles", &["0".into()]),
        "0 profil"
    );
    assert_eq!(
        language.plural_text(2, "%s profile", "%s profiles", &["2".into()]),
        "2 profils"
    );
    assert_eq!(
        language.text("%s / %s", &["0".into(), "false".into()]),
        "0 / false"
    );
    Ok(())
}
```

Count selects the source phrase before dictionary lookup. It does not automatically fill a
slot, so pass the count text explicitly. French includes zero and one in `One`; English and
the current fallback use one only. Regional keys use the base language. This is a `u64`
one/other policy, not full CLDR or fractional grammar.

## Add scoped reactivity

Inside a component, construct `LanguageContext`, provide it with `LanguageProvider`, and use
`Translate` when a placeholder must contain a reusable view. This component's two deliberate
slots preserve a bold zero and plain false text:

```rust
use stackpress_leptos::{language::{Language, LanguageContext, LanguageProvider, Translate}, leptos::prelude::*};

#[component]
pub fn TranslatedSummary() -> impl IntoView {
    let language = LanguageContext::new(
        Language::new("en".into(), Default::default(), "%s".into())
            .expect("fixed valid dictionary"),
    );
    let slots = vec![
        ViewFn::from(|| view! { <strong>"0"</strong> }),
        ViewFn::from(|| "false"),
    ];
    view! {
        <LanguageProvider language>
            <Translate language phrase="%s / %s" slots/>
        </LanguageProvider>
    }
}
```

The phrase and fragments render as escaped text; only the explicit slots create views.
`change` replaces the whole checked language/map; no old entries leak into the new locale.
Use a reactive closure when rendering `LanguageContext::text` so changes are observed.

Missing slots become empty and excess slots are ignored. Exact whitespace, empty strings,
zero, and false text survive. No phrase is interpreted as raw HTML or executable code.
You can now choose text versus node interpolation deliberately; consult
[Language](../api/language.md) and [Translate](../api/translate.md) for limits and props.
