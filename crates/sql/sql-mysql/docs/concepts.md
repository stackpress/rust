# Owners, leases and observation

An Engine owns one MySQL database adapter. Each ordinary operation acquires a connection
lease; a transaction retains one lease until consuming completion or drop. A lease owns both
native capacity and an application admission permit, including while cleanup is pending.

## Two bounds control concurrency

`max_in_flight` counts admitted calls, pool waiters and held transactions. Admission is
immediate: excess work receives `ErrorKind::Limit`. Admitted work can wait for at most
`acquire_timeout_ms` for one of `max_connections` native connections; that wait can return
`PoolTimedOut`. Cancellation releases waiting admission. Closing wakes waiters with `Closed`.

The pool minimum is zero. Eager open proves one connection can be established; it does not
preallocate the maximum. Share `Arc<Engine>` and bound your application's tasks as well.
These settings are not a request queue or a server-wide connection limit.

## Transactions retain exclusive ownership

`Engine::begin` returns only after BEGIN is acknowledged. Pass `&mut Transaction` to helpers
to keep all their statements on that lease. An unrelated Engine call does not join it.
Acquiring again while holding the only connection can time out.

After a session run has been polled, failure or cancellation leaves it rollback-only:
later runs and commit return `RollbackOnly`. Explicit rollback remains allowed. Engine
rendering/validation can fail before reaching the session, so a preflight error alone does
not prove the native transaction was poisoned. See [the transaction guide](guides/transactions.md).

## Reuse requires acknowledged completion

SQLx 0.9.0 has no public MySQL session reset, and Query does not record builder/raw provenance.
The adapter therefore reuses only an acknowledged, lexically allowlisted stateless subset,
followed by an acknowledged ROLLBACK safety command. It retires DDL, CTEs, unknown functions
and other unclassified/session-affecting work, even after success. This is a coordinator
decision; no driver fork is used.

ROLLBACK is not a general session reset. The rule assumes reusable DML has no hidden
session-mutating triggers or routines; the adapter does not inspect their definitions.
Applications with such effects need a separate connection boundary. The exact
[reuse allowlist](api/database.md#lease-reuse-allowlist) is intentionally conservative.

Errors, cancellation, incomplete rows, uncertain transaction control and failed cleanup
retire the connection. An uncertain transaction remains retired even after an acknowledged
rollback. A fresh lease cannot inherit that connection's temporary state.

## Cancellation loses observation

A write or COMMIT may already have succeeded when its future is dropped or its deadline
expires. Cleanup failure after a successful write can also turn the call into an error.
There is no automatic retry, exactly-once promise or claim that an unobserved commit rolled
back. Reconcile application state before deciding whether to retry.

Reset and retirement each have their own finite cleanup budget. Background retirement retains
admission, is registered before its first poll and is observed by database close. Local socket
disposal does not promise immediate remote session termination.

## Shutdown belongs to the application

Stop new application work, finish or drop transactions, await application tasks, then await
`Engine::close` while Tokio is alive. Polling close shuts admission and waits for held and
retiring connections. A caller-held transaction can keep close waiting; close has no overall
deadline of its own. Give application shutdown a separate deadline.

Interrupted close may be awaited again; the database never reopens. The first recorded
background cleanup failure is returned once. Outside a runtime, drop disposes sockets by
RAII without spawning cleanup or claiming a remote rollback acknowledgement.
