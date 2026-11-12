# Migrating from 0.1 to 0.2

0.2.0 is the beta release of the 23-crate Stackpress family. It uses a normal Cargo
version, without `-beta`. The supported series is 0.2.x; 0.1.x users should upgrade.
The API may still change before 1.0. Rust 1.96 and edition 2024 are unchanged.

Update every direct Stackpress dependency together, including build and dev
requirements. Cargo's `"0.1"` range does not select `0.2.0`.

Before:

```toml
[dependencies]
stackpress = "0.1"
stackpress-lib = "0.1"
```

After:

```toml
[dependencies]
stackpress = "0.2"
stackpress-lib = "0.2"
```

Keep your existing features. Regenerate application models, stores, admin UI and
aggregate clients using your application's generator, then run its freshness check.
Do not edit managed Rust files by hand. The changes below also affect paths
re-exported by the `stackpress` facade.

## Object entries can fail

`Nest::entries()` now returns `stackpress_lib::Result<&DataMap<String, Value>>`.
Propagate the error from a fallible caller, or handle it explicitly.

Before:

```rust
use stackpress_lib::Nest;
fn object_len(data: &Nest) -> usize {
    data.entries().len()
}
```

After:

```rust
use stackpress_lib::{Nest, Result};
fn object_len(data: &Nest) -> Result<usize> {
    Ok(data.entries()?.len())
}
```

Use `data.len()` if only the count is needed.

## Editor display can fail

`EditPolicy::display` now returns `Result<String, EditError>`. This applies to
custom policies and the decimal, phone and temporal policies.

Before:

```rust
use stackpress_leptos::format::{DisplayLocale, edit::EditPolicy};
fn label<P: EditPolicy>(policy: &P, value: &P::Value) -> String {
    policy.display(value, DisplayLocale::English)
}
```

After:

```rust
use stackpress_leptos::format::{DisplayLocale, edit::{EditError, EditPolicy}};
fn label<P: EditPolicy>(policy: &P, value: &P::Value) -> Result<String, EditError> {
    policy.display(value, DisplayLocale::English)
}
```

Inside an existing custom `impl EditPolicy` whose `Value` is `String`, change the
method like this; retain the other required methods.

Before:

```rust
fn display(&self, value: &String, _locale: DisplayLocale) -> String {
    value.clone()
}
```

After:

```rust
fn display(&self, value: &String, _locale: DisplayLocale) -> Result<String, EditError> {
    Ok(value.clone())
}
```

Return an `EditError` when presentation fails. The editor prepares display before
publishing a changed value. Cancel and locale actions keep the draft and expose the
existing error state if display fails.

## Store DDL and relation preparation can fail

`SourceStore::create` now returns `Result<Create, Self::Error>`. Update custom
implementations as well as callers. For a wrapper around a `NativeStore`, the method
inside your existing `SourceStore` implementation whose associated `Error` is
`ActionError` changes as follows.

Before:

```rust
fn create(&self) -> stackpress_sql::Create {
    self.inner.create()
}
```

After:

```rust
fn create(&self) -> Result<stackpress_sql::Create, Self::Error> {
    self.inner.create()
}
```

A custom implementation that constructs valid DDL directly should wrap it in `Ok`;
return its associated error for invalid metadata.

`NativeStore::create`, `paths`, `selectors` and `joins` now return
`stackpress_framework_sql::source::SourceResult<T>` rather than `T`.
In an existing function returning `SourceResult`, propagate each preparation failure
before execution.

Before:

```rust
let ddl = store.create();
let paths = store.paths("application.profile.name");
let selectors = store.selectors(&query.columns);
let joins = store.joins(&query);
```

After:

```rust
let ddl = store.create()?;
let paths = store.paths("application.profile.name")?;
let selectors = store.selectors(&query.columns)?;
let joins = store.joins(&query)?;
```

Chained DDL preparation changes from `store.create().definition()?` to
`store.create()?.definition()?`. If your caller combines `ActionError` and SQL
errors, return a suitable outer error and convert both. Malformed local/foreign
relation keys and missing unique columns now produce errors before IO. Existing
unknown-path behavior remains.

## Generated aggregate definition uses the core error type

Regenerate aggregate clients. Their public `definition()` still returns table
definitions, but its error is now `stackpress_lib::Error`, wrapping store preparation
and SQL errors. Update wrappers that spell the old result type.

Before:

```rust
fn definitions() -> stackpress_sql::Result<Vec<stackpress_sql::TableDefinition>> {
    generated::client::framework::definition()
}
```

After:

```rust
fn definitions() -> stackpress_lib::Result<Vec<stackpress_sql::TableDefinition>> {
    generated::client::framework::definition()
}
```

Here `generated` is your application's generated module. Code that matches a SQL
error directly must now handle the core error and its retained source instead.

## CSV uploads preserve bytes

`Upload.csv` is `CsvBytes`. Convert owned or borrowed UTF-8 strings with `.into()`.

Before:

```rust
use stackpress_framework_admin::transfer::Upload;
let upload = Upload {
    csv: String::from("id,name\na,Ada"),
    decode_export_guards: false,
    confirm: false,
};
```

After:

```rust
use stackpress_framework_admin::transfer::Upload;
let upload = Upload {
    csv: String::from("id,name\na,Ada").into(),
    decode_export_guards: false,
    confirm: false,
};
```

For uploaded files, preserve the original bytes through the host boundary.

Before:

```rust
let csv = String::from_utf8(file_bytes)?;
let upload = Upload { csv, decode_export_guards: false, confirm: false };
```

After:

```rust
let upload = Upload {
    csv: file_bytes.into(),
    decode_export_guards: false,
    confirm: false,
};
let original = upload.csv.as_bytes();
```

`CsvBytes` supports `as_bytes`, `AsRef<[u8]>`, `len`, `is_empty` and `clear`. It no
longer provides arbitrary `String` methods. Convert to text explicitly only when
your application needs text and has chosen how to handle decoding errors.

JSON deserialization accepts both shapes below, while serialization emits the
second shape. Update clients that read serialized `Upload` objects.

Before (still accepted as input):

```json
{"csv":"id\na","decode_export_guards":false,"confirm":false}
```

After:

```json
{"csv":[105,100,10,97],"decode_export_guards":false,"confirm":false}
```

API `views::management::import_csv` accepts `impl AsRef<[u8]>` instead of `&str`.
Ordinary string calls remain usable. Prefer bytes at upload boundaries. If a
callback requires the old concrete function-pointer type, retain it with a wrapper.

Before:

```rust
let rows = stackpress_framework_api::views::management::import_csv(text.as_str())?;
```

After:

```rust
let rows = stackpress_framework_api::views::management::import_csv(file_bytes.as_slice())?;
```

For example, replace a typed callback assignment like this.

Before:

```rust
use serde_json::Value;
use stackpress_framework_schema::SchemaResult;
let parser: fn(&str) -> SchemaResult<Vec<Value>> =
    stackpress_framework_api::views::management::import_csv;
```

After:

```rust
use serde_json::Value;
use stackpress_framework_schema::SchemaResult;
fn parse_text(text: &str) -> SchemaResult<Vec<Value>> {
    stackpress_framework_api::views::management::import_csv(text)
}
let parser: fn(&str) -> SchemaResult<Vec<Value>> = parse_text;
```

The shared importer now recognizes UTF-8/UTF-16 BOMs and comma/semicolon/tab
separators. Unmarked legacy data uses Windows-1252 or an Arabic-evidence Windows-1256
heuristic. That heuristic can be ambiguous; supply UTF-8 when the source encoding is
known. Raw and decoded sizes must both fit the configured limit. Export remains
UTF-8 comma CSV. Header rules, typed conversion, explicit confirmation and row
budgets still apply. The native admin handler preserves JSON row batches separately
from raw CSV bodies.

## Other changes to account for

Invalid admin object children and malformed source catalogs now return errors or
render the existing rejection message. Handle those failures instead of depending
on an unwind. Library code denies six panic-family Clippy lints, with named tested
constant-parser exceptions; application callback unwind handling is unchanged.

Generator file sync failures can stop installation before publication. Inspect the
returned error and follow the existing managed-output recovery process. Parent
folder sync after rename is best effort on Unix.

The standalone email TLS provider fix and G57 HTTP date conversion already shipped
in 0.1.0. No additional migration for them is introduced by this release.
See the [changelog](../CHANGELOG.md) for the release boundaries.
