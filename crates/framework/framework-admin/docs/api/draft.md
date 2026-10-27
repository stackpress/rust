# Draft and RoleContext

`Draft::new(fields, seed, role) -> SchemaResult<Draft>` admits only approved, nonsensitive fields from an object seed. It bounds JSON and rejects duplicate admitted field keys. Field/span creation applies approved literal defaults only to missing values; explicit null remains null. Filter/list/view construction does not apply those defaults.

`Draft::get(field)` returns an optional root-owned `RwSignal<Input<serde_json::Value>>`. Unknown and excluded fields return None. `snapshot()` returns a bounded object of admitted values: missing fields disappear, explicit null stays null, and false/zero/empty stay values. Snapshot reads are untracked; reactive presentation should read its relevant signals before taking a snapshot. Signals must remain within their live Leptos Owner.

`RoleContext` supplies a unique DOM id, approved `UiField`, placement `Role`, and its draft signal to a compiled component. `text_prop`, `bool_prop`, and `number_prop` read only that placement's normalized props. `input_required` applies creation requiredness to field/span while leaving filters optional. `text` returns escaped-renderable plain data, preserving false and zero and distinguishing missing from null.

The [component catalog](catalog.md) lists native adapters, exact value shapes and bounds. The emitter checks value compatibility and numeric configuration before publication. Direct callers with incompatible seeds receive a visible alert. Composite controls retain domain validation in the generated input contract; creating a draft or editing a control does not validate or submit it.
