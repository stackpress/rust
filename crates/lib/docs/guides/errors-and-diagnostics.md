# Return errors and inspect diagnostics

An internal failure can contain a useful cause and still need a deliberate public message. Keep
those views separate so a handler can propagate diagnostics without copying developer details into
its result.

## Create a public failure deliberately

Inside a function returning `stackpress_lib::Result<()>`, wrap a real conversion failure:

```rust
use stackpress_lib::{Error, Response};

let cause = "invalid"
    .parse::<u32>()
    .expect_err("invalid integer fixture");
let error = Error::from_source(cause)
    .with_code(400)
    .with_public_message("Count must be an integer")
    .with_errors(serde_json::json!({"count": ["must be an integer"]}));
let mut res = Response::new();
res.apply_error(error)?;
assert_eq!(
    res.try_to_json()?["error"],
    "Count must be an integer"
);
assert!(res.diagnostic().is_some());
```

`apply_error` owns the original error and stores its public view. The envelope omits the original
cause, origin, invocation path, and native backtrace. `into_error` can later move the retained
original error back out of the response.

`Error::new` and `Error::from_source` default to code 500 and public text `Internal Server Error`.
`Error::invalid` deliberately exposes its supplied message with code 400; `Error::limit` uses that
same public policy with kind `Limit`. Choose wording accordingly.

## Propagate handler failures

Return `Result<Flow>` from named handlers and use `?` for failure propagation. This handler rejects
absent operation input explicitly:

```rust
use stackpress_lib::{Flow, Invocation, Request, Result};

async fn require_count(req: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A typed read reports an incompatible present value before this absence check.
    let _count = req
        .data
        .path_as::<u64>("count")?
        .ok_or_else(|| Error::invalid("Count is required"))?;
    Ok(Flow::Continue)
}
```

A failed phase stops remaining phases/listeners. Nested propagation keeps the deepest complete
invocation path and original cause. It does not invoke a general `error` event or undo earlier
mutations.

## Inspect the developer view

At the call boundary, inspect `message`, `kind`, `origin`, `path`, optional `span`, and `frames`.
Each frame identifies its event, matcher, listener, label, plugin, priority, phase, and captured
registration sites. `std::error::Error::source` gives the owned original cause for ordinary
error-chain tools.

`Display` and `Debug` are developer views; use `public_view` for selected public fields. Native
backtrace capture depends on the runtime environment, while structured origins and invocation frames
remain available independently. `set_stack` is a separate explicit public disclosure, never an
automatic copy of diagnostic frames.

You can now preserve useful failure context while choosing the public result yourself. Consult
[error API](../api/error.md) for builders and field semantics, [response API](../api/response.md)
for conversion, and [status API](../api/status.md) for application codes.
