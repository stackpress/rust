# Build-time templates

Use `stackpress_lib::template` for strict text rendering in a build tool. It has no
file/network access or Rust-language policy. Identifiers, types, literals and
source imports must be checked by the generator before they enter template data.

```rust
use stackpress_lib::template::TemplateEngine;
use serde_json::json;

let engine = TemplateEngine::default();
let template = engine.compile("{{#@:names name}}{{name}};{{/@:names}}")?;
let source = template.render(&json!({"names": ["id", "title"]}))?;
assert_eq!(source, "id;title;");
```

Examples omit a surrounding function returning `TemplateResult<()>`. For a task
walkthrough, read [rendering source templates](../guides/render-build-templates.md).

## Contents

- [Engines and compiled templates](#engines-and-compiled-templates)
- [Grammar and value lookup](#grammar-and-value-lookup)
- [Inline helpers](#inline-helpers)
- [Options and errors](#options-and-errors)

## Engines and compiled templates

`TemplateEngine::default()` uses `{{` and `}}` and built-in helpers. Explicit
`TemplateEngine::new(TemplateOptions)` validates delimiters and limits.

| Method | Parameters | Use |
| --- | --- | --- |
| `TemplateEngine::compile` | `source: &str` | Own the parsed source and helper snapshot; check syntax, helpers and source/depth limits. |
| `TemplateEngine::render` | `source: &str`, `data: &serde_json::Value` | Compile once and render once; compile errors precede data errors. |
| `TemplateEngine::render_pairs` | `source: &str`, `pairs: &[(&str, &str)]` | Same, with string-valued object construction after byte/depth validation. Duplicate keys fail. |
| `TemplateEngine::register_helper` | `name: &str`, `helper: TemplateHelper` | Register without changing already compiled templates; reject invalid/reserved/duplicate names. |
| `Template::render` | `data: &serde_json::Value` | Reuse immutable parsed structure with fresh output and budgets; root must be an object. |
| `Template::render_pairs` | `pairs: &[(&str, &str)]` | Same, with a bounded object of strings. |
| `template::render` | `source: &str`, `data: &serde_json::Value` | Default-engine one-call rendering. |

**Returns:** `new` returns `TemplateResult<TemplateEngine>`; `compile` returns
`TemplateResult<Template>`; the render operations return `TemplateResult<String>`.
`register_helper` returns `TemplateResult<&mut TemplateEngine>` for chaining.
Defaults are infallible; fallible methods return the categories described below.

The named types are also exported by the lib root and `stackpress`. The facade
exposes this module as `stackpress::template` and `stackpress::lib::template`.
`Template` and `TemplateEngine` are `Send + Sync`; a render never mutates their
source, options, registrations, or caller data. A failed render returns no partial
output. Trusted helpers are responsible for any effects or panic behavior.

For repeated scalar snippets, compile once and reuse `Template::render_pairs`:

```rust
use stackpress_lib::TemplateEngine;

let template = TemplateEngine::default().compile("Vec<{{inner}}>")?;
assert_eq!(template.render_pairs(&[("inner", "String")])?, "Vec<String>");
assert_eq!(template.render_pairs(&[("inner", "bool")])?, "Vec<bool>");
assert_eq!(stackpress_lib::template::render("{{name}}", &serde_json::json!({"name":"id"}))?, "id");
```

**Returns:** both render paths produce a fresh `TemplateResult<String>` each
time. A failed invocation does not exhaust a later invocation's budgets.

## Grammar and value lookup

| Form | Behavior |
| --- | --- |
| `{{path}}` | Strings verbatim; numbers and booleans as text. Missing/null/collections fail. |
| `{{#?:flag}}yes{{\|}}no{{/?:flag}}` | Boolean condition; optional else. |
| `{{#!:flag}}no{{\|}}yes{{/!:flag}}` | Boolean inversion; optional else. |
| `{{#@:items item key}}...{{/@:items}}` | Array/object iteration, with optional ordinary identifier aliases. No join argument or loop else. |
| `{{helper:path argument...}}` | Compiled inline helper; primary argument is a path; remaining arguments are strict paths or JSON scalar literals. |

ASCII whitespace around tag content is ignored; text whitespace is preserved
exactly. No body trimming, newline normalization, or standalone-tag stripping.
Double the opening delimiter to produce it literally: `{{{{name}}` becomes
`{{name}}`. Closing delimiters outside tags are literal text. Nested identical
block kinds/paths are supported. Unknown helpers and malformed syntax fail at
compile time even in an unselected branch. Value lookups/helper calls occur only
in visited branches and loop bodies.

Paths use ASCII identifier roots, then dotted identifier or canonical decimal
segments, such as `field.name` and `items.0.name`. There is no arbitrary expression,
bracket access, method call, or quoted key syntax. Normalize unusual object keys
in a template-specific view model. On arrays, decimal segments index elements;
on objects, they are exact keys.

Lookup checks current aliases, enclosing aliases, then the root. `@root.path`
selects the original object. Item members are explicit, never auto-spread.
Loops always provide `@this`, `@key`, `@index`, `@first`, `@last`, `@odd`, and
`@even`. Array keys/indices are zero-based; object keys are strings and object
indices are zero-based entry positions. Objects retain JSON map insertion order.

## Inline helpers

`TemplateHelper` is `fn(&[&serde_json::Value]) -> TemplateResult<String>`.
Return text or `TemplateError::helper("static context")`. Arguments resolve in
source order; missing paths always fail, including fallback arguments.

| Built-in | Arguments and result |
| --- | --- |
| `A:path`, `a:path` | Exactly one string; Unicode uppercase/lowercase mapping. |
| `.:path count [suffix]` | String, nonnegative integer character count, optional string suffix. Retains Unicode scalar values, appending suffix only when truncated. Zero clips nonempty input entirely. |
| `\|:path alternatives...` | First non-null/non-false/nonzero/nonempty-string scalar, or empty text if none. Every collection argument fails. This explicit helper alone applies fallback truthiness. |

JSON string arguments use double quotes and JSON escapes. A closing delimiter
ends the tag even inside such an argument; supply that string through data.
Custom helper names are ASCII identifiers; built-in/block names cannot be replaced.
Helpers receive no engine or template body. Returned text and all inserted data
remain inert, even if they contain delimiters. Helpers are compiled trusted code;
their allocations, effects, panics and runtime cannot be sandboxed by these limits.

Register a named function, render once, and inspect a strict lookup error:

```rust
use stackpress_lib::{TemplateEngine, TemplateError, TemplateErrorKind, TemplateResult};
use serde_json::{Value, json};

fn shout(arguments: &[&Value]) -> TemplateResult<String> {
    let [value] = arguments else {
        return Err(TemplateError::helper("shout requires one argument"));
    };
    let text = value.as_str().ok_or_else(|| TemplateError::helper("shout requires text"))?;
    Ok(text.to_uppercase())
}

let mut engine = TemplateEngine::default();
engine.register_helper("shout", shout)?;
assert_eq!(engine.render("{{shout:title}}", &json!({"title":"profile"}))?, "PROFILE");
let error = engine.render("{{missing}}", &json!({})).expect_err("missing path must fail");
assert_eq!(error.kind(), TemplateErrorKind::MissingValue);
assert_eq!(error.span(), Some(0..11));
```

**Returns:** registration returns the mutable engine on success; `shout` returns
its text or a static-context helper error. `kind()` returns `TemplateErrorKind`,
and `span()` returns `Option<Range<usize>>` without retaining the rejected value.

## Options and errors

`TemplateOptions` has `delimiters: [String; 2]` and `limits: TemplateLimits`.
Delimiters must be distinct/nonempty, at most 16 UTF-8 bytes, without whitespace,
and neither may be a prefix of the other. Source templates can use `[%` / `%]`
to preserve embedded Rust `format!` braces exactly.

| Limit field | Default |
| --- | --- |
| `max_template_bytes` | 1 MiB UTF-8 source, checked before copying. |
| `max_data_bytes` | 1 MiB compact JSON bytes, including escapes/keys/punctuation, counted without a serialized buffer or root clone. |
| `max_output_bytes` | 1 MiB accumulated UTF-8 output, checked before each append. |
| `max_nesting`, `max_data_depth` | 64 each; first block/root value is depth one. Each setting must be `1..=128`. |
| `max_iterations` | 100,000 entries aggregated across nested loops, even for empty-output bodies. |
| `max_steps` | 1,000,000 node visits, path-segment lookups and helper calls aggregated per render. |

All limits are positive; there is no unlimited sentinel. Byte/work budgets can be
raised explicitly. For codegen, account for JSON context overhead independently
from the output artifact's existing cap; raising data limits does not raise output
limits. Limits do not measure process memory, helper allocations or elapsed time.

`TemplateError` implements `std::error::Error`. `kind()` returns `InvalidOptions`,
`Syntax`, `UnknownHelper`, `DuplicateHelper`, `InvalidData`, `MissingValue`,
`InvalidType`, `Limit`, or `Helper`. `span()` returns optional template byte offsets
as `Range<usize>`. Display/Debug retain fixed context and offsets, not source/data.
The first visited failure wins. A helper's static message is caller-owned context.

Public declarations and executable examples live in [source](../../src/template/mod.rs).
