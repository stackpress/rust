# Stackpress framework aggregate

Compose the selected Stackpress plugins in source order with one ordinary registration function. Use this crate when an application wants the standard composition; each constituent remains independently usable. Features are selected before compilation, and disabling one requires rebuilding the application.

Start with [your first aggregate event](docs/getting-started.md), then read the [composition model](docs/concepts.md). The aggregate contributes no global guard or route ownership arbitration: several matching handlers may run, in priority then registration order, and any may Abort. Plugins and routes secure their own configured processes, following G42/G43.

## Choose a reading path

Pick the task you need now; the reference pages contain signatures and detailed behaviour.

| Task | Read |
| --- | --- |
| Register an application and compiled models | [Registration guide](docs/guides/registration.md) |
| Select native, facade or browser features | [Feature reference](docs/api/features.md) |
| Bind commands to application IO | [Command guide](docs/guides/commands.md) |
| Supply source configuration | [Configuration reference](docs/api/configuration.md) |
| Render and build shared UI | [Website guide](docs/guides/website.md) |
| Generate application code | [Build-host guide](docs/guides/codegen.md) |
| Start from an existing application graph | [Eight consumer recipes](docs/guides/consumers.md) |
| Look up a public surface | [API index](docs/api/README.md) |
| Understand source edges and evidence limits | [Source behaviour](docs/source-behavior.md), [verification](docs/verification.md) |

## Implementation and evidence

The nine stages are server, schema, language, CSRF, SQL, view, session, API and admin. Session itself composes auth → email → identity. `default = []`; `full` selects the runtime stages without choosing a database connector, renderer or code generator. Native command execution remains application-owned.

The [source ledger](proofs/coverage-map.json) maps inventory/export/risk rows to tests and named limits. [Doc claims](docs/claims.json) link these pages to current source and tests; [the checker](docs/check.py) also compiles exact Markdown examples outside the repository using unchanged consumer locks. See [verification](docs/verification.md) for the evidence boundary.

## Application-generated source families

The [source-client recipe](docs/guides/registration.md#generate-an-extended-source-client)
uses the shipped Idea imports and the public source emitter. CompiledClient.bindings
selects monomorphized session/API registrars for the actual application family, while
model listeners remain in SQL Listen and generated admin stays last. The default facade
and no-feature graphs are unchanged. Native schemas/stores/client stay separate from
portable UI in SSR/hydrate/CSR artifacts; no codec state or native provider enters a seed.
