# LanguageContext and LanguageProvider

Root-scoped reactive dictionary replacement. Source: [language.rs](../../src/language.rs).

```rust
use stackpress_leptos::{language::{Language, LanguageContext, LanguageProvider, use_language}, leptos::prelude::*};
#[component]
pub fn LocaleRoot() -> impl IntoView {
    let language = LanguageContext::new(
        Language::new("en".into(), Default::default(), "%s".into()).expect("fixed language"),
    );
    view! { <LanguageProvider language><LocaleLabel/></LanguageProvider> }
}
#[component]
fn LocaleLabel() -> impl IntoView {
    let language = use_language().expect("ancestor provider");
    move || language.text("Profiles", &[])
}
fn provider_props_type(_props: stackpress_leptos::language::LanguageProviderProps) {}

fn replace_language(context: LanguageContext, next: Language) -> String {
    context.change(next);
    let locale = context.get().locale().to_owned();
    let count = context.plural_text(2, "%s profile", "%s profiles", &["2".into()]);
    format!("{locale}: {count}")
}
```

## LanguageContext

The Copy/Clone handle owns one signal in the current Owner. Copying aliases it; it does not
create a new locale root.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `new` | `Language` | Initialize explicit checked state. |
| `get(self)` | None | Reactive read, cloning `Language`. |
| `change(self, language)` | `Language` | Replaces the locale and entire dictionary atomically. |
| `text(self, phrase, values)` | `&str`, `&[String]` | Reactively translates using current language. |
| `plural_text(self, count, one, other, values)` | `u64`, two `&str`, `&[String]` | Reactively selects grammar and translates from the same language. |
| `use_language()` | None | Looks up nearest provided context. |

**Returns** `new` returns Self, `get` returns Language, `change` returns unit, and text methods
return String. `use_language` returns `Result<LanguageContext, &'static str>` with
`Translations require LanguageProvider` when absent. A reactive consumer must call reads inside
its tracking computation/closure. No fetch, persistence, or implicit dictionary merge occurs.

## LanguageProvider

Use with Leptos `view!`; it provides context before evaluating children.

| Prop | Type | Meaning |
| --- | --- | --- |
| `language` | `LanguageContext` | Required checked root handle. |
| `children` | `Children` | Required consumers. |

**Returns** `impl IntoView`, without an extra DOM wrapper. The public generated
`LanguageProviderProps` is the Leptos props/builder surface for these same fields.
Keep signal access in the owning root's UI context; marshal host completions there first.

Related: [Language](language.md), [Translate](translate.md), [root lifetime](../../../framework/framework-view/docs/leptos-concepts.md#client-state-belongs-to-one-root).
