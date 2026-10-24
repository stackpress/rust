# Parse buffered input

Input syntax should not silently change field identity or lose false, zero, or null. The parsers
return a bounded `Nest`, letting an operation inspect input before applying it to a request.

## Parse and inspect a query

Inside a function returning `stackpress_lib::Result<()>`, parse a query with nested fields and
repeated append values:

```rust
use stackpress_lib::data::parse_query;
use stackpress_lib::{ParseLimits, Value};

let limits = ParseLimits::default();
let data = parse_query(
    "?profile[name]=Ada+Lovelace&tags[]=rust&tags[]=events&active=false",
    &limits,
)?;
assert_eq!(
    data.path_as::<String>("profile.name")?,
    Some("Ada Lovelace".into())
);
assert_eq!(data.path("active").and_then(Value::as_bool), Some(false));
assert_eq!(data.path_as::<Vec<String>>("tags")?.unwrap().len(), 2);
```

The parser decodes names and values once, treats `+` as space, and interprets brackets as path
steps. Repeating an ordinary key replaces its value; `[]` appends. Exact booleans/null, finite JSON
numbers, and valid object/array JSON become typed values; malformed JSON-like scalar text stays
text.

## Keep command arguments literal

Use argv directly when it is already available. Do not join it into a query string:

```rust
use stackpress_lib::data::parse_args;

let args = ["tool", "--token", "a&b=c+d%20", "-v"].map(String::from);
let data = parse_args(&args, 1, &limits)?;
assert_eq!(data.path_as::<String>("token")?, Some("a&b=c+d%20".into()));
assert_eq!(data.path_as::<bool>("v")?, Some(true));
```

The value remains one string, including its delimiters. Long options consume an inline `=value` or
the next argument, grouped short flags become true, and `--` switches later items to positional
fields. `parse_arg_string` adds only whitespace/single/double quote grouping; it performs no shell
expansion or backslash escape processing.

## Select a body parser

Call `parse_body` with an explicit content type and buffered bytes:

```rust
use stackpress_lib::data::parse_body;

let post = parse_body(
    "Application/JSON; charset=UTF-8",
    br#"{"active":false}"#,
    &limits,
)?;
assert_eq!(post.path_as::<bool>("active")?, Some(false));
```

JSON requires an object root. URL-encoded bodies use the query policy. Multipart requires a declared
boundary and complete CRLF framing; named files retain exact bytes and metadata, including empty
files. Unsupported body media types return an error. Request construction does not call these
parsers on a buffered body automatically.

## Pick workload budgets

Defaults allow 1 MiB input/payload bytes, depth 64, 10,000 entries/nodes or parts, and 8 KiB headers
per multipart part. Root and container nodes count too, so an entry budget is not simply a count of
top-level fields. A chosen depth above 64 does not lift the implementation's depth ceiling.

Pass the same deliberate `ParseLimits` to parsing and the request builder when the workload differs.
Returned errors expose no partially built `Nest`; strict percent/UTF-8 handling also prevents
silently repaired query fields. Cookie input has its own permissive decoding policy described in
[Sessions and cookies](sessions-and-cookies.md).

You can now select the parser that matches the original boundary without reinterpreting argv as
query syntax. See [parser API](../api/parsers.md) for full grammar, error conditions, and multipart
constraints, and [data API](../api/data.md) for path and value budgets.
