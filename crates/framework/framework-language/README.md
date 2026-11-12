# stackpress-framework-language

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Validate an explicit locale registry and direct aliases. Stackpress view preferences use these pure values; cookies, routing and hosts belong to other crates.

```rust
use std::collections::BTreeMap;
use stackpress_framework_language::LocaleRegistry;
let locales = LocaleRegistry::new(vec!["en".into()], "en".into(), BTreeMap::new())?;
assert_eq!(locales.default_locale(), "en");
# Ok::<(), stackpress_framework_language::PreferenceError>(())
```

The configured fallback must be registered. Locale selection does not infer a region or inspect a request.

Licence: [Stackpress License](LICENSE).
