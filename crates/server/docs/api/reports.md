# Lifecycle report API

Ordered diagnostics remain separate from public response and native delivery. Sources:
[report.rs](../../src/report.rs), [lifecycle.rs](../../src/lifecycle.rs).

```rust
use stackpress_lib::{Flow, Invocation, Method, Request, Response, Result};
use stackpress_server::{HandleOptions, ServerBuilder};

async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
let mut app = ServerBuilder::new();
app.on("greet", greet)?;
app.get("/hello", greet)?;
let server = app.build().await?;
let mut reply = server.call("greet", serde_json::json!({})).await?;
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Report data](#1-report-data) · [2. Enums and correlation](#2-enums-and-correlation) ·
[3. Report operations](#3-report-operations)

## 1. Report data

HostReply (Debug) has public `response: Response` and `report: HandleReport`. HandleReport (Debug)
has the following fields; inspecting or changing them does not grant native output authority.

| Field | Type | Meaning |
| --- | --- | --- |
| correlation_id | CorrelationId | One checked per-host allocation. |
| event | String | Selected event, or fixed invalid-event marker. |
| outcome | LifecycleOutcome | First failure, otherwise first Abort/finalized entry/completion. |
| stages | Vec<StageRecord> | Actual execution observations in order. |
| failures | Vec<StageFailure> | Original and secondary errors retained in order. |
| error_offers | Vec<ErrorOffer> | Actual offers referencing an original failure index. |
| operation | Option<DispatchOutcome> | Raw operation outcome, None when skipped/failed. |
| output_eligible | bool | Snapshot: output was Open. |
| terminal_failure | bool | Terminal error/output condition prevents cosmetic recovery. |

StageRecord is Clone/Debug with `stage: LifecycleStage`, `event: Option<String>`, `outcome:
StageOutcome`, `delivery_state: DeliveryState`. StageFailure is Clone with public `stage:
LifecycleStage`, `error: Arc<Error>`; Clone shares that diagnostic allocation. Its Debug omits cause
messages. ErrorOffer is Clone/Debug with `failure: usize` (index into failures) and `outcome:
ErrorHandling`. Setup failure creates no invented Error offer.

## 2. Enums and correlation

LifecycleStage is Input, Request, Operation, Response, Error, Output, Dispatch. LifecycleOutcome is
Completed, Aborted { stage }, Failed { stage }, Finalized { stage }. ErrorHandling is NoListener,
Completed, Aborted, Failed. StageOutcome is Completed, NoListener, Aborted, Failed, Finalized. These
enums are Clone/Copy/Debug/Eq/PartialEq.

CorrelationId is Clone/Debug/Eq/PartialEq/Hash and Display with no public constructor/field. IDs
start at "1" per Server, or "prefix:1". They can repeat across hosts and are not authentication or
distributed trace identity. Checked exhaustion is an outer handling Error before input/callbacks.

### 2.1. CorrelationId::as_str

`as_str(&self) -> &str`

Borrow the printable local representation.

```rust
assert!(!reply.report.correlation_id.as_str().is_empty());
```

**Returns** Borrowed ID text, without allocation.

## 3. Report operations

Public success may coexist with diagnostic Failed after deliberate recovery. Default report Display
prints event/correlation/stage/listener/site metadata; it does not automatically dump input,
sessions, config, captures, service payloads, or developer cause messages.

### 3.1. is_success

`is_success(&self, response: &Response) -> bool`

Evaluate final public success rather than diagnostic outcome.

```rust
assert!(reply.report.is_success(&reply.response));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `response` | `&Response` | The corresponding final response. |

**Returns** True for status 200–399 with no public error, no terminal failure, and neither Failed
nor Attempted delivery. Open output may qualify; this is not proof of a native send.

### 3.2. developer

`developer(&self) -> DeveloperReport<'_>`

Opt into developer messages and cause formatting.

```rust
let diagnostic = reply.report.developer().to_string();
assert!(diagnostic.contains("greet"));
```

**Returns** A borrowing Display wrapper. Formatting includes retained messages and up to 32 causes
per failure, bounding cyclic source traversal. This is explicit disclosure, not automatic logging.

### 3.3. record_output

`record_output(&mut self, response: &Response)`

Record adapter success/suppression after its sole attempt.

```rust
reply.response.stop()?;
reply.report.record_output(&reply.response);
assert!(!reply.report.output_eligible);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `response` | `&Response` | Current delivery-state snapshot from the adapter. |

**Returns** Unit; appends Output/Completed and updates eligibility. It does not send, suppress, or
acquire output authority itself.

### 3.4. record_output_failure

`record_output_failure(&mut self, error: Error, response: &Response)`

Retain conversion/write failure without running hooks again.

```rust
reply
    .report
    .record_output_failure(stackpress_lib::Error::new("writer failed"), &reply.response);
assert!(!reply.report.is_success(&reply.response));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `error` | `Error` | Owned original output cause. |
| `response` | `&Response` | Delivery snapshot, potentially partial or already Sent. |

**Returns** Unit; retains an Output failure, appends Output/Failed, and marks terminal failure. No
Response mutation, hook dispatch, or native retry occurs.

Related: [errors guide](../guides/errors-and-recovery.md), [output guard](output.md),
[Server](server.md).
