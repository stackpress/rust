# Language helpers and prefix dispatch

Enable `language` on a native graph. `language::register(&mut ServerBuilder) -> Result<()>` queues Config and Listen; only a truthy source language section publishes Service and the request hook. No process-global active language is stored.

## Configuration and request selection

Config has key (default locale), locale (default en_US) and an insertion-ordered languages map. Service.config owns that immutable per-host configuration. `locales() -> Vec<&str>` preserves keys; `language(name) -> Option<Value>` copies label/translations, returning None for missing/falsy entries. Missing/nonobject translations become an empty object in that copy.

`Service::load(req, defaults) -> Language<'_>` borrows configuration, taking a nonempty string cookie or the supplied fallback without validating membership. Language.locale is a per-instance owned string. Its helpers are:

| Method | Return / effect |
| --- | --- |
| label() | serde_json::Value; truthy configured label, otherwise current locale string. |
| translations() | Option<&Map<String, Value>>; None for missing/nonobject translations. |
| save(res) | Result<&Self>; stages source unscoped locale cookie. |
| update(locale, res) | Result<&Self>; changes locale before staging that cookie; staging failure retains the selection change. |
| translate(phrase, variables) | String; truthy string translation or phrase, then first %s replacement per Scalar using source dollar replacement tokens. |

Scalar represents String(&str), Number(f64), Boolean(bool), Null and Undefined. Source number/string conversion and first-replacement semantics are retained; this is separate from a preference-registry locale normalization policy.

<!-- example: full -->
```rust
use stackpress_framework::language::{Config, Scalar, Service};
use stackpress_lib::Request;

pub fn translation() {
    let mut config = Config::default();
    config.languages.insert(
        "en_US".into(),
        serde_json::json!({
            "label":"English", "translations":{"Hello %s":"Welcome %s"}
        }),
    );
    let service = Service { config };
    let language = service.load(&Request::new(), "en_US");
    assert_eq!(
        language.translate("Hello %s", &[Scalar::String("Ada")]),
        "Welcome Ada"
    );
}
```

The helper operates without a plugin or host. For HTTP request selection, the registered `request` handler first stages a recognized data locale, then lets a recognized URL prefix override it.

## Nested dispatch boundary

For a recognized prefix, the hook strips it, reconstructs origin + path without URL query/fragment, and raw-emits the method/path event using the same response. Separate query/post/data/header/cookie/body values survive the buffered child projection. The adapter must finish loading native input first; no Request or reader is cloned or replayed.

Child Abort remains local and the outer hook returns Continue, matching upstream. Unknown prefixes do not dispatch. Independent invocations retain their own locale/cookies and response state while suspended. Evidence: [language source](../../src/native/language.rs), [source/helper/reader tests](../../tests/cases/language.rs), [concurrent locale test](../../tests/cases/closeout.rs), [Request compile-fail case](../../tests/ui/request_clone.rs).
