# Generated SQL data layer

Start with [getting started](getting-started.md), then [executor and result concepts](concepts.md). Use the [read guide](guides/read-and-include.md) for pages/relations and the [operation guide](guides/register-operations.md) for an explicit host boundary. The [API index](api/README.md) lists the native runtime and emitted surfaces.

M2 executes generated CRUD, queries, relations and selected handlers on SQLite. PostgreSQL conflict SQL has renderer tests; network generated-CRUD integration remains a later SQL-lane checkpoint. MySQL targeted conflicts and generated RETURNING writes fail before submission. M3 adds explicit immutable history, reviewed plans and SQLite reconstruction/operator tooling; see the [migration API and operator reference](api/migrations.md) for safety contracts and backend limits.

The accepted shared changes are recorded in the [upsert proposal](m2-upsert-proposal.md) and [executor-limit proposal](m2-executor-limits-proposal.md). Those proposal pages preserve the original reasoning, not current milestone status. The coordinator's M2 receipt records final verification.
