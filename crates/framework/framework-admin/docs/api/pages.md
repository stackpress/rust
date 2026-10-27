# Page state and search

`search::SearchQuery` owns `q`, an equality-filter map, one optional `(field, direction)` order, `skip` and `limit` (default 20). `filter(field, Null)` removes that filter, preserving concrete false/zero values. `cycle_order` chooses descending, ascending, then absent; filter/order changes reset skip. `operation(&UiModel)` checks public names, scalar types and budgets and returns the actual generated query envelope. Generated page adapters then decode and validate the model's concrete query type.

`query()` returns one bounded percent-encoded `state` JSON parameter. `from_query()` decodes exactly once and rejects duplicate/unknown parameters, malformed escapes/UTF-8/JSON and oversized state. These controllers do not mutate a global window. Native hosts decode parent/key parameters separately, and validate every submitted search against generated metadata and policy.

`pages::AdminConfig::new(mount, models)` validates the fixed mount and unique compiled model names. `LayoutAdmin` provides model navigation, explicit action slots and page children. The host supplies authorized actions; visibility grants no capability. `PageSpec` contains projected model metadata and soft-remove availability. `Snapshot` is the public loader result plus selected action/key/search/parent state, action projection, downward-only transfer limits and app-provided anti-forgery token. Applications own token/session policy.

`Form` carries a draft, action, instance and optional locked related field. Update removes identity; Copy begins without the source identity. `FormIssues` connects public field issues to controls; `LockedFields` can add disabled constraints to field/filter roles. These UI constraints do not replace native policy or generated DTO validation.

`Command` is a typed UI transport request with a fixed endpoint, actual operation input, anti-forgery token, write flag and completion callback. `Reply` contains the native status and public envelope. `WriteState` distinguishes Idle/Pending/Saved/Failed/Unknown. HTTP and envelope acknowledgement are both required for Saved. UnknownOutcome takes precedence, preserves the draft and requires explicit reconciliation; callers must not retry automatically.

The [generated admin example](../../../../../examples/generated-admin/README.md) demonstrates actual generated stores/handlers over SQLite, compiled route registration, public SSR/hydrate/CSR state, related search/create and finite failure proofs. Its labeled fixture principal/token are not a production authentication/session implementation.

`Action::Import` selects preview-first create-only CSV import; `Action::Export` selects all authorized filtered matches while ignoring pagination. See [bounded CSV transfer](transfer.md) for the wire format, explicit guard decoding and host obligations.
