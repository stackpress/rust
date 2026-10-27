# Draft and RoleContext

`Draft::new(fields, seed, role) -> SchemaResult<Draft>` admits only approved, nonsensitive fields from an object seed. It bounds JSON and rejects duplicate admitted field keys. Field/span creation applies approved literal defaults only to missing values; explicit null remains null. Filter/list/view construction does not apply those defaults.

`Draft::get(field)` returns an optional root-owned `RwSignal<Input<serde_json::Value>>`. Unknown and excluded fields return None. `snapshot()` returns a bounded object of admitted values: missing fields disappear, explicit null stays null, and false/zero/empty stay values. Snapshot reads are untracked; reactive presentation should read its relevant signals before taking a snapshot. Signals must remain within their live Leptos Owner.

`RoleContext` supplies a unique DOM id, approved `UiField`, placement `Role`, and its draft signal to a compiled component. `text_prop` and `bool_prop` read only that placement's normalized props. `input_required` applies creation requiredness to field/span while leaving filters optional. `text` returns escaped-renderable plain data, preserving false and zero and distinguishing missing from null.

Native components are `TextField`, `TextareaField`, `CheckboxField`, `SwitchField`, and `SelectField`; display components are `TextView`, `BooleanView`, and `CaseView`. The emitter checks native scalar compatibility. Select admits at most 100 string enum choices of at most 1024 bytes each and refuses the UI library's reserved placeholder value.
