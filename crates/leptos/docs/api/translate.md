# Translate

Reactive escaped phrase fragments with deliberate view slots. Source: [language.rs](../../src/language.rs).

```rust
use stackpress_leptos::{language::{Language, LanguageContext, Translate}, leptos::prelude::*};
pub fn counted() -> impl IntoView {
    let language = LanguageContext::new(
        Language::new("en".into(), Default::default(), "%s".into()).expect("fixed language"),
    );
    let count = RwSignal::new(2_u64);
    let slots = vec![ViewFn::from(move || count.get().to_string())];
    view! { <Translate language phrase="%s profiles" one="%s profile" count slots/> }
}
fn props_type(_props: stackpress_leptos::language::TranslateProps) {}
```

## Props

`Translate` can receive a context directly without a provider, as shown above.

| Prop | Type | Default / meaning |
| --- | --- | --- |
| `language` | `LanguageContext` | Required explicit root-owned language. |
| `phrase` | `String` via Into | Required source phrase; Other form if `one` is present. |
| `one` | optional `String` via Into | Absent; source One form enabling counted selection. |
| `count` | `Signal<u64>` via Into | Zero; selects grammar only when `one` is supplied. |
| `slots` | `Vec<ViewFn>` | Empty; reusable deliberate views in placeholder order. |

**Returns** `impl IntoView` that recomposes translated text fragments and view slots when
language/count change. Missing slots are empty; extra slots are ignored. Count is not an
implicit slot; the example supplies its text through a ViewFn. Whitespace and falsy text
survive. Phrase fragments are escaped, and no dictionary text becomes raw HTML.

The public generated `TranslateProps` is Leptos's props/builder surface for these same fields.
Views belong to the active Owner and must not retain a native invocation or publish into a
disposed root. This component uses the same limited integer plural policy as Language.

Related: [Language](language.md), [LanguageContext](language-context.md), [translation guide](../guides/translate.md).
