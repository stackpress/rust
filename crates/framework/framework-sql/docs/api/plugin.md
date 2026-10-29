# Native SQL plugin

Enable framework-sql's native `server` feature. Select
`stackpress_framework_sql::plugin::register` with `ServerBuilder::plugin`; its exact signature
is `fn(&mut ServerBuilder) -> stackpress_lib::Result<()>`.
See [selection and migration](../guides/select-plugins.md) for complete examples and direct use.

## Inputs and startup

| Input | Contract |
| --- | --- |
| Engine | Existing `stackpress_sql::Engine` service; Config verifies it without opening or querying it |
| Policy | `operations::Policy` service supplied before SQL Config; every protected handler authorizes each invocation |
| Contributions | One service supplied during Config; `operations: &'static [plugin::RegisterOperations]`, length 1–128 |
| Callback | `fn(&mut StartupContext, &mut operations::Registration) -> stackpress_lib::Result<()>`; chooses real generated operations |

Config claims `plugin::Service`, whose construction is private. It proves explicit SQL
selection and verified dependencies; it grants no operation authority. Consumers such as the
generated admin host require this service. Supplying Engine and Policy without selecting SQL
integration does not satisfy that contract.

Listen reads Contributions and validates its count before any contribution executes. It runs
the static callbacks in order, sharing one Registration marker. The marker rejects conflicting
generated operation selections across families. Listeners registered inside those callbacks
retain the selected SQL Listen task's plugin identity; function calls do not select another
plugin. Generated page routes retain their selected page-host identity. The registrar contributes
no routes or typed configuration sections. It never creates an Engine, runs queries or database setup, closes a
resource, or binds a listener. Cancellation does not start detached work.

## Errors and composition

Missing/wrong-order services, repeated singleton selection, absent/empty/oversized inventories,
operation conflicts and callback errors use the existing builder/startup Result contracts.
A failed build publishes no Server. Earlier callback effects are not transactional and later
callbacks do not run after failure. Keep callbacks limited to registration.

A generated page-host registrar supplies the one Contributions service itself. To combine its
operations with another family, supply a single aggregated Contributions service and call the
generated direct route helper, instead of selecting the generated page-host registrar too.
Direct APIs need no opaque Service; see the [operation guide](../guides/register-operations.md).
