# Language and PluralCategory

Checked dictionary, literal placeholders, and integer grammar. Source: [language.rs](../../src/language.rs).

```rust
use stackpress_leptos::language::{Language, PluralCategory};
fn main() -> Result<(), &'static str> {
    let language = Language::new("en".into(), Default::default(), "%s".into())?;
    assert_eq!(language.locale(), "en");
    assert_eq!(language.plural_category(0), PluralCategory::Other);
    assert_eq!(language.plural_phrase(1, "one", "many"), "one");
    assert_eq!(language.phrase("Missing"), "Missing");
    assert_eq!(language.template("%s / %s"), vec!["", " / ", ""]);
    assert_eq!(
        language.text("%s / %s", &["0".into(), "false".into()]),
        "0 / false"
    );
    assert_eq!(
        language.plural_text(2, "%s item", "%s items", &["2".into()]),
        "2 items"
    );
    Ok(())
}
```

Contents: [Construction](#1-construction) · [Lookup](#2-lookup) · [Plural grammar](#3-plural-grammar)

## 1. Construction

`Language::new(locale: String, phrases: BTreeMap<String, String>, placeholder: String)` validates
before returning language data. It does not fetch a dictionary or normalize locale spelling.

| Parameter | Limits |
| --- | --- |
| `locale` | 1–64 bytes, ASCII letters/digits/underscore/hyphen only. |
| `phrases` | At most 256 pairs, each key/value ≤4,096 bytes, summed key/value bytes ≤64 KiB. |
| `placeholder` | 1–32 bytes, treated as a literal separator, not a regex. |

**Returns** `Result<Language, &'static str>` with an invalid-locale, invalid-placeholder, or map
budget error. The type is Clone, Debug, PartialEq/Eq, Serialize/Deserialize; decoding rejects
unknown fields and runs the same constructor. Its standalone locale key syntax is broader
than `PageContract.locale`; page callers must satisfy both.

## 2. Lookup

Use these methods for text. No result is interpreted as HTML or executable code.

| Method | Parameters | Behavior |
| --- | --- | --- |
| `locale(&self)` | None | Borrows exact locale as `&str`. |
| `phrase(&self, phrase)` | `&str` | Borrows translated text or original source phrase if absent. |
| `template(&self, phrase)` | `&str` | Looks up and splits around literal placeholders into `Vec<String>`. |
| `text(&self, phrase, values)` | `&str`, `&[String]` | Inserts text values in slot order, preserving exact whitespace and empty/zero/false strings. |

**Returns** The types above; `text` returns `String`. Missing values become empty and extras are
ignored. Escaping happens when strings are rendered through Leptos, not by mutating the
returned plain string. Node values cannot be passed to the text API.

## 3. Plural grammar

`PluralCategory` is Copy/Clone, Debug, PartialEq/Eq with `One` and `Other`. English uses One
only for count 1; French uses One for 0 and 1. Base language matching is case insensitive and
splits regional keys on hyphen/underscore. Other locales use the current English fallback.

| Method | Parameters | Behavior |
| --- | --- | --- |
| `plural_category(&self, count)` | `u64` | Selects `PluralCategory`. |
| `plural_phrase(&self, count, one, other)` | `u64`, two `&str` | Borrows the selected source phrase, before translation. |
| `plural_text(&self, count, one, other, values)` | `u64`, two `&str`, `&[String]` | Selects source phrase, then translates and fills text slots. |

**Returns** Category, borrowed phrase, or formatted String respectively. Count chooses grammar
only; it is not automatically inserted as a slot. Fractional counts and additional CLDR
categories are outside the current policy.

Related: [translation guide](../guides/translate.md), [Translate](translate.md).
