# Request API

Owned input values, explicit initialization precedence, and one-attempt loading. Source:
[request.rs](../../src/request.rs). Request is Default/Debug but deliberately not Clone.

```rust
use stackpress_lib::{
    Body, Headers, LoadResult, LoadState, Nest, ParseLimits, Request, RequestView, Result,
    Session, with_unknown_host,
};

async fn read_form(_: RequestView) -> Result<LoadResult> {
    Ok(LoadResult {
        body: Some(Body::Text("name=Grace".into())),
        post: Some(Nest::from_object(serde_json::json!({"name": "Grace"}))?),
    })
}
```

Examples run independently inside an async function returning `Result<()>`. Unless constructed in
the snippet, use `let mut req = Request::new();`. All builder setters consume/return the builder;
only build returns construction errors.

Contents: [Public fields and defaults](#1-public-fields-and-defaults) ·
[Request operations](#2-request-operations) ·
[RequestBuilder operations](#3-requestbuilder-operations) · [URL helper](#4-url-helper)

## 1. Public fields and defaults

Request separates source stores from aggregate input. Construction overlays query → post → explicit
data at the top level; later direct source edits do not synchronize data. No body parser or native
reader runs automatically.

| Request field / state | Type | Default / behavior |
| --- | --- | --- |
| `data` | `Nest` | Empty aggregate input; public and mutable. |
| `query` | `Nest` | Empty unless URL query/override supplies fields. |
| `post` | `Nest` | Empty unless explicitly supplied or loaded. |
| `headers` | `Headers` | Empty validated multi-value fields. |
| `session` | `Session` | Empty or explicit initial session / parsed Cookie headers. |
| `url` | `url::Url` | `http://unknownhost/`. |
| `method` | `Method` | GET. |
| body accessor | `&Body` | Empty. |
| mimetype accessor | `&str` | `text/plain`, independently of Content-Type. |
| load_state accessor | `LoadState` | Unloaded. |

`RequestView` is Clone and has public `data`, `query`, `post`, `headers`, `session`, `url`, and
`method` fields of the same types, plus `body: Body` and `mimetype: String`. It omits attempt state,
retained limits, and native resources. Ordinary snapshot values are independent; only SharedNative
aliases. Debug of Request/RequestView omits buffered values, headers, session contents, and URL, but
includes data keys.

`LoadResult` has public `body: Option<Body>` and `post: Option<Nest>`; Default has both None. None
means leave the existing field/source alone; an empty body or empty store is still a supplied value.
Returned post overlays both post and aggregate data.

## 2. Request operations

Successful load repetition is a no-op. A failed or interrupted attempt is never retried through the
same request. A data snapshot alone cannot recreate native read ownership.

### 2.1. Request::new

`Request::new() -> Self`

Construct the independent defaults above; Default is equivalent.

```rust
let req = Request::new();
assert_eq!(req.method.as_str(), "GET");
```

**Returns** An unloaded request with no native resource or callback.

### 2.2. Request::builder

`Request::builder() -> RequestBuilder`

Begin deferred construction for explicit input and budgets.

```rust
let req = Request::builder()
    .data(serde_json::json!({"name": "Ada"}))
    .build()?;
assert!(req.data.has("name"));
```

**Returns** A default RequestBuilder. This operation performs no fallible input conversion.

### 2.3. Request::body

`body(&self) -> &Body`

Borrow the buffered input without consuming a reader.

```rust
assert!(matches!(req.body(), Body::Empty));
```

**Returns** The exact borrowed Body representation.

### 2.4. Request::mimetype

`mimetype(&self) -> &str`

Inspect the explicit MIME selection; it is not inferred from headers.

```rust
assert_eq!(req.mimetype(), "text/plain");
```

**Returns** Borrowed MIME text.

### 2.5. Request::loaded

`loaded(&self) -> bool`

Check successful finalization.

```rust
assert!(!req.loaded());
```

**Returns** True only for Loaded; Attempted/Failed remain false.

### 2.6. Request::load_state

`load_state(&self) -> LoadState`

Inspect the authoritative read state.

```rust
assert_eq!(req.load_state(), LoadState::Unloaded);
```

**Returns** A Copy state: Unloaded, Attempted, Loaded, or Failed.

### 2.7. Request::view

`view(&self) -> RequestView`

Copy data into an isolated owned loader snapshot without copying attempt state.

```rust
let mut view = req.view();
view.data.set("snapshot_only", true)?;
assert!(!req.data.has("snapshot_only"));
```

**Returns** A Clone data-only view. SharedNative identity is the only deliberate shared payload; no
load attempt is begun.

### 2.8. Request::load

`async load(&mut self) -> Result<()>`

Finalize input using a no-op loader with no native read. It still validates the post/data commit
under retained request limits.

```rust
req.load().await?;
assert_eq!(req.load_state(), LoadState::Loaded);
```

**Returns** Unit after success; repeated Loaded returns unit immediately. Attempted/Failed yields
State error. Commit validation failure records Failed. No body parsing occurs.

### 2.9. Request::load_with

`async load_with<F, Fut>(&mut self, loader: F) -> Result<()> where F: FnOnce(RequestView) -> Fut + Send, Fut: Future<Output = Result<LoadResult>> + Send`

Pass an owned snapshot to one loader, then atomically install validated body/post/data updates.
Snapshot edits alone never update original stores. The callback may borrow caller-owned Send reader
state; it has no static bound.

```rust
req.load_with(read_form).await?;
assert_eq!(req.data.path_as::<String>("name")?, Some("Grace".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `loader` | `F` | One-shot callback accepting RequestView and returning a Send LoadResult future. |

**Returns** Unit after successful staging/commit, with Loaded state. State becomes Attempted before
callback entry, then Failed on callback/commit error; cancellation leaves Attempted. Loaded
repetition skips the callback; Attempted/Failed returns State error. Returned body and resulting
post/data enforce retained request limits and store merge policies before commit.

## 3. RequestBuilder operations

Builder is Default, with the same absent input/default policy as Request::new. Setters replace
earlier inputs of the same kind. Parsing/conversion is deferred to build, except data_serialized
records a serialization result immediately. Final limits apply regardless of setter order.

### 3.1. RequestBuilder::url

`url(self, url: impl Into<String>) -> Self`

Choose URL text; valid absolute URLs are kept, while relative-without-base input gets the
unknown-host prefix. Malformed absolute URLs remain errors.

```rust
let req = Request::builder().url("/profiles?name=Ada").build()?;
assert_eq!(req.url.host_str(), Some("unknownhost"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `url` | `impl Into<String>` | Owned URL text, bounded by final max_bytes at build. |

**Returns** The moved builder; URL errors are deferred to build.

### 3.2. RequestBuilder::method

`method(self, method: impl Into<String>) -> Self`

Choose a real verb, normalized and validated at build.

```rust
let req = Request::builder().method("post").build()?;
assert_eq!(req.method.as_str(), "POST");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `method` | `impl Into<String>` | 1–128-byte method token; see [Method](pattern.md). |

**Returns** The moved builder; invalid token errors are deferred.

### 3.3. RequestBuilder::headers

`headers(self, headers: Headers) -> Self`

Move validated fields; combined invocation limits are rechecked at build.

```rust
let headers = Headers::from_pairs([("accept", "application/json")])?;
let req = Request::builder().headers(headers).build()?;
assert!(req.headers.has("accept"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `headers` | `Headers` | Owned field collection; Content-Type does not choose request MIME or parse the body. |

**Returns** The moved builder.

### 3.4. RequestBuilder::session

`session(self, session: Session) -> Self`

Choose initial state over all Cookie headers. Construction clears outgoing revisions while retaining
live values/options.

```rust
let initial = Session::from_pairs([("theme", "dark")])?;
let req = Request::builder().session(initial).build()?;
assert!(req.session.revisions().is_empty());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `session` | `Session` | Consumed explicit initial state. |

**Returns** The moved builder; the session is installed as initial input at build.

### 3.5. RequestBuilder::query

`query<D: IntoNest + Send + 'static>(self, data: D) -> Self`

Override the URL query with parsed object input; clear a prior query_string override. Preserve
parsed value types.

```rust
let req = Request::builder()
    .url("/x?count=2")
    .query(serde_json::json!({"count": "1"}))
    .build()?;
assert_eq!(req.query.path_as::<String>("count")?, Some("1".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `data` | `D: IntoNest + Send + 'static` | Owned parsed object input. |

**Returns** The moved builder; conversion/root/budget errors occur at build.

### 3.6. RequestBuilder::query_string

`query_string(self, text: impl Into<String>) -> Self`

Override the URL query with raw text; clear a prior parsed query override.

```rust
let req = Request::builder().query_string("count=2").build()?;
assert_eq!(req.query.path_as::<u64>("count")?, Some(2));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `impl Into<String>` | Query text parsed by parse_query under final limits. |

**Returns** The moved builder; parser errors occur at build.

### 3.7. RequestBuilder::post

`post<D: IntoNest + Send + 'static>(self, data: D) -> Self`

Supply parsed post fields, overlaid after query into aggregate data.

```rust
let req = Request::builder()
    .post(serde_json::json!({"name": "Ada"}))
    .build()?;
assert!(req.post.has("name"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `data` | `D: IntoNest + Send + 'static` | Owned object-root source; not raw body parsing. |

**Returns** The moved builder; conversion/overlay failures occur at build.

### 3.8. RequestBuilder::data

`data<D: IntoNest + Send + 'static>(self, data: D) -> Self`

Supply the final top-level initialization overlay. A later successful load may overwrite these same
fields.

```rust
let req = Request::builder()
    .data(serde_json::json!({"active": false}))
    .build()?;
assert_eq!(req.data.path_as::<bool>("active")?, Some(false));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `data` | `D: IntoNest + Send + 'static` | Owned Nest, rich object Value, JSON object, or custom IntoNest input. |

**Returns** The moved builder; conversion/root/budget failures occur at build.

### 3.9. RequestBuilder::data_serialized

`data_serialized<T: Serialize + ?Sized>(self, data: &T) -> Self`

Serialize typed input now, retaining its owned JSON or error for build. This replaces a previous
explicit data selection.

```rust
let req = Request::builder()
    .data_serialized(&serde_json::json!({"name": "Ada"}))
    .build()?;
assert!(req.data.has("name"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `data` | `&T: Serialize` | Borrowed typed value; it need not outlive the setter. |

**Returns** The moved builder. Serialization errors are recorded immediately and returned by build;
object-root/final-limit checking happens at build.

### 3.10. RequestBuilder::body

`body(self, body: Body) -> Self`

Choose exact buffered representation without loading or parsing it.

```rust
let req = Request::builder()
    .body(Body::Text("name=Ada".into()))
    .build()?;
assert!(req.post.is_empty());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `body` | `Body` | Consumed buffer, checked under final limits at build. |

**Returns** The moved builder.

### 3.11. RequestBuilder::mimetype

`mimetype(self, mime: impl Into<String>) -> Self`

Choose explicit MIME independently of headers.

```rust
let req = Request::builder().mimetype("application/json").build()?;
assert_eq!(req.mimetype(), "application/json");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `mime` | `impl Into<String>` | Bounded type/subtype plus valid MIME parameters; no control bytes. |

**Returns** The moved builder; syntax errors are returned by build. No body parsing follows
automatically.

### 3.12. RequestBuilder::limits

`limits(self, limits: ParseLimits) -> Self`

Set the final policy for deferred source construction and later load commits.

```rust
let req = Request::builder()
    .limits(ParseLimits {
        max_bytes: 4096,
        ..Default::default()
    })
    .build()?;
assert_eq!(req.data.limits().max_bytes, 4096);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `ParseLimits` | Copied final workload policy. |

**Returns** The moved builder. Existing Headers/Session APIs retain their own default construction
limits; this does not relax them.

### 3.13. RequestBuilder::build

`build(self) -> Result<Request>`

Consume the builder, validate sources, construct the source stores, and overlay aggregate data.
Explicit query/query_string wins over URL query; the most recently selected override wins. Explicit
session wins over joined Cookie occurrences.

```rust
let req = Request::builder()
    .url("/x?name=query")
    .post(serde_json::json!({"name": "post"}))
    .data(serde_json::json!({"name": "Ada"}))
    .build()?;
assert_eq!(req.data.path_as::<String>("name")?, Some("Ada".into()));
```

**Returns** A complete Unloaded Request. URL/method/MIME/header/body validation, query/cookie
parsing, input conversion, object roots, and resulting data budgets can fail. No partially built
request is returned; no native I/O is performed.

## 4. URL helper

This helper preserves a literal upstream prefix operation. It differs from RequestBuilder's URL
parsing and relative-URL fallback.

### 4.1. with_unknown_host

`with_unknown_host(text: &str) -> String`

Prefix supplied text literally, adding a slash unless already present.

```rust
assert_eq!(
    with_unknown_host("https://example.test"),
    "http://unknownhost/https://example.test"
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Any path-like text; this helper does not parse it. |

**Returns** An owned prefixed string, infallibly. Protocol-looking text is still placed after the
unknownhost prefix; it grants no network access.

## Related

[Requests and responses guide](../guides/requests-and-responses.md) · [Response](response.md) ·
[Parsers](parsers.md) · [Data](data.md) · [Session](session.md) · [API index](README.md)
