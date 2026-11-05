# Generated API and session model family

`codegen::config()` composes the unmodified `stackpress-api/schema.idea`, including
`stackpress-session/schema.idea`. `codegen::normalize()` returns SourceGraph for all four
models. The old three-model Typed projection and synthetic Profile are removed. Applications
may import `stackpress/stackpress.idea` and append a Profile column under G19.

## Register the application family

`sql::register_with<M: ApiModels>(&mut ServerBuilder) -> Result<()>` contributes the same
Application/Session event-major inventory using actual associated SourceActions. ApiModels
inherits SessionModels, so auth and API share the exact Profile. The ordinary API process
`register_with<M>` selects the same process registrar; it resolves named model events and
installs neither data listeners nor tables. Default functions keep their stock-family paths.

`views::admin::register_with<M>` registers selected module routes immediately;
`register_queued_with<M>` queues them at Route for source aggregate order. Both take the
compiled portable Ui callbacks, Config and selected module names. The application supplies
its model catalog presentation and actual family. Native Dates are converted only at this
page/wire boundary. Every handler retains source CSRF, error and no-check behavior.

## Generate native and portable partitions

The [API checkpoint tool](../../../../examples/api-checkpoint/tool/src/main.rs) emits the
full source/stores/UI graph through public generators and the managed writer. Its native
web crate imports schemas/stores; its shared module and separate hydrate/CSR client import
only ui. Stored API secrets remain visible wherever the original source displays them;
private codec seeds/provider metadata never enter that portable catalog or SSR seed.
The app opens and installs databases explicitly; generation and registration do no IO.

`models::{ApplicationSchema,SessionSchema,ApplicationStore,SessionStore,ApplicationActions,SessionActions}`
keep constructor/path adapters backed by generated defaults. RecordSchema/SourceColumn
use columns()/column()/describe(). Under G53, `eq` takes one value per field; arrays are misuse
and retain source indexed-path AND semantics. Multi-value `has` requires all values in the
list column on SQLite, PostgreSQL and MySQL; scalar SQL is unchanged. `hasnt` still follows
the source's positive membership and indexed-array behavior; G53 does not change it.
The lower G46/G48 dialect/setup rules and original source oracles remain unchanged.

Evidence: [extended generated handlers](../tests/generated_family.rs), [default SQL tests](../tests/sql.rs),
[original management matrix](../tests/management.rs) and the [finite checkpoint host](../../../../examples/api-checkpoint/web/src/lib.rs).
