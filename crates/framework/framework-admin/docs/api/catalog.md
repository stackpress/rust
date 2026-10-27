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
| `field.rating` | `RatingField`, Integer | `max: Number`, integral 1..=100, default 5; zero is a rating; Clear selects omission |
| `field.slug` | `SlugField`, String | Verbatim typing; Apply slug explicitly uses ASCII words, lowercase and dash separators; 4096-byte transform budget |
| `view.text` | `TextView`, public data except virtual relations | `null_label: String` |
| `view.yesno` | `BooleanView`, Boolean | `null_label`, `yes_label`, `no_label`: String |
| `view.uppercase`, `view.lowercase`, `view.capitalize` | `CaseView`, public data except virtual relations | `null_label: String`; fixed `case: String` |

Select options are bounded to 100 unique values, each at most 1024 bytes; the reserved native placeholder value is refused. Search text is an intermediate draft: editing it clears the selected machine value and therefore omits the field until another declared option is selected. Unknown choices cannot be submitted as selected values.

Collection row identity belongs to the UI controller and is never serialized into domain data. Add/edit/remove/reorder retain stable keys within the current collection; explicit replacement through the draft rebuilds the collection. Metadata JSON objects have no semantic ordering, while their editable rows retain order for the current mount.

Missing and Null have an empty presentation without being rewritten. Choosing Value initializes an empty array/object, empty string, false, numeric lower bound, or zero rating as appropriate. An edit publishes exact concrete values. Invalid initial shapes fail visibly; invalid intermediate metadata preserves the accepted snapshot and reports the error. The app must validate generated public inputs before any operation.

List selections also generate the model's [table function](codegen.md). It uses the exact selected display adapter for each column rather than converting every value to plain text.

Formatted number/date/phone inputs, rich text, file/media and relation/service suggestions remain unavailable. `field.suggest` does not become a local enum search: select `field.select({ searchable true })` for that capability.
