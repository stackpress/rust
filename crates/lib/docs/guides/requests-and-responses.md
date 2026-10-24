# Construct requests and prepare responses

An adapter needs to separate buffered values from a native reader or writer. Build the values first;
give each I/O boundary one callback attempt and inspect its state afterward.

## Build input with explicit precedence

Inside a function returning `stackpress_lib::Result<()>`, construct overlapping query, post, and
operation data:

```rust
use stackpress_lib::{Request, Response};

let mut req = Request::builder()
    .url("/profiles?name=query")
    .post(serde_json::json!({"name": "post"}))
    .data(serde_json::json!({"name": "Ada"}))
    .build()?;
assert_eq!(req.data.path_as::<String>("name")?, Some("Ada".into()));
assert_eq!(req.query.path_as::<String>("name")?, Some("query".into()));
```

Aggregate data overlays query → post → explicit data, while source stores stay separate. The
relative URL receives the unknown-host base; constructing it performs no network access. A buffered
body and MIME do not automatically parse into post data.

## Commit one loader result

A loader receives an owned snapshot and returns only the body/post changes to install. Add this
named loader declaration:

```rust
use stackpress_lib::{Body, LoadResult, Nest, RequestView, Result};

async fn read_form(_: RequestView) -> Result<LoadResult> {
    // A real adapter can acquire bytes here, then return validated staged fields.
    Ok(LoadResult {
        body: Some(Body::Text("name=Grace".into())),
        post: Some(Nest::from_object(serde_json::json!({"name": "Grace"}))?),
    })
}
```

This example returns buffered fixture data so it needs no transport. Snapshot edits alone would not
change the original request. In the async caller, install and verify the returned fields:

```rust
req.load_with(read_form).await?;
assert_eq!(req.data.path_as::<String>("name")?, Some("Grace".into()));
assert!(req.loaded());
```

Loaded post overlays both post and aggregate data, including previously explicit fields. Validation
completes before these outputs commit. A successful repeated load is a no-op; a callback/commit
error records `Failed`, and cancellation after entry leaves `Attempted`. Both states reject another
load.

## Prepare a result before delivery

Choose structured results for capability output and JSON text when you need a text body:

```rust
let mut res = Response::new();
res.rows(&[serde_json::json!({"id": 42})], 1)?;
res.status_code(200, None)?;
assert_eq!(res.to_envelope().try_to_json()?["total"], 1);
```

`rows` requires an array serialization and preserves a supplied total, including zero. `results`
instead sets total 1. These helpers set status 200, so put a desired status override afterward.
`json` stores pretty JSON text; `results` retains a structured `Value`.

For native delivery, supply a named callback with
`async fn write(view: ResponseView) -> Result<()>`, then await `res.dispatch_with(write)`. It
receives a data-only snapshot. `dispatch().await` finalizes without output; `stop()` records
explicit suppression without a writer.

## Read attempt state accurately

Use `load_state()` and `delivery_state()` when distinguishing success from interruption. The state
transitions are:

| Operation | Before callback | Success | Ordinary callback failure | Dropped while pending |
| --- | --- | --- | --- | --- |
| Load | `Attempted` | `Loaded` | `Failed` | `Attempted` |
| Delivery | `Attempted` | `Sent` | `Failed` | `Attempted` |

Delivery validation happens before the attempt transition; a validation failure there leaves output
open. After any delivery attempt, mutations and guarded mutable accessors fail. Repeating dispatch
after `Sent` or `Stopped` is a no-op. `sent()` includes `Stopped`, so use the enum to establish
whether a writer completed.

You can now trace precedence, staged loading, and output ownership separately. See
[request API](../api/request.md), [response API](../api/response.md), and
[headers/body API](../api/headers-and-body.md) for exact signatures and state guards.
