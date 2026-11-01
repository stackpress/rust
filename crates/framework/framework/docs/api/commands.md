# Command bridges and Bindings

The command layer preserves source event/script inputs and response behaviour over explicit native capabilities. It supplies no listener, watcher, console, bundler or process supervisor implementation. [Finite binding example](../guides/commands.md) shows a complete callable host.

## Bindings and direct invocation

`Bindings::default()` creates an empty map. `bind(name: &str, action: impl Handler + 'static)` stores a compiled handler; a later binding of the same name replaces it. It does not execute the action.

`Bindings::invoke(name, &mut Request, &mut Response, Invocation<'_>) -> Result<Flow>` invokes the selected handler. **Returns** its flow/error unchanged, or `native command binding is unavailable`. `commands::server::invoke(name, serde_json::Value, res, inv)` makes a separate owned options request before calling that binding.

`register_server(app)` queues develop → emit → serve for Listen. `register_sql(app)` queues install → migrate → purge → push → query → populate → uninstall → upgrade. Schema registration queues generate; source-present view queues build. No command runs during these registrations.

## Event contracts

An earlier response status other than absent/zero/200 skips serve, emit, generate and SQL events. Develop and build deliberately proceed. Public wrapper errors use response code 500 with source text and empty errors; thrown provider/type/SQL failures remain Result errors.

| Event | Input / execution / result |
| --- | --- |
| develop | Environment argv/cwd plus server.develop; nullish defaults: debounce 100, extensions [.ts], ignore [], timeout 10000, watch [.]. Zero/false/empty values remain supplied values. Calls develop binding, then stages 200; no prior-status skip. |
| emit | Requires argv[3], else Missing event name. Joins/reparses remaining argv text, raw-emits child event, ignores local child Abort, calls output with pretty response text, then stages 200. |
| serve | host defaults 127.0.0.1, port 3000; calls serve binding, then stages 200, preserving returned Flow. |
| generate | Skips absent explicit client key; a present client without truthy tsconfig stages Missing tsconfig path. Resolves Options, calls generate binding, then stages 200. |
| install | Requires Engine and compiled client/Scripts; uninstall then install in one transaction; optional revision insert follows commit. |
| migrate | Requires Engine; missing revisions or migrations path skips script work. Writes every nonempty plan in order; warning/force does not refuse migration. |
| purge | Requires Engine and compiled client/Scripts; transaction executes purge in create order. SQLite uses accepted DELETE mapping. |
| push | Requires Engine; absent revisions skips; fewer than two revisions delegates install, otherwise upgrade. Scripts are obtained lazily. |
| query | Event-only command: requires Engine, takes last nonempty argument from argv[2..], replaces backslashes with single quotes, executes trusted native SQL and passes owned QueryResult to output. Empty input skips query/output. |
| populate | Configured event/data array, default []; raw sequential child dispatch, independent child responses, no Engine/client required; child non-OK status does not gate the next entry. |
| uninstall | Requires Engine and compiled client/Scripts; reverse-create-order IF EXISTS drops in one transaction. Does not delete revision history. |
| upgrade | Requires Engine; absent root/no adjacent plan skips. Uses latest adjacent plan, refusing a nonempty warning unless force; executes nonempty queries transactionally. |
| build | Source-present view only; ignores earlier status, awaits selected build stages, submits optional packaging without awaiting it, then stages 200. Build script results are not copied into event response. |

All SQL wrappers except populate stage `No database found` when Engine is missing, before script-specific skips. Successful wrappers stage 200, including script skips. See [SQL reference](sql.md) for exact resource/transaction contracts and [view reference](view.md) for packaging timing.

## Direct server scripts

`server::serve`, `develop`, `emit` share the event handler signature `(req, res, inv) -> Result<Flow>`. `emit_script(argv, res, inv) -> Result<()>` defaults to skip=2, while the event wrapper uses skip=3. `emit_with_skip(argv, skip, res, inv)` exposes the explicit prefix length; direct scripts do not add the event wrapper's final 200 status.

Arguments are joined before parsing, preserving the source Terminal boundary. Populate and emit reuse ordinary events rather than introducing another handled request lifecycle. Evidence: [commands implementation](../../src/native/commands.rs), [server functions](../../src/native/commands/server.rs), [input/skip/error tests](../../tests/cases/commands.rs), [script oracle](../../tests/fixtures/source-scripts.json), [command oracle](../../tests/fixtures/source-commands.json).
