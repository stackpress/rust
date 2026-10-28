# Await bounded shutdown

Close HTTP admission before closing process services. Awaiting the serving
future after its stop input resolves ensures active connections have completed,
or have been aborted and reaped, while those services and the runtime remain alive.

## Use the application stop input

BoundHttpServer::serve_until takes any Send future returning unit. The
[getting-started main](../getting-started.md) wires process signals directly;
the [deployment example](../../examples/deployment.rs) also handles SIGTERM on
Unix. Keep signal policy in the application. The library installs no process
handler and starts no runtime.

The result is a DrainReport, not an outer Result. Inspect `failure` after cleanup;
a fatal accept error still follows the same drain path. Explicitly await each
owned service's close operation before returning from main, including when
serving failed. Dropping Arc<Server> alone does not invoke an async close hook.

## Order stop and cleanup

A ready stop wins over a ready accept. Admission and stop share the last-spawn
boundary; a connection accepted after that boundary cannot acquire a task.
Connection and handshake permits are acquired before spawn or TLS work;
excess IO closes promptly. Stop is sticky and repeated stop is harmless.

The source/listener and owned backlog drop first. Active connections begin
graceful shutdown together under one absolute `shutdown_grace`, default 30 s.
Overdue tasks are aborted together and every join handle is reaped before
return. This is a group deadline, not another full grace per connection.

With a generic acceptor, use a cloned `Shutdown` with
`Arc<HttpServer>::serve_acceptor`, or use `serve_acceptor_until` with a future.
An Acceptor must be cancellation-safe and must not hide detached work; the
caller owns any backlog bounds. See [IO/supervision reference](../api/host.md).

## Read the final report

`admitted`, `rejected`, `completed`, `aborted`, and `task_failures` retain totals.
Completed tasks may contain connection failures; they are not necessarily
successful exchanges. Recent ConnectionReports are bounded by max_connections;
`discarded_reports` counts older history. See [reports](../api/reports.md) for
publication, native flush, and explicit diagnostic ownership.

Dropping the supervisor aborts owned tasks; awaiting shutdown additionally
establishes reaping before service close. Async cancellation cannot preempt a
blocking or CPU-bound user poll. Application handlers and observers must
cooperate; no process-abort cleanup guarantee is provided.
