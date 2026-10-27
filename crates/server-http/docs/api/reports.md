# Reports and observation API

Reports distinguish lifecycle publication from local delivery completion.
Default Debug redacts native cause text; explicit failure fields retain original
typed diagnostics. Source: [reports](../../src/report.rs),
[supervisor](../../src/supervisor.rs), and [observer](../../src/observe.rs).

## Delivery and exchanges

HttpDelivery is Clone/Copy/Debug/Eq/PartialEq:

| Variant | Meaning |
| --- | --- |
| RejectedBeforeHost | Parser/boundary/admission rejection without a completed lifecycle. |
| Suppressed | Framework explicitly stopped output. |
| Published | Adapter moved output; producer/native flush not yet complete. |
| Completed | Producer handoff and native flush completed; no remote ACK claim. |
| Failed | Producer, conversion or native failure; no retry. |
| Cancelled | Active loading, handling or delivery dropped/cancelled. |

HttpExchangeReport holds `exchange_id: u64` (connection-local),
`host: Option<HandleReport>`, `delivery: HttpDelivery`, `bytes: u64`,
`elapsed: Duration`, `failure: Option<Arc<Error>>`, `status: Option<u16>`.
Bytes count handoff to Hyper, not remote acknowledgement. The first original
failure is retained. A status may describe admission's close-only 503.
Host is absent when lifecycle handling never completed.

ConnectionReport returns only after active connection ownership ends. Fields
are ordered `exchanges: Vec<HttpExchangeReport>`, original
`failure: Option<Arc<Error>>`, `shutdown: bool` (caller stop initiated), and
`forced: bool` (grace expired and work dropped). Earlier settled exchanges stay
complete if a later request fails.

## DrainReport

The supervisor returns one final aggregate after task completion or abort/reap.
No outer Result hides cleanup on fatal acceptance failure.

| Field | Meaning |
| --- | --- |
| admitted: usize | All tasks admitted before stop. |
| rejected: usize | Excess IO closed before spawn/handshake. |
| completed: usize | Tasks observed completed, including errors. |
| aborted: usize | Tasks forcibly aborted/reaped at the common deadline. |
| task_failures: usize | Task panics/cancellations, separate from ordinary connection diagnostics. |
| reports: VecDeque<ConnectionReport> | Recent reports, at most max_connections. |
| discarded_reports: usize | Older reports omitted from bounded history. |
| failure: Option<Arc<Error>> | Original fatal accept/task failure, retained after cleanup. |

Counters retain totals after history is discarded. Each connection's exchange
history is bounded by request limits. See [shutdown](../guides/shutdown.md).

## ExchangeObserver

`ExchangeObserver: Send + Sync` requires
`observe(&self, observation: ExchangeObservation)`. Any compatible Fn closure
implements it. Attach Arc<dyn ExchangeObserver> through NetworkOptions or
with_observer before preparation.

ExchangeObservation is Clone/Debug. Fields: `connection_id: u64`,
`exchange_id: u64`, `status: Option<u16>`, `delivery: HttpDelivery`, `bytes: u64`,
`elapsed: Duration`, `error_kind: Option<ErrorKind>`, `host_report: bool`.
Only local IDs and final fixed-size metadata are included. No path, headers,
cookies, secrets, address, payload, SQL or native cause text is present.

Callbacks occur at connection ownership end, outside internal locks after
active IO/future/permits drop. No queue/sender task is created. Callbacks must
not block; caller retention/forwarding needs its own bound. Panic unwinds and
is not retried; Drop already unwinding skips callbacks. The
[observer guide](../guides/observers.md) shows a bounded counter.
