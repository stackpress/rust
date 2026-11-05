# Source model UI

`stackpress_framework_schema::codegen::SourceGraph::project_source_ui` produces a portable
`SourceUiCatalog` from the fully composed application graph. It does not normalize
through Typed schema policy. Source roles describe controls and display fields;
they grant no authority to a route or event.

Build tools select `stackpress_framework_admin::codegen::source_definitions`, project that
catalog, then call `emit_source_roles` or `emit_source_pages`. Both use structural
templates and the existing managed output writer. `emit_source_pages` includes
roles, model-independent page dispatch and a host model catalog in source order.
The application supplies native registration and IO; generation opens no database.
Typed `emit_roles` and `emit_pages` retain their existing inputs and output.

| Feature | Purpose |
|---|---|
| `codegen` | Build-host definitions and emitters. |
| `source` | Portable source roles and page callbacks, independently of codegen. |
| `source-ssr` | Source roles plus SSR and native local-time date formatting. |
| `source,hydrate` | Browser hydration; no native chrono dependency. |
| `source,csr` | Separate browser CSR artifact; no native chrono dependency. |
| `format` | Existing opt-in shared format roles used by the complete shipped model UI. |

Generated roles consume caller-owned `Draft` values or JSON rows. Generated code
uses the schema runtime's serde_json re-export. The frontend still selects Leptos
and its renderer explicitly. Portable descriptors contain source role props and
shape, not SQL metadata, encryption/hash attributes, provider arguments or seeds.
Data supplied by a page remains that source process's responsibility; this is not
a blanket redaction or authorization policy.

Source date roles retain the original local-time Moment spelling, including the
literal `i` in the shipped format. The existing Leptos UTC-offset formatter is
not substituted. Joined template roles receive the actual row, while direct
legacy callers can retain their relation-choice context.

Original blog `field.editor` toolbar props and `field.select` options normalize
as source data. **Rich editor adapter absent:** no reusable rich editor exists
in the shared controls. The editor catalog remains unavailable for concrete
emission; an application must select a compiled implementation. No editor is
silently replaced by a textarea. The stock select adapter handles string-array
options and placeholder; source styling or non-string-array options require an
application adapter and are diagnosed at emission. The stock date adapter implements
the shipped `m d, Y h:iA` format; other Moment formats need an application adapter.
Metadata uses the source two-column table and cell class; CSS-object styles require
an application adapter. Source collection `add` props reach the shared controls;
ordinary callers that omit them retain “Add row.”

The source metadata field applies the original form generator's `String(val)` conversion
before using the shared text metadata editor (`stackpress-view/src/transform/form/field.ts:139–141`).
Viewing the editor leaves the exact draft unchanged; an edit publishes its string values.
The Typed metadata editor keeps its existing admission rules. The source detail view still
refuses a nested object child, matching the original view's behavior.

Evidence: [source UI tests](../tests/source.rs) cover full shipped composition,
extension roles, private metadata exclusion, managed regeneration and native local
time in fresh processes. [Schema prop tests](../../framework-schema/tests/source_ui.rs)
pin the exact original blog declarations. The M3 receipt records separate finite
generated SSR/hydrate/CSR compilation and normal-edge graph checks.
