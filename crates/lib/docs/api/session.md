# Session and cookie API

Ordered cookie-backed values, latest outgoing revisions, and checked pure cookie codecs. This state
does not establish authenticated identity. Source: [session.rs](../../src/session.rs).

```rust
use stackpress_lib::session::cookie;
use stackpress_lib::{
    CookieOptions, Priority, Result, SameSite, Session, SessionRevision, SessionValue,
};

let mut session = Session::from_pairs([("theme", "dark")])?;
```

Method snippets run independently inside a function returning `Result<()>`, with these imports and
initial `session`. [Sessions and cookies](../guides/sessions-and-cookies.md) shows header
attachment.

Contents: [Values, revisions, and options](#1-values-revisions-and-options) ·
[Session operations](#2-session-operations) · [Cookie functions](#3-cookie-functions)

## 1. Values, revisions, and options

Session is Clone/Default with redacted Debug listing names and revision count. SessionValue is
Clone/Debug/Eq/PartialEq with `Text(String)` and `List(Vec<String>)`; conversions accept `&str`,
String, and Vec<String>. Lists never silently become a cookie string.

`SessionRevision` is Clone/Debug/Eq/PartialEq with
`Set { value: SessionValue, options: Option<CookieOptions> }` and
`Remove { options: Option<CookieOptions> }`. Each name retains the latest revision at its first
revision insertion position. Remove options retain prior scope after live options are removed.
`from_pairs` and Request initialization produce no outgoing revisions.

CookieOptions is Clone/Debug/Default/Eq/PartialEq. Default has every optional field None and every
boolean false:

| Field | Type | Meaning |
| --- | --- | --- |
| `domain` | `Option<String>` | Explicit domain; validated DNS labels, optional leading dot. |
| `path` | `Option<String>` | Explicit path scope; printable ASCII without semicolon, at most 8,192 bytes. |
| `expires` | `Option<SystemTime>` | HTTP-date expiry; supported range begins at UNIX_EPOCH and ends before epoch second 253402300800. |
| `max_age` | `Option<i64>` | Signed whole seconds, including zero/negative. |
| `http_only` | `bool` | Emit HttpOnly when true. |
| `secure` | `bool` | Emit Secure when true. |
| `partitioned` | `bool` | Emit Partitioned when true. |
| `priority` | `Option<Priority>` | Priority::Low, Medium, High. |
| `same_site` | `Option<SameSite>` | SameSite::Strict, Lax, None; enum None differs from absent Option. |

Priority and SameSite are Clone/Copy/Debug/Eq/PartialEq. Validation checks syntax/date range; it
does not enforce browser security combinations such as requiring Secure with a particular SameSite
or partitioning choice. Session budgets use fixed default limits: live names/revision names each at
most 10,000, and retained live values/options plus revision copies/scopes at most 1 MiB.

### 1.1. SessionValue::as_str

`as_str(&self) -> Option<&str>`

Borrow a scalar without collapsing a list.

```rust
assert_eq!(SessionValue::from("dark").as_str(), Some("dark"));
```

**Returns** Borrowed Text, or None for List.

### 1.2. CookieOptions::validate

`validate(&self) -> Result<()>`

Check options before storing them or serializing a cookie.

```rust
CookieOptions {
    path: Some("/preferences".into()),
    same_site: Some(SameSite::Lax),
    ..Default::default()
}
.validate()?;
```

**Returns** Unit, or an input error. Domain labels must be nonempty ASCII alphanumeric/hyphen,
start/end alphanumeric, ≤63 bytes each and ≤253 bytes after an optional leading dot. Invalid path
bytes/length and dates outside the supported nonnegative range also fail; no domain ownership or
transport trust is inferred.

## 2. Session operations

Mutations stage and validate live state plus its matching revision before replacing state. Set
without options retains current live options; explicit options replace them. Serialization does not
clear revisions or write headers.

### 2.1. Session::new

`Session::new() -> Self`

Create an empty independent collection; Default is equivalent.

```rust
let session = Session::new();
assert!(session.revisions().is_empty());
```

**Returns** An empty session with no options/revisions.

### 2.2. Session::from_pairs

`Session::from_pairs<K: Into<String>, V: Into<SessionValue>>(pairs: impl IntoIterator<Item = (K, V)>) -> Result<Self>`

Build validated initial values, then clear construction revisions.

```rust
let session = Session::from_pairs([("theme", "dark")])?;
assert!(session.revisions().is_empty());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pairs` | `impl IntoIterator<Item = (K, V)>` | Owned/copyable name/value inputs; duplicate names replace prior values. |

**Returns** An initial Session with no outgoing revisions, or name/value-budget error. Construction
validates using the same staged setters before clearing revisions.

### 2.3. Session::get

`get(&self, name: &str) -> Option<&SessionValue>`

Read a case-sensitive live cookie name.

```rust
assert_eq!(
    session.get("theme").and_then(SessionValue::as_str),
    Some("dark")
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Exact name. |

**Returns** Borrowed live Text/List or None.

### 2.4. Session::options

`options(&self, name: &str) -> Option<&CookieOptions>`

Read live options; removed scope resides in revisions instead.

```rust
assert!(session.options("theme").is_none());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Exact name. |

**Returns** Borrowed live options or None; initial pairs do not supply options.

### 2.5. Session::revisions

`revisions(&self) -> &DataMap<String, SessionRevision>`

Inspect latest outgoing change per name without acknowledging delivery.

```rust
assert!(session.revisions().is_empty());
```

**Returns** Borrowed ordered revision map, including tombstones for missing names.

### 2.6. Session::entries

`entries(&self) -> impl Iterator<Item = (&String, &SessionValue)>`

Inspect live insertion order independently of revisions.

```rust
assert_eq!(session.entries().count(), 1);
```

**Returns** A borrowing iterator of live names/values.

### 2.7. Session::len

`len(&self) -> usize`

Count live names rather than outgoing revisions.

```rust
assert_eq!(session.len(), 1);
```

**Returns** Live name count.

### 2.8. Session::is_empty

`is_empty(&self) -> bool`

Test live value presence.

```rust
assert!(!session.is_empty());
```

**Returns** True with no live values, even when removal revisions remain.

### 2.9. Session::set

`set(&mut self, name: &str, value: impl Into<SessionValue>, options: Option<CookieOptions>) -> Result<&mut Self>`

Set a live value and matching latest revision. Omitted options retain existing live scope; Some
replaces the complete options value.

```rust
session.set(
    "theme",
    "light",
    Some(CookieOptions {
        path: Some("/preferences".into()),
        ..Default::default()
    }),
)?;
session.set("theme", "dark", None)?;
assert_eq!(
    session.options("theme").unwrap().path.as_deref(),
    Some("/preferences")
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Nonempty ASCII token name, at most 8,192 bytes. |
| `value` | `impl Into<SessionValue>` | Owned Text or List. |
| `options` | `Option<CookieOptions>` | None retains live options; Some replaces them after validation. |

**Returns** The mutable session, or name/options/budget error with no partial state/revision update.
A name removed previously has no live options to retain on a new set.

### 2.10. Session::remove

`remove(&mut self, name: &str) -> Result<bool>`

Remove live value/options and record a scoped tombstone. Repeated removal keeps existing removal
scope; a missing name still gets a revision.

```rust
assert!(session.remove("theme")?);
assert!(!session.remove("theme")?);
assert!(matches!(
    session.revisions().get("theme"),
    Some(SessionRevision::Remove { .. })
));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Valid cookie token name. |

**Returns** True if a live value was removed, false if already missing, or name/budget error without
mutation. Removal serialization forces Max-Age=0 and UNIX_EPOCH expiry while retaining stored
options.

### 2.11. Session::clear

`clear(&mut self) -> Result<()>`

Stage removal of every live key, retaining prior pending revisions and scope.

```rust
session.clear()?;
assert!(session.is_empty());
assert!(session.revisions().contains_key("theme"));
```

**Returns** Unit, or a mutation/budget error leaving the original batch unchanged. Pending removals
remain even though no live values remain.

### 2.12. Session::cookie_headers

`cookie_headers(&self) -> Result<Vec<String>>`

Serialize scalar revisions into separate Set-Cookie field values.

```rust
session.set("theme", "light", None)?;
assert_eq!(session.cookie_headers()?, ["theme=light"]);
```

**Returns** Owned header values in revision order; initial-only state yields an empty vector. List
values return an explicit-codec error; cookie serialization errors propagate. Revisions remain
pending.

### 2.13. Session::cookie_headers_with

`cookie_headers_with(&self, codec: impl Fn(&SessionValue) -> Result<String>) -> Result<Vec<String>>`

Select text encoding for every Set revision before checked percent-encoded cookie serialization.
Remove bypasses the value codec.

```rust
session.set("roles", vec!["reader".into(), "writer".into()], None)?;
let headers = session.cookie_headers_with(|value| match value {
    SessionValue::Text(text) => Ok(text.clone()),
    SessionValue::List(items) => Ok(serde_json::to_string(items)?),
})?;
assert!(headers[0].starts_with("roles="));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `codec` | `impl Fn(&SessionValue) -> Result<String>` | Application-chosen text representation; result is still percent-encoded by cookie::serialize. |

**Returns** Separate header strings, or codec/cookie validation error. No partial vector is
returned, no headers are attached, and no revision is acknowledged/cleared.

## 3. Cookie functions

The pure `session::cookie` module parses incoming Cookie text and emits one Set-Cookie value at a
time. Input permissiveness differs from strict query decoding. Names remain case-sensitive; cookie
plus signs are literal.

### 3.1. cookie::parse

`cookie::parse(text: &str) -> Result<DataMap<String, String>>`

Trim spaces/tabs, ignore fragments without equals or valid token names, and retain the first
accepted value per name. Percent-decode once; malformed percent/UTF-8 decoding falls back to the raw
value.

```rust
let cookies = cookie::parse("theme=dark; theme=light; raw=%ZZ; plus=a+b")?;
assert_eq!(cookies["theme"], "dark");
assert_eq!(cookies["plus"], "a+b");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Incoming Cookie header text; repeated header lines may be joined with `; ` by the adapter. |

**Returns** An insertion-ordered string map, including empty decoded strings. Default raw/decoded
byte limits and fragment-count limits return errors; malformed fragments are otherwise ignored.

### 3.2. cookie::parse_with

`cookie::parse_with(text: &str, decode: impl Fn(&str) -> Option<String>) -> Result<DataMap<String, String>>`

Use an explicit value decoder. None skips that candidate; a later occurrence can then become the
first retained value.

```rust
let values = cookie::parse_with("theme=dark; ignored=x", |value| {
    if value == "x" {
        None
    } else {
        Some(value.to_owned())
    }
})?;
assert_eq!(values.len(), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Same input framing as parse. |
| `decode` | `impl Fn(&str) -> Option<String>` | Trimmed raw value in; owned accepted text or None out. |

**Returns** An ordered map or default raw/decoded-budget/fragment-count error. Duplicate retained
names bypass the decoder; custom decoded values are byte-budgeted, not reinterpreted as fields.

### 3.3. cookie::serialize

`cookie::serialize(name: &str, value: &str, options: &CookieOptions) -> Result<String>`

Percent-encode UTF-8 value bytes and append checked options in stable wire order.

```rust
let header = cookie::serialize("name", "Ada Lovelace", &CookieOptions::default())?;
assert_eq!(header, "name=Ada%20Lovelace");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Valid bounded ASCII token. |
| `value` | `&str` | Scalar text to percent-encode. |
| `options` | `&CookieOptions` | Validated domain/path/date/typed output attributes. |

**Returns** One owned header value, or invalid name/options/encoded-value error. Empty values are
permitted. This does not infer HTTP security policy or join separate cookies.

### 3.4. cookie::serialize_with

`cookie::serialize_with(name: &str, value: &str, options: &CookieOptions, encode: impl Fn(&str) -> String) -> Result<String>`

Select a custom encoder, then revalidate its output before using it as header framing.

```rust
let header = cookie::serialize_with("theme", "dark", &CookieOptions::default(), str::to_owned)?;
assert_eq!(header, "theme=dark");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Valid bounded ASCII token. |
| `value` | `&str` | Text passed to the encoder. |
| `options` | `&CookieOptions` | Checked before encoder entry. |
| `encode` | `impl Fn(&str) -> String` | Produces encoded ASCII value; not encoded again. |

**Returns** One header value. Encoded text over default max_bytes, bytes outside 0x21–0x7e, or
semicolon/comma/double-quote/backslash returns an input error; empty text is accepted. Name/options
errors occur before the encoder. Attributes append Max-Age, Domain, Path, Expires, HttpOnly, Secure,
Partitioned, Priority, then SameSite when selected.

## Related

[Sessions and cookies guide](../guides/sessions-and-cookies.md) ·
[Headers/body](headers-and-body.md) · [Request](request.md) · [Response](response.md) ·
[API index](README.md)
