# Track sessions and scoped cookies

Deleting a cookie requires the same domain and path used when setting it. `Session` records outgoing
revisions with those options, letting an adapter serialize updates without losing removal scope.

## Set and remove a scoped value

Inside a function returning `stackpress_lib::Result<()>`, create a session and update one cookie:

```rust
use stackpress_lib::{CookieOptions, Session};

let mut session = Session::new();
session.set(
    "theme",
    "dark",
    Some(CookieOptions {
        path: Some("/preferences".into()),
        http_only: true,
        ..Default::default()
    }),
)?;
session.set("theme", "light", None)?;
assert_eq!(
    session.options("theme").unwrap().path.as_deref(),
    Some("/preferences")
);
```

Omitting options on a later set retains the live options. Passing `Some(CookieOptions::default())`
explicitly replaces them. Each name has its latest revision, rather than an append-only history of
every write.

Now remove the value and inspect the wire header:

```rust
assert!(session.remove("theme")?);
let headers = session.cookie_headers()?;
assert!(headers[0].contains("Path=/preferences"));
assert!(headers[0].contains("Max-Age=0"));
```

Removal records a tombstone with the old scope, epoch expiry, and zero maximum age at serialization.
Removing an already missing name still records a removal; repeated removal preserves a previous
tombstone's scope. `clear` removes live names while retaining their removal revisions.

## Attach outgoing revisions

In a function that owns `res: Response`, append each serialized revision separately:

```rust
let outgoing = res.session().cookie_headers()?;
for value in outgoing {
    // Each Set-Cookie occurrence is an independent header value.
    res.headers_mut()?.append("set-cookie", &value)?;
}
```

Session changes do not automatically write headers. Prepare them while the response is open, before
`dispatch_with`. Input sessions from `Session::from_pairs` and request construction start without
outgoing revisions.

## Choose a list codec explicitly

`SessionValue` can hold a string list, but the default cookie writer rejects lists. A caller can
deliberately select JSON as the list representation:

```rust
use stackpress_lib::SessionValue;

session.set("roles", vec!["reader".into(), "writer".into()], None)?;
let headers = session.cookie_headers_with(|value| match value {
    SessionValue::Text(text) => Ok(text.clone()),
    SessionValue::List(items) => Ok(serde_json::to_string(items)?),
})?;
```

The codec produces text; the cookie serializer then percent-encodes and validates it. A matching
input decoding policy is the adapter's responsibility. Cookie values alone establish no
authentication or authorization.

You have preserved scope through updates and deletion, and made wire conversion explicit. Use
[session API](../api/session.md) for option validation, custom codecs, and revision fields; see
[headers API](../api/headers-and-body.md) for repeated values.
