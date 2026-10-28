# Concrete component catalog

Canonical definitions remain in the app-owned registry. Schema owns aliases and derived filter/span placements; availability requires an implemented adapter with compatible public data. Every input accepts optional typed `label: String` and `disabled: Boolean` props. Presence controls obey disabled state too.

| Canonical selection | Adapter and public value | Additional props / bounds |
| --- | --- | --- |
| `field.input` | `TextField`, scalar String | `placeholder: String`, `read_only: Boolean` |
| `field.textarea` | `TextareaField`, scalar String | `read_only: Boolean` |
| `field.checkbox`, `field.switch` | `CheckboxField`, `SwitchField`, Boolean | Exact false remains supplied |
| `field.select` | `SelectField`, finite string enum | `placeholder: String` in native mode, `searchable: Boolean`; searchable chooses `ComboboxField` over the same local enum; supplying placeholder with searchable is diagnosed because the current combobox API has no placeholder; no endpoint |
| `field.stringlist`, `field.textlist` | `TextListField`, String array | At most 100 rows, 4096 bytes per string; empty and duplicate strings survive |
| `field.tags` | `TextListField` using UI TagList | Fixed `tags: true`; other text lists fix `tags: false` |
| `field.metadata` | `MetadataField`, string-valued Object | At most 100 entries, 1024-byte keys, 4096-byte values; duplicate editable keys report an error and preserve the accepted draft |
| `field.slider` | `SliderField`, scalar Integer or Number | `min`, `max`, `step`: Number; defaults 0/100/1; finite ordered bounds, positive complete grid with at most 10,000 steps; Integer scale values must be integral; all scale values within browser-safe integer magnitude |
| `field.rating` | `RatingField`, Integer | `max: Number`, integral 1..=100, default 5; the minimum is derived from public schema `@min` rules (negative bounds admit zero; fractional bounds round up to the next whole rating). Stars cover admitted positive values; the small zero control appears only when the minimum admits 0. Clear selects omission |
| `field.slug` | `SlugField`, String | Verbatim typing; Apply slug explicitly uses ASCII words, lowercase and dash separators; 4096-byte transform budget |
| `view.text` | `TextView`, public data except virtual relations | `null_label: String` |
| `view.yesno` | `BooleanView`, Boolean | `null_label`, `yes_label`, `no_label`: String |
| `view.uppercase`, `view.lowercase`, `view.capitalize` | `CaseView`, public data except virtual relations | `null_label: String`; fixed `case: String` |

Select options are bounded to 100 unique values, each at most 1024 bytes; the reserved native placeholder value is refused. Search text is an intermediate draft: editing it clears the selected machine value and therefore omits the field until another declared option is selected. Unknown choices cannot be submitted as selected values.

Collection row identity belongs to the UI controller and is never serialized into domain data. Add/edit/remove/reorder retain stable keys within the current collection; explicit replacement through the draft rebuilds the collection. Metadata JSON objects have no semantic ordering, while their editable rows retain order for the current mount.

Missing and Null have an empty presentation without being rewritten. Choosing Value initializes an empty array/object, empty string, false, numeric lower bound, or zero rating as appropriate. An edit publishes exact concrete values. Invalid initial shapes fail visibly; invalid intermediate metadata preserves the accepted snapshot and reports the error. The app must validate generated public inputs before any operation.

List selections also generate the model's [table function](codegen.md). It uses the exact selected display adapter for each column rather than converting every value to plain text.

Rich text, file/media and relation/service suggestions remain unavailable. `field.suggest` does not become a local enum search: select `field.select({ searchable true })` for that capability.

## Optional formatted roles

Enable the admin `format` feature before registering definitions or consuming generated formatted wrappers. It enables framework-ui's existing formatting policies. Without it, these selections remain diagnosed as unavailable. The default dependency graph retains the existing URL/IDNA ICU normalizer baseline and gains none of the formatting ICU packages or phonenumber.

| Canonical selection | Public value | Additional props / bounds |
| --- | --- | --- |
| `field.number`, `field.price` | Number, Integer or exact decimal String | `scale: Number` integral 0..=18, default 2 (Integer: 0 only); `digits: Number` integral 1..=96, default 24 |
| `field.integer` | Integer | Same decimal policy, scale 0 |
| `field.date`, `field.time`, `field.datetime` | Date, Time, Datetime respectively | `precision: Number` integral 0..=9, default 0 for Date and 9 otherwise; Date permits only 0 |
| `field.phone` | String | Required `region: String` (for example `US`); valid national or international entry commits E.164 |
| `view.number`, `view.price` | Numeric family or exact decimal String | Same scale/digits configuration; grouped localized display |
| `view.date` | Date, Time or Datetime | Localized date/time display with explicit offset for instants |
| `view.phone` | Canonical E.164 String | International display |

These definitions accept `locale: String`, either `en` (default) or `fr`. Field definitions also accept the ordinary label/disabled props; views accept `null_label`. Frozen registry aliases derive filter/span placements from fields and list placements from views. Configuration and value-type mismatches are diagnosed at the Idea attribute position, or the structural path for normalized JSON.

Revision 2 has no decimal wire type. Use a String for exact decimal storage; Number remains the schema's finite floating-point family and Integer remains a checked browser-safe integer. Rounding follows the existing UI decimal policy. The adapter normalizes successful temporal commits through the actual schema Date/Time/Datetime constructors; instants require an explicit offset and are stored in canonical UTC. Phone regions are explicit metadata, never inferred from the machine.

Typing changes only the editing presentation. Enter or blur attempts a checked commit; partial/invalid input retains the last accepted value and reports a field error. Escape cancels the intermediate edit. Temporal presentation uses declared precision; without a declaration it trims trailing fractional zeros while preserving every nonzero fractional digit and the explicit offset. Canonical commits retain their full admitted precision. The example sizes formatted inputs for full wide-screen values and shows the associated hint only in the focused field, retaining each control's help/error descriptions.

The page's `PendingEdits` context blocks submission until all formatted values are committed or canceled. Programmatic Draft resets are checked and do not emit user commits. Null, Missing, false and zero retain their existing meanings. When an operation reports a generic `invalid` issue, the submitted public snapshot is checked against client-checkable public rules to recover specific wording. Unavailable host rules retain the safe fallback.

Create/copy hides the presence selector for required non-null fields. Nullable fields can select “Clear (empty)” or “Set value”; filters and patches can omit values, with patch omission labeled “Leave unchanged”. Public rule messages are preferred for validation errors, with specific public-descriptor fallbacks. Result links use a public name/title/label or readable identity values.
