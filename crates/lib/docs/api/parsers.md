# Input parser API

Pure buffered boundary parsers exported from `stackpress_lib::data`. They return new stores and
perform no native reads. Source: [parsers.rs](../../src/data/parsers.rs).

```rust
use stackpress_lib::data::{
    parse_arg_string, parse_args, parse_body, parse_json_object, parse_multipart, parse_query,
};
use stackpress_lib::{ParseLimits, Result};

let limits = ParseLimits::default();
```

Examples run independently inside an async or synchronous function returning `Result<()>`, using the
imports and `limits` above. For a walkthrough, read [Parsing input](../guides/parsing-input.md).

Contents: [Shared policy](#1-shared-policy) · [Parser functions](#2-parser-functions)

## 1. Shared policy

Every parser applies the supplied byte/entry/depth budgets. The returned `Nest` retains that policy.
See [ParseLimits](data.md#1-limits-and-value-shapes) for defaults and node accounting.

Query, argv, and multipart text use the same scalar policy: exact `true`, `false`, `null`, JSON
numbers, and valid object/array JSON become values. Other text, including malformed JSON-like scalar
text and empty strings, stays text. Parsed JSON fields are not scalar-coerced again.

Bracket names accept `profile[name]`, numeric indices, and empty append brackets. They reject
unclosed brackets, stray closing brackets, suffix text after brackets, empty roots, and path limits.
Dots in a bracket-parser field name are literal; [Nest dot paths](data.md#4-paths) are a separate
spelling.

## 2. Parser functions

Choose the function matching the original input boundary. Parser errors are ordinary `Error` values,
not partial stores.

### 2.1. parse_query

`parse_query(text: &str, limits: &ParseLimits) -> Result<Nest>`

Decode a query or URL-encoded form. An optional leading `?` and empty `&` fragments are ignored.
Split each pair at its first `=`; a missing value becomes empty text. Decode names/values once, with
`+` as space. Later ordinary keys replace earlier ones; append brackets add values.

```rust
let data = parse_query("?name=Ada+Lovelace&tags[]=rust&tags[]=events", &limits)?;
assert_eq!(data.path_as::<String>("name")?, Some("Ada Lovelace".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Raw query text, without a URL. |
| `limits` | `&ParseLimits` | Input, decoded tree, pair-count, and depth budgets. |

**Returns** A new `Nest`. Empty input gives an empty store. Malformed percent escapes, decoded
non-UTF-8, invalid bracket paths, scalar traversal conflicts, or exceeded limits return an error. A
pair-count budget also counts replaced keys.

### 2.2. parse_args

`parse_args(args: &[String], skip: usize, limits: &ParseLimits) -> Result<Nest>`

Parse supplied argv without percent decoding. `--key=value` or `--key value` sets a field; the next
argument is consumed even if it begins with minus. ASCII alphanumeric grouped short flags set true.
Numeric arguments are positional values, including negative numbers. Bare `--` ends options; lone
`-` is ignored while options are enabled.

```rust
let argv = ["tool", "--token", "a&b=c+d%20", "-vq", "--", "-literal"].map(String::from);
let data = parse_args(&argv, 1, &limits)?;
assert_eq!(data.path_as::<String>("token")?, Some("a&b=c+d%20".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `args` | `&[String]` | Original ordered arguments; all their bytes count, including skipped items. |
| `skip` | `usize` | Number of leading items to ignore; beyond the end yields empty input. |
| `limits` | `&ParseLimits` | Aggregate bytes, scanned argument position, and output budgets. |

**Returns** A new `Nest` with positional fields `"0"`, `"1"`, etc. Missing long-option values,
invalid short flags/names/paths, conflicts, or limits return an error. Delimiters inside an argv
value remain payload.

### 2.3. parse_arg_string

`parse_arg_string(text: &str, limits: &ParseLimits) -> Result<Nest>`

Tokenize whitespace and single/double quotes, then call the argv parser with skip zero. Matching
quotes are removed; quoted empty arguments survive. Backslashes have no escape role, and no
command/environment expansion occurs.

```rust
let data = parse_arg_string(r#"--name "Ada Lovelace" --empty '' -v"#, &limits)?;
assert_eq!(data.path_as::<String>("empty")?, Some("".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Small command-like string, not shell source. |
| `limits` | `&ParseLimits` | Raw text, token count, and argv/output budgets. |

**Returns** A new `Nest`, or an error for unclosed quotes, argv grammar/path failures, or limits.
This tokenizer does not implement shell syntax.

### 2.4. parse_json_object

`parse_json_object(text: &str, limits: &ParseLimits) -> Result<Nest>`

Parse JSON with an object root, preserving JSON types and insertion order. Use this when the body
format is already known.

```rust
let data = parse_json_object(r#" {"active":false,"count":0,"nickname":null}"#, &limits)?;
assert_eq!(data.path_as::<bool>("active")?, Some(false));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | JSON text; JSON whitespace is permitted. |
| `limits` | `&ParseLimits` | Raw byte and parsed-tree budgets. |

**Returns** An object-root `Nest`. Malformed JSON retains its serde cause and code 400. Non-object
roots and tree/raw limits return errors; valid false, zero, null, and empty objects remain present.

### 2.5. parse_multipart

`parse_multipart(body: &[u8], boundary: &str, limits: &ParseLimits) -> Result<Nest>`

Parse buffered multipart form data with exact binary files. A `filename` parameter, even empty,
selects `Value::File`; missing file MIME defaults to `application/octet-stream`. Other named form
fields require UTF-8 and use scalar/bracket policy. Parts without a usable form-data name are
ignored but still count.

```rust
let body = b"--B\r\nContent-Disposition: form-data; name=\"count\"\r\n\r\n2\r\n--B--\r\n";
let data = parse_multipart(body, "B", &limits)?;
assert_eq!(data.path_as::<u64>("count")?, Some(2));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `body` | `&[u8]` | Complete buffer starting at the initial boundary, with CRLF framing and final closing boundary. |
| `boundary` | `&str` | Decoded boundary value without MIME quotes or leading `--`; 1–70 allowed ASCII bytes, no trailing space. |
| `limits` | `&ParseLimits` | Buffer/tree/part count and per-part header budgets. |

**Returns** A new `Nest`. Invalid boundary/framing/header/parameter syntax, duplicate normalized
part headers or parameters, truncation, text UTF-8 failure, path conflicts, and exceeded limits
return errors. Only an optional final CRLF may follow the closing boundary; preambles/epilogues and
streaming are unsupported.

### 2.6. parse_body

`parse_body(content_type: &str, body: &[u8], limits: &ParseLimits) -> Result<Nest>`

Dispatch using a case-insensitive MIME essence and parameter names. Supports `application/json`,
`text/json`, essences ending in `+json`, URL-encoded forms, and multipart with a boundary. Quoted
parameters and quoted-pair escapes are parsed; duplicate normalized parameters are rejected.

```rust
let data = parse_body(
    "Application/JSON; charset=\"UTF-8\"",
    br#"{"ok":true}"#,
    &limits,
)?;
assert_eq!(data.path_as::<bool>("ok")?, Some(true));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `content_type` | `&str` | Explicit MIME text; charset does not transcode bytes. |
| `body` | `&[u8]` | Complete buffered body. |
| `limits` | `&ParseLimits` | Forwarded parser policy. |

**Returns** A `Nest` from the selected parser. Missing multipart boundary, invalid MIME parameters,
underlying parser failures, and limits return errors. Non-multipart input must be UTF-8; an
unsupported media type with valid UTF-8 yields code 415.

## Related

[Parsing input guide](../guides/parsing-input.md) · [Data](data.md) · [Request](request.md) ·
[API index](README.md)
