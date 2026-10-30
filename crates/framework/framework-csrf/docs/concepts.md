# Incoming and outgoing CSRF state

A generated token belongs to a response; a submitted token belongs to a later request. Keeping those two moments separate explains why generating a token does not validate the current request automatically.

## The three operations

The default field name is `csrf`. All operations use the same selected name, but they read or mutate different carriers.

| Operation | Incoming request | Outgoing response |
| --- | --- | --- |
| `generate` | Does not read it | Stages a session cookie, then data `{name, token}` |
| `valid` | Reads scalar session cookie and aggregate data strings | On mismatch, sets 419 / Page Expired with the configured explanation |
| `clear` | Removes the aggregate data field; retains incoming cookie and query/post snapshots | Removes staged cookie and form projection |

The [quick start](getting-started.md) constructs both moments explicitly. In a rendered form, put the projected `name` and `token` into a hidden input; the browser supplies its cookie independently on submission.

## Who chooses validation

Direct use needs only `Service`. With `server`, the CSRF registrar publishes a default/configured service even when the CSRF section is missing or nonobject. It contributes no route, request listener or global guard. Routes and events call `valid` themselves and decide whether to stop their process after false.

Validation has no built-in method, Origin, expiry or hex-format requirement. A successful comparison does not clear state. These are preserved source choices; application processes decide their own checks and clear point.

## Partial mutation and errors

Generation stages the cookie before form data. Clear removes request data before outgoing cookie/data. Existing carrier, cookie-name or response-state errors propagate in that order, so a later error can leave earlier mutations in place. The service adds no transaction or rollback.

The model is a stateless mechanism over request/response carriers. Use [protect a process](guides/protect-a-process.md) for explicit control flow and [source behavior](source-behavior.md) for exact comparison and failure details.
