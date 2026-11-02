# Render source from build templates

Use a template when a build tool emits repeated source declarations. Keep language
validation and metadata decisions in the generator; let a readable template own
the source structure. This guide assumes an existing crate depending on
`stackpress-lib` and `serde_json` and a function returning `TemplateResult<()>`.

## Author the structural template

Create `templates/contract.rs.tpl` beside the generator module:

```text
pub struct [%model%] {
[%#@:fields field%]    pub [%field.member%]: [%field.ty%],
[%/@:fields%]}
```

The custom delimiters leave Rust braces unchanged. Whitespace between the loop
tags belongs to the repeated field; the engine does not trim it.

## Compile and render

First validate/normalize identifiers and types with your generator's existing
rules. This example uses fixed checked atoms to keep that boundary visible.

```rust
use stackpress_lib::template::{TemplateEngine, TemplateOptions};
use serde_json::json;

let engine = TemplateEngine::new(TemplateOptions {
    delimiters: ["[%".into(), "%]".into()],
    ..Default::default()
})?;
let template = engine.compile(include_str!("templates/contract.rs.tpl"))?;
let source = template.render(&json!({
    "model": "Profile",
    "fields": [{"member": "id", "ty": "String"}]
}))?;
assert_eq!(source, "pub struct Profile {\n    pub id: String,\n}\n");
```

Reuse the compiled template for other models. Pass the resulting text to your
existing output plan/writer; rendering itself performs no I/O or registration.
For embedded strings, encode a Rust literal in your generation layer before
passing it as data. JSON escaping does not substitute for Rust literal encoding.

## Compose a checked fragment

Short expressions can use scalar pairs after their inputs have been checked:

```rust
use stackpress_lib::TemplateEngine;

let snippets = TemplateEngine::default();
let ty = snippets.render_pairs("Option<{{inner}}>", &[("inner", "String")])?;
assert_eq!(ty, "Option<String>");
```

Insert the returned fragment as data in a later template. It is never reparsed,
even when its text contains `{{...}}` or `[%...%]`. Do not accept unchecked schema
strings as code fragments. Shape repeated declarations and branches in the
template instead of wrapping an old formatted function in a single source slot.

## Handle errors and context size

Missing paths and wrong types return `TemplateError`; an unselected conditional
branch does not look up its values. Branch syntax is still checked at compile time.
Inspect `kind()` and `span()` when reporting which template failed, and preserve
the original cause in your generator's error type.

Large codegen contexts may need `TemplateOptions.limits.max_data_bytes` raised
above the default one MiB because encoded field keys and repeated fragments add
JSON overhead. Keep your existing artifact/output-plan limit unchanged and test
both sides of its boundary. The engine separately checks its own output budget.

See the [template API](../api/template.md) for complete syntax, helpers, scope,
defaults and the trusted compiled-helper boundary.
