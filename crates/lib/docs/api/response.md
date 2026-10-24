# Response API

Buffered application output, public result envelopes, and one-attempt delivery. Source:
[response.rs](../../src/response.rs). Response implements Default/Debug and deliberately has no
Clone.

```rust
use stackpress_lib::{
    Body, DeliveryState, Error, ParseLimits, PublicError, Response, ResponseEnvelope,
    ResponseView, Result, SourceLocation, Status, Value,
};

async fn write(view: ResponseView) -> Result<()> {
    // A native adapter would write this snapshot using caller-owned resources.
    assert!(view.envelope.results.is_some());
    Ok(())
}
```

Run each snippet independently inside an async function returning `Result<()>`, with
`let mut res = Response::new();` unless it constructs its own response. Setters/accessors returning
mutable state are guarded by delivery state.

Contents: [Defaults, envelopes, and views](#1-defaults-envelopes-and-views) ·
[Construction and read access](#2-construction-and-read-access) ·
[Guarded mutable access](#3-guarded-mutable-access) ·
[Body, status, and public errors](#4-body-status-and-public-errors) ·
[Envelope transfer and snapshots](#5-envelope-transfer-and-snapshots) ·
[Delivery and suppression](#6-delivery-and-suppression)

## 1. Defaults, envelopes, and views

New output is Open with Body::Empty, no status/MIME/public message/private error/stack, total 0, and
empty headers/session/data/errors. Response data is metadata, distinct from the selected result
body. Convenience body setters do not automatically clear unrelated error fields, metadata, headers,
or stack; set/html/xml/json also retain an existing total.

| ResponseEnvelope field | Type | Meaning |
| --- | --- | --- |
| `code` | `Option<u16>` | Application code. |
| `status` | `Option<String>` | Application label. |
| `error` | `Option<String>` | Explicit public message. |
| `errors` | `Option<Value>` | Public field errors; applying to Response requires an object root. |
| `results` | `Option<Value>` | Rich in-process result; present Null differs from absence. |
| `total` | `Option<u64>` | Optional count, including zero. |
| `stack` | `Option<Vec<SourceLocation>>` | Explicitly disclosed locations, not private diagnostics. |

ResponseEnvelope is Clone/Debug/Default; Default leaves every field None. Envelopes made from
Response always include errors (even `{}` ) and total (even 0). Empty body becomes absent results;
Text becomes String, Bytes remains Bytes, and Body::Value retains its Value. Envelopes omit headers,
MIME, session, metadata, native resources, private error causes, and delivery state.

`ResponseView` is Clone with public `status: Option<Status>`, `body: Body`, `headers: Headers`,
`session: Session`, `data: Nest`, `envelope: ResponseEnvelope`, and `mimetype: Option<String>`.
Snapshots contain neither delivery state nor private causes. Ordinary data clones independently;
SharedNative retains identity. Response/View Debug reports state/representation without buffered
payloads.

## 2. Construction and read access

Read access remains available after finalization. Use precise attempt state rather than sent() when
distinguishing successful writer completion from explicit suppression.

### 2.1. Response::new

`Response::new() -> Self`

Create independently owned open output with default ParseLimits; Default is equivalent.

```rust
let res = Response::new();
assert!(res.status().is_none());
```

**Returns** A new open response; no output is delivered.

### 2.2. Response::with_limits

`Response::with_limits(limits: ParseLimits) -> Self`

Retain a deliberate body/data/error policy for mutation and delivery checks.

```rust
let res = Response::with_limits(ParseLimits {
    max_bytes: 4096,
    ..Default::default()
});
assert_eq!(res.data().limits().max_bytes, 4096);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `ParseLimits` | Selected workload budgets; header/session construction still has its own default policy. |

**Returns** Open empty output, infallibly; subsequent checked operations establish whether values
fit the policy.

### 2.3. Response::body

`body(&self) -> &Body`

Borrow the exact selected buffer.

```rust
assert!(matches!(res.body(), Body::Empty));
```

**Returns** The body, including text/raw-byte/rich distinctions.

### 2.4. Response::result

`result(&self) -> Option<&Value>`

Borrow structured results only.

```rust
res.results(&false)?;
assert_eq!(res.result().and_then(Value::as_bool), Some(false));
```

**Returns** Some only for Body::Value. Text and raw Bytes return None even though envelopes can
represent them.

### 2.5. Response::status

`status(&self) -> Option<&Status>`

Inspect the chosen application status.

```rust
assert_eq!(res.status(), None);
```

**Returns** Borrowed status, or None when unset; no implicit code 200.

### 2.6. Response::mimetype

`mimetype(&self) -> Option<&str>`

Read the explicit body media selection.

```rust
res.html("<p>Hello</p>")?;
assert_eq!(res.mimetype(), Some("text/html"));
```

**Returns** Borrowed MIME, or None when unset.

### 2.7. Response::total

`total(&self) -> u64`

Read the stored result count.

```rust
assert_eq!(res.total(), 0);
```

**Returns** The count, including zero.

### 2.8. Response::error

`error(&self) -> Option<&str>`

Read intentional public error text.

```rust
assert!(res.error().is_none());
```

**Returns** Borrowed public message, or None; empty text is a present message.

### 2.9. Response::stack

`stack(&self) -> Option<&[SourceLocation]>`

Read explicitly supplied public locations.

```rust
assert!(res.stack().is_none());
```

**Returns** Borrowed disclosed locations, or None. Diagnostic frames are separate.

### 2.10. Response::diagnostic

`diagnostic(&self) -> Option<&Error>`

Inspect an owned failure retained through apply_error.

```rust
res.apply_error(Error::new("private failure"))?;
assert_eq!(res.diagnostic().unwrap().message(), "private failure");
```

**Returns** Borrowed original Error or None; this is a developer view.

### 2.11. Response::delivery_state

`delivery_state(&self) -> DeliveryState`

Read the authoritative send/suppression state.

```rust
assert_eq!(res.delivery_state(), DeliveryState::Open);
```

**Returns** One of Open, Attempted, Sent, Stopped, Failed.

### 2.12. Response::sent

`sent(&self) -> bool`

Check the compatibility predicate meaning finalized by delivery or suppression.

```rust
res.stop()?;
assert!(res.sent());
```

**Returns** True for Sent or Stopped; false for Open, Attempted, Failed. It does not prove bytes
were written.

### 2.13. Response::redirected

`redirected(&self) -> bool`

Check whether Location is present.

```rust
res.redirect("/profiles")?;
assert!(res.redirected());
```

**Returns** Presence of the normalized Location header; status range is not inspected.

### 2.14. Response::headers

`headers(&self) -> &Headers`

Borrow normalized multi-value output fields.

```rust
assert!(res.headers().is_empty());
```

**Returns** Immutable Headers; mutation uses headers_mut.

### 2.15. Response::session

`session(&self) -> &Session`

Borrow outgoing cookie values and revisions.

```rust
assert!(res.session().is_empty());
```

**Returns** Immutable Session; this does not serialize headers or acknowledge delivery.

### 2.16. Response::data

`data(&self) -> &Nest`

Borrow response metadata independently of result body.

```rust
assert!(res.data().is_empty());
```

**Returns** Immutable metadata store.

### 2.17. Response::errors

`errors(&self) -> &Nest`

Borrow the public field-error store.

```rust
assert!(res.errors().is_empty());
```

**Returns** Immutable object-root field errors.

## 3. Guarded mutable access

Every accessor below checks DeliveryState::Open. This guard prevents edits after any writer attempt,
including failure/cancellation, as well as Sent/Stopped. The returned borrow is exclusive, so it
cannot remain live across another mutable response operation.

### 3.1. Response::headers_mut

`headers_mut(&mut self) -> Result<&mut Headers>`

Edit checked multi-value fields.

```rust
res.headers_mut()?.append("set-cookie", "theme=dark")?;
```

**Returns** A mutable Headers borrow while Open, or a State error. The collection's own operations
may return their own validation errors. No delivery is performed.

### 3.2. Response::session_mut

`session_mut(&mut self) -> Result<&mut Session>`

Record cookie changes before preparing native output.

```rust
res.session_mut()?.set("theme", "dark", None)?;
```

**Returns** A mutable Session borrow while Open, or a State error. The collection's own operations
may return their own validation errors. No delivery is performed.

### 3.3. Response::data_mut

`data_mut(&mut self) -> Result<&mut Nest>`

Edit response metadata.

```rust
res.data_mut()?.set("operation", "profile-read")?;
```

**Returns** A mutable Nest borrow while Open, or a State error. The collection's own operations may
return their own validation errors. No delivery is performed.

### 3.4. Response::errors_mut

`errors_mut(&mut self) -> Result<&mut Nest>`

Edit explicit public field errors.

```rust
res.errors_mut()?.set("name", "required")?;
```

**Returns** A mutable Nest borrow while Open, or a State error. The collection's own operations may
return their own validation errors. No delivery is performed.

## 4. Body, status, and public errors

All mutation methods reject non-Open output. Fallible input preparation precedes replacement. `set`,
html/xml/json/results/results_value/rows select OK; put a desired status override after them.
Redirect chooses Found and Location without clearing an existing body/MIME.

### 4.1. Response::status_code

`status_code(&mut self, code: u16, message: Option<String>) -> Result<&mut Self>`

Set an application code and optionally a custom label.

```rust
res.results(&"created")?.status_code(201, None)?;
assert_eq!(res.status(), Some(&Status::CREATED));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `code` | `u16` | Application code; not constrained to HTTP-valid values. |
| `message` | `Option<String>` | None uses catalog/Unknown; Some uses a custom label, at most 8,192 bytes. |

**Returns** The mutable response, or State/label-limit error; no body or count is changed.

### 4.2. Response::set

`set(&mut self, mime: &str, body: Body) -> Result<&mut Self>`

Select MIME/body and status OK after validating both.

```rust
res.set("application/octet-stream", Body::Bytes(vec![0, 255]))?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `mime` | `&str` | Valid bounded type/subtype plus parameters; no controls. |
| `body` | `Body` | Owned buffer validated under retained response limits. |

**Returns** The mutable response, or State/MIME/body-limit error without replacement. Total and
unrelated fields are retained.

### 4.3. Response::html

`html(&mut self, text: impl Into<String>) -> Result<&mut Self>`

Buffer exact text with `text/html` and OK status; no escaping/rendering is performed.

```rust
res.html("<h1>Ada</h1>")?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `impl Into<String>` | Owned text within response byte limits. |

**Returns** The mutable response, or State/body-limit error. Existing total/error/header/session
metadata is not automatically cleared.

### 4.4. Response::xml

`xml(&mut self, text: impl Into<String>) -> Result<&mut Self>`

Buffer exact text with `text/xml` and OK status; no escaping/rendering is performed.

```rust
res.xml("<profile/>")?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `impl Into<String>` | Owned text within response byte limits. |

**Returns** The mutable response, or State/body-limit error. Existing total/error/header/session
metadata is not automatically cleared.

### 4.5. Response::json

`json<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self>`

Serialize a typed value as pretty JSON text with application/json and OK status. A string input is
serialized as a JSON string, not treated as pre-encoded JSON.

```rust
res.json(&serde_json::json!({"name": "Ada"}))?;
assert!(matches!(res.body(), Body::Text(_)));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `&T: Serialize` | Typed value; rich Value serialization may reject non-JSON/default-budget input. |

**Returns** The mutable response, or State/serialization/tree/final-text budget error. Previous
output is preserved on error; success retains existing total. Use set with Body::Text for
deliberately raw JSON text.

### 4.6. Response::results

`results<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self>`

Serialize typed data into a structured Value, application/json, OK status, and total 1.

```rust
res.results(&serde_json::json!({"name": "Ada"}))?;
assert_eq!(res.total(), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `&T: Serialize` | Real serializable scalar/object/array; no object-only restriction. |

**Returns** The mutable response, or State/serialization/value-limit error without replacement. This
does not create JSON text or send output.

### 4.7. Response::results_value

`results_value(&mut self, value: Value) -> Result<&mut Self>`

Keep rich in-process values while selecting application/json, OK, and total 1.

```rust
res.results_value(Value::Bytes(vec![0, 255]))?;
assert!(res.to_envelope().try_to_json().is_err());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `Value` | Owned rich value; non-JSON variants are allowed in process. |

**Returns** The mutable response, or State/value-limit error. MIME selection does not guarantee JSON
convertibility; use an explicit wire representation before JSON export.

### 4.8. Response::rows

`rows<T: Serialize + ?Sized>(&mut self, value: &T, total: u64) -> Result<&mut Self>`

Store a serialized array as structured output, with application/json and OK status.

```rust
res.rows(&[serde_json::json!({"id": 7})], 0)?;
assert_eq!(res.total(), 0);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `&T: Serialize` | Must serialize to an array; arbitrary objects are rejected. |
| `total` | `u64` | Caller-supplied overall count; may differ from row count and may be zero. |

**Returns** The mutable response, or State/serialization/non-array/value-limit error without
changing previous output.

### 4.9. Response::set_total

`set_total(&mut self, total: u64) -> Result<&mut Self>`

Override the count after a convenience method.

```rust
res.results(&false)?.set_total(0)?;
assert_eq!(res.total(), 0);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `total` | `u64` | Explicit count including zero. |

**Returns** The mutable response, or State error; no status/body change.

### 4.10. Response::redirect

`redirect(&mut self, location: &str) -> Result<&mut Self>`

Set checked Location and Found status after header validation.

```rust
res.redirect("/profiles")?;
assert_eq!(res.status(), Some(&Status::FOUND));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `location` | `&str` | Header value; framing/aggregate header checks apply, without URL-policy validation. |

**Returns** The mutable response, or State/header validation error without redirect mutation.
Existing body/MIME/count remain as they were.

### 4.11. Response::set_error

`set_error(&mut self, error: PublicError) -> Result<&mut Self>`

Apply public code/label/message/object field errors and clear a prior private diagnostic cause.

```rust
res.set_error(PublicError {
    code: 400,
    status: "Bad Request".into(),
    error: "Name is required".into(),
    errors: serde_json::json!({"name": "required"}),
})?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `error` | `PublicError` | Object-root field errors, status label ≤8,192 bytes, message within max_bytes. |

**Returns** The mutable response, or State/root/limit/conversion error without partial application.
It does not clear an existing body, total, MIME, or disclosed stack.

### 4.12. Response::apply_error

`apply_error(&mut self, error: Error) -> Result<&mut Self>`

Apply public_view while retaining the original owned error for diagnostics and into_error.

```rust
res.apply_error(
    Error::new("private database detail").with_public_message("Could not load profile"),
)?;
assert!(res.diagnostic().is_some());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `error` | `Error` | Consumed diagnostic error and original source chain. |

**Returns** The mutable response, or public-view application error with the original error attached
as its cause. Private frames/backtrace are not copied to public stack; body/count/MIME are retained.

### 4.13. Response::set_stack

`set_stack(&mut self, stack: Vec<SourceLocation>) -> Result<&mut Self>`

Deliberately disclose source locations in the public envelope.

```rust
res.set_stack(vec![SourceLocation::caller()])?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `stack` | `Vec<SourceLocation>` | Explicit locations; count bounded by max_entries. Location string sizes are not checked here. |

**Returns** The mutable response, or State/count-limit error; this is independent of private
diagnostic frames.

## 5. Envelope transfer and snapshots

Transfers carry public result/status fields, never delivery ownership. `None` means preserve/omit;
present false, zero, empty, and null remain real supplied values.

### 5.1. Response::apply_envelope

`apply_envelope(&mut self, envelope: ResponseEnvelope) -> Result<&mut Self>`

Stage all field validation, then apply supplied fields. Code alone uses the catalog label; status
alone uses the current code or 0 if unset. Supplying a public error clears an old private cause.
Object/array results select application/json; scalar results retain existing MIME.

```rust
res.apply_envelope(ResponseEnvelope {
    results: Some(Value::Bool(false)),
    total: Some(0),
    ..Default::default()
})?;
assert!(res.status().is_none());
assert_eq!(res.result().and_then(Value::as_bool), Some(false));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `envelope` | `ResponseEnvelope` | Owned optional values; errors must be an object, status ≤8,192 bytes, message within max_bytes, stack count within max_entries. |

**Returns** The mutable response, or State/root/value/metadata-limit error without partial mutation.
Absent fields retain existing state; present results do not manufacture a status or count. Delivery
state remains Open.

### 5.2. Response::to_envelope

`to_envelope(&self) -> ResponseEnvelope`

Clone the public rich-value result/status view without JSON conversion.

```rust
res.results(&false)?;
let envelope = res.to_envelope();
assert_eq!(envelope.total, Some(1));
```

**Returns** An infallible cloned envelope. Ordinary values are independent; explicit SharedNative
remains shared. Errors and total are always present. Headers/session/MIME/data/private
cause/delivery state are omitted.

### 5.3. Response::into_envelope

`into_envelope(self) -> ResponseEnvelope`

Consume the output owner and move public values without cloning result data.

```rust
res.results(&false)?;
let envelope = res.into_envelope();
assert_eq!(envelope.results.unwrap().as_bool(), Some(false));
```

**Returns** A moved public envelope, infallibly. Private cause and all non-envelope state are
discarded; no delivery occurs.

### 5.4. Response::into_error

`into_error(self) -> Error`

Recover an owned original failure, or construct an error from the public message/code/field errors.

```rust
res.apply_error(Error::new("original failure"))?;
assert_eq!(res.into_error().message(), "original failure");
```

**Returns** The exact retained original Error when present. Otherwise constructs a Domain error from
public message or `Unknown Error`, code or 500, and converted field errors; it does not retain a
custom status label or disclosed stack, or invent a cause. If field-error JSON conversion fails,
that failure becomes the new error's cause.

### 5.5. Response::view

`view(&self) -> ResponseView`

Copy output data for an isolated adapter snapshot without native/delivery ownership.

```rust
res.results(&false)?;
let mut view = res.view();
view.data.set("snapshot_only", true)?;
assert!(!res.data().has("snapshot_only"));
```

**Returns** A Clone data-only view, infallibly. It includes the public envelope but no private error
cause or attempt state; SharedNative sharing remains explicit.

### 5.6. ResponseEnvelope::try_to_json

`try_to_json(&self) -> Result<serde_json::Value>`

Convert present envelope fields to JSON explicitly.

```rust
let envelope = ResponseEnvelope {
    results: Some(Value::Null),
    total: Some(0),
    ..Default::default()
};
let json = envelope.try_to_json()?;
assert!(json["results"].is_null());
assert_eq!(json["total"], 0);
```

**Returns** An object omitting None fields and retaining present values. Errors/results use Value's
default conversion policy, independently of a response's retained limits; unsupported values/limits
return a path prefixed with `errors` or `results`. Stack is serialized only when explicitly
supplied. This method does not apply Response's metadata-length/count validation to a manually
constructed envelope.

## 6. Delivery and suppression

DeliveryState is Clone/Copy/Debug/Eq/PartialEq with Open, Attempted, Sent, Stopped, Failed variants.
Values freeze after an attempt. A failure may follow partial external output; cancellation is not
evidence that nothing was sent. No implicit retry, rollback, or async cleanup occurs.

### 6.1. Response::dispatch

`async dispatch(&mut self) -> Result<()>`

Finalize with a no-op writer and no native output, using the same validation/attempt contract.

```rust
res.results(&false)?;
res.dispatch().await?;
assert_eq!(res.delivery_state(), DeliveryState::Sent);
```

**Returns** Unit after no-output finalization. Sent/Stopped repetition is a no-op; Attempted/Failed
returns State error. Pre-attempt value/header limits may fail while leaving Open.

### 6.2. Response::dispatch_with

`async dispatch_with<F, Fut>(&mut self, dispatcher: F) -> Result<()> where F: FnOnce(ResponseView) -> Fut + Send, Fut: Future<Output = Result<()>> + Send`

Validate buffered body/data/errors/headers under response limits, snapshot output, mark Attempted,
and invoke one writer. The writer may borrow caller-owned Send native state; no static callback
bound is required.

```rust
res.results(&false)?;
res.dispatch_with(write).await?;
assert_eq!(res.delivery_state(), DeliveryState::Sent);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `dispatcher` | `F` | One-shot callback receiving an owned isolated ResponseView and returning a Send unit-result future. |

**Returns** Unit on success with Sent. Callback failure records Failed and returns its error;
dropping while pending leaves Attempted. Sent/Stopped repeats skip the callback. Attempted/Failed
returns State error. Preflight validation failure leaves Open. Session cookie encoding/header
attachment is caller policy, not automatic delivery work.

### 6.3. Response::stop

`stop(&mut self) -> Result<()>`

Suppress output explicitly, freeze mutation, and prevent writer entry.

```rust
res.stop()?;
assert_eq!(res.delivery_state(), DeliveryState::Stopped);
res.dispatch_with(write).await?; // The callback is skipped after suppression.
```

**Returns** Unit with Stopped; repeated stop is a no-op. Other non-Open states, including Sent,
return State error. No writer is invoked and no body validation/output is performed.

## Related

[Requests and responses guide](../guides/requests-and-responses.md) · [Error](error.md) ·
[Status](status.md) · [Headers/body](headers-and-body.md) · [Session](session.md) ·
[API index](README.md)
