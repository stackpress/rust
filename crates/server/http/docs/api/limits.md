# Limits, deadlines and statuses

NetworkLimits is Clone/Debug with finite native defaults. Edit
`NetworkOptions::limits` before preparing the host, or use
`HttpServer::with_limits` before sharing it. Source: [options](../../src/options.rs),
[input](../../src/input.rs), [connection](../../src/connection.rs).

## Counts and bytes

All ceilings include equality. Zero is invalid, never unlimited. Handshake
capacity cannot exceed connection capacity; connection counts must fit Tokio
semaphores. Header count must not exceed parser_buffer_bytes.

| Field | Default | Charged scope |
| --- | --- | --- |
| max_connections | 256 | Owned connections, including handshakes; acquired before spawn. |
| max_handshakes | 32 | Concurrent TLS handshakes, inside connection budget. |
| max_headers | 64 | Raw parser header occurrences, plus retained head check. |
| parser_buffer_bytes | 32 KiB | Hyper read/write buffer; at least 8192 bytes. |
| max_header_bytes | 16 KiB | Retained normalized header names/values, counting repeats. |
| max_target_bytes | 8 KiB | Native request target. |
| max_body_bytes | 1 MiB | Known and actual decoded body bytes, before retained copies. |
| max_request_bytes | 2 MiB | Method + target + retained headers + decoded body, excluding raw framing overhead. |
| max_requests | 100 | Exchanges per connection, including rejected requests. |

HttpOptions and Lib ParseLimits independently bound normalized values, parsing,
and responses. Raising a native limit does not raise those conversion limits.
The host does not impose a whole-process memory ceiling on user, TLS, or kernel
allocations.

## Time controls

Every duration must be positive and fit the monotonic clock. Progress never
extends an absolute deadline, so a one-byte trickle cannot keep work alive
indefinitely. TLS and connection age start at admission.

| Field | Default | Deadline ownership |
| --- | --- | --- |
| connection_lifetime | 300 s | Absolute age; expiry begins bounded graceful drain. |
| handshake_timeout | 5 s | Absolute TLS handshake, capped by connection age. |
| head_timeout | 10 s | Absolute head budget, including Hyper read-ahead partial heads. |
| head_idle_timeout | 5 s | Head read progress interval. |
| body_timeout | 30 s | Absolute decoded-body load inside Input. |
| body_idle_timeout | 5 s | Interval without decoded body progress. |
| lifecycle_timeout | 30 s | Absolute computation after loading completes or fails. |
| response_timeout | 60 s | Complete output lifetime, including slow buffered writes. |
| write_idle_timeout | 10 s | Interval without actual native writes. |
| keep_alive_idle | 15 s | Quiet reusable connection. |
| shutdown_grace | 30 s | One group drain budget, then abort/reap. |

A stream's total_timeout/idle_timeout further tighten output budgets; actual
native writes, rather than producer polling alone, establish write progress.
Hyper's timer owns parsing boundaries, including a partial successor already
buffered after an earlier request. No raw bytes are reparsed. Async clocks
cannot interrupt a blocking user poll or synchronous startup filesystem read.

## Boundary results

These native statuses do not change the buffered adapter's existing contract.
A selected status in a report is not a guarantee that bytes could be written.

| Condition | Status and lifecycle behavior |
| --- | --- |
| Malformed normalization before a Request exists (G17) | Sanitized 400 and Input-stage HandleReport; no request/error/response callbacks. |
| Visible CL+TE, unsupported framing, authority/proxy mismatch | Empty 400 and close before routes. |
| Hyper malformed head/framing | Hyper's safe error response or close; no route. |
| Header count/bytes, target, head aggregate excess | 431 before routes; native parser may close on failure. |
| Known/actual decoded-body or body aggregate excess | 413 through the once-only Input error offer (G12); no selected operation on failed input. |
| Body deadline | 408 through Input's G12 error policy. |
| Head or lifecycle deadline before publication | Empty 408 when this host still owns an unpublished writer; Hyper's parser timeout may close instead. |
| Excess admission before parsing/TLS | Close IO promptly with report-only 503; no fabricated wire response. |
| Output conversion failure before publication | One fixed empty 500, retained Output diagnostic; no retry/hooks replay. |
| Producer/write failure or deadline after publication | Close output, retain failure; no second response or appended JSON. |

Normalized loading failures permit the existing Error/Response recovery policy;
handlers may replace the public failure response. G17 has no normalized Request
to offer to callbacks. See [server error rules](../../../server/docs/guides/errors-and-recovery.md)
and the [G31 framing policy](../concepts.md#framing-and-connection-reuse).
