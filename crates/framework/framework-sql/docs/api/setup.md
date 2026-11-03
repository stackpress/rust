# Explicit source schema setup

`stackpress_framework_sql::setup` selects G48 setup semantics without changing ordinary
Engine/Transaction contracts. Call it explicitly with an application-owned Engine;
registration never installs a schema. [Implementation](../../src/setup.rs).

| Function | Input | Result and effect |
| --- | --- | --- |
| `execute_plan` | `&Engine, &QueryPlan` | `Result<Vec<QueryResult>>`: MySQL validates the plan then runs its statements non-atomically; PostgreSQL/SQLite retain Engine::execute_plan. |
| `execute` | `&Engine, &[Query]` | Same ordered result: MySQL runs outside a transaction; PostgreSQL/SQLite own one transaction and roll back after the first failure. Empty scripts are valid. |

Errors retain their original SQL kind/display/cause. On MySQL, `plan_progress()` reports
the acknowledged prefix and failed index, not rollback evidence. The failed statement
can have executed without acknowledgement; cancellation returns no report and can leave
partial state. Nothing retries, repairs or adopts a partial installation automatically.

Session/API install and upgrade, and aggregate install/purge/uninstall/upgrade/push,
choose this setup policy. Aggregate install publishes its source revision only after
complete setup success; uninstall does not delete history. Its public `transaction`
function remains atomic. [Atomic policy tests](../../tests/setup.rs) and the
[owned live proofs](../../../framework/tests/live_setup.rs) pin these boundaries.

G27's [SQLite migration runner](migrations.md) remains SQLite-only. G48 does not add a
MySQL runner, target lock, drift detector or generated-workspace ledger. MySQL source
upgrades use the aggregate's existing adjacent-migration command.
