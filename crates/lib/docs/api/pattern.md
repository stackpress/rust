# Pattern and method API

Explicit literal, token, regex, and method/path matching. Patterns do not inspect `Request` or open
endpoints. Source: [pattern.rs](../../src/pattern.rs).

```rust
use stackpress_lib::{EventPattern, Method, MethodPattern, Result};

let pattern = EventPattern::express("/profiles/:id", '/')?;
```

Method examples run independently inside a function returning `Result<()>` with these imports and
`pattern`. See [Events and routing](../guides/events-and-routing.md) for integration.

Contents: [Public values](#1-public-values) · [Method operations](#2-method-operations) ·
[Pattern construction](#3-pattern-construction) · [Pattern inspection](#4-pattern-inspection)

## 1. Public values

Matching produces data owned by one match. It is separate from request data and listener execution.

| Type / field | Meaning |
| --- | --- |
| `MatchData.args: Vec<Option<String>>` | Positional wildcard or regex groups. Unmatched optional regex groups remain `None`. |
| `MatchData.params: DataMap<String, String>` | Named captures, containing only present values. |
| `MethodPattern::Exact(Method)` | Match one normalized real verb. |
| `MethodPattern::Any` | Match an uppercase nonempty method token in a route event. |
| `Route.method: Option<String>` | Normalized exact method, or `None` for Any. |
| `Route.path: String` | Original contributed route path. |

`MatchData` implements `Clone`, `Debug`, `Default`, `Eq`, and `PartialEq`. `Method` implements
`Clone`, `Debug`, `Default`, `Eq`, and `PartialEq`; its default is GET. `EventPattern` has no public
`Clone`. `MethodPattern` and `Route` expose `Debug`.

`Method` and `MethodPattern` each expose `GET`, `HEAD`, `POST`, `PUT`, `DELETE`, `CONNECT`,
`OPTIONS`, `TRACE`, and `PATCH` constants. For example, `Method::POST` is a request value;
`MethodPattern::POST` is an exact matcher. Neither has an `ALL` constant.

## 2. Method operations

Use constants for standard verbs and checked construction for extensions. Raw event matching does
not normalize emitted text.

### 2.1. Method::new

`Method::new(method: &str) -> Result<Self>`

Validate and ASCII-uppercase a real method token.

```rust
assert_eq!(Method::new("propfind")?.as_str(), "PROPFIND");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `&str` | 1–128 ASCII token bytes: letters, digits, or `!#$%&'*+-.^_` plus backtick, vertical bar, and tilde. |

**Returns** A normalized `Method`; empty, oversized, whitespace, non-ASCII, or other invalid token
bytes return an input error.

### 2.2. Method::as_str

`as_str(&self) -> &str`

Borrow the normalized token for an event or adapter.

```rust
assert_eq!(Method::POST.as_str(), "POST");
```

**Returns** The borrowed uppercase token, without allocation or error.

### 2.3. MethodPattern::exact

`MethodPattern::exact(method: &str) -> Result<Self>`

Build an exact matcher for an extension or supplied verb.

```rust
let custom = MethodPattern::exact("propfind")?;
assert!(
    EventPattern::route(custom, "/files")?
        .captures("PROPFIND /files")?
        .is_some()
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `&str` | Same token contract as `Method::new`. |

**Returns** An `Exact(Method)` matcher; invalid method input returns an error.

## 3. Pattern construction

Pattern source is bounded to 8,192 bytes; regex compilation is additionally capped at 1 MiB compiled
size. Registration rejects empty/oversized stored source. Exact construction itself is infallible
and defers that registration check.

### 3.1. EventPattern::exact

`EventPattern::exact(source: &str) -> Self`

Store literal bytes; punctuation such as `*` and `:` has no dynamic meaning.

```rust
let literal = EventPattern::exact("profile:*:read");
assert!(literal.captures("profile:*:read")?.is_some());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&str` | Literal event spelling. |

**Returns** An owned literal pattern. Empty/oversized source is rejected when registering it, rather
than by this constructor.

### 3.2. EventPattern::express

`EventPattern::express(source: &str, separator: char) -> Result<Self>`

Compile a whole-event token pattern. `:name` captures one or more non-separator characters into
params; names use ASCII letters/digits/underscore/hyphen, excluding the separator. `*` captures a
nonempty segment; `**` captures zero or more characters across separators. Literal fragments are
regex-escaped. Wildcard text is split on the chosen separator into args, preserving empty edge
segments.

```rust
let audit = EventPattern::express("audit-**", '-')?;
assert_eq!(
    audit.captures("audit-one-two")?.unwrap().args,
    [Some("one".into()), Some("two".into())]
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&str` | Explicit token pattern, anchored over the whole event. |
| `separator` | `char` | One caller-selected character; no default argument. Control characters, colon, and star are rejected. |

**Returns** A compiled pattern. Empty parameter names, duplicate names, adjacent dynamic tokens,
star runs longer than two, invalid separators, source/compiled-size limits, and regex compilation
failures return errors.

### 3.3. EventPattern::regex

`EventPattern::regex(source: &str, case_insensitive: bool) -> Result<Self>`

Compile Rust regex syntax. Anchors are caller-controlled; only the first match/captures run for a
registration. Every capture group, including named groups, appears positionally in args; present
named groups also appear in params. Repeated matching has no global cursor.

```rust
let pattern = EventPattern::regex(r"^(?P<kind>profile)(?:-(\d+))?$", true)?;
assert_eq!(pattern.captures("PROFILE")?.unwrap().args[1], None);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&str` | Rust regex expression; lookaround/backreferences are unsupported. |
| `case_insensitive` | `bool` | Explicit case-insensitive compilation choice. |

**Returns** A compiled regex pattern, or a source/compiled-size/syntax error. Compiler errors retain
a cause with registration classification. JavaScript flags and global-match iteration are not part
of this API.

### 3.4. EventPattern::route

`EventPattern::route(method: MethodPattern, path: &str) -> Result<Self>`

Compile a `METHOD /path` event matcher with retained route metadata and `/` token syntax. Exact
literal routes are strict. Parameterized/wildcard paths and Any-method routes tolerate trailing
slashes. Any matches an uppercase token, not a fictional request verb.

```rust
let route = EventPattern::route(MethodPattern::GET, "/profiles/:id")?;
assert_eq!(
    route.captures("GET /profiles/42/")?.unwrap().params["id"],
    "42"
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `MethodPattern` | Exact normalized verb or `Any`. |
| `path` | `&str` | Absolute path beginning with `/`; explicit token grammar. |

**Returns** A pattern with `Route` metadata. Relative paths, invalid token grammar, and compilation
limits return errors. Matching performs no URL/percent decoding and injects no captures into request
data.

## 4. Pattern inspection

These operations inspect the compiled matcher without selecting or invoking a listener.

### 4.1. EventPattern::source

`source(&self) -> &str`

Read original contributed text; routes display the method (or ANY) plus the original path.

```rust
assert_eq!(pattern.source(), "/profiles/:id");
```

**Returns** A borrowed source string, not a reversible serialization of all matcher options.

### 4.2. EventPattern::route_metadata

`route_metadata(&self) -> Option<&Route>`

Inspect the method/path contribution when the route constructor was used.

```rust
let route = EventPattern::route(MethodPattern::Any, "/health")?;
assert!(route.route_metadata().unwrap().method.is_none());
```

**Returns** Borrowed metadata, or `None` for exact/Express/regex patterns created without the route
constructor.

### 4.3. EventPattern::same_pattern

`same_pattern(&self, other: &Self) -> bool`

Compare typed matcher identity for clearing registrations. Kind, source, separator, and regex case
option participate; equivalent regex behavior alone is insufficient.

```rust
assert!(pattern.same_pattern(&EventPattern::express("/profiles/:id", '/')?));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `other` | `&EventPattern` | Pattern identity to compare. |

**Returns** True for the same typed identity; capture names and matcher kind remain significant.

### 4.4. EventPattern::captures

`captures(&self, event: &str) -> Result<Option<MatchData>>`

Inspect one event without execution.

```rust
let matched = pattern.captures("/profiles/42")?.unwrap();
assert_eq!(matched.params["id"], "42");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `event` | `&str` | At most 8,192 bytes; unlike emitter dispatch, this direct operation permits empty input. |

**Returns** `Some(MatchData)` on a match, `None` otherwise, or a limit error for oversized input.
Optional regex holes remain `None`; matching does not mutate stored pattern state.

## Related

[Events and routing guide](../guides/events-and-routing.md) · [Event](event.md) ·
[Request](request.md) · [API index](README.md)
