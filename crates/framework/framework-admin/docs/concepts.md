# Public role generation

An application constructs one immutable `DefinitionRegistry`, including field access policy and concrete component definitions. Ordinary schema, SQL, and UI functions normalize the same finalized input with that registry. One Idea output writer owns their separate partitions and removes obsolete files.

Schema owns aliases, argument typing, fixed-prop precedence, public projection, and descriptor revision 2. UI generation consumes `UiSchema` and resolves component keys against build-host `ComponentImport` mappings. Imports and diagnostic source coordinates stay outside browser descriptors. Rust strings use escaped literals; labels and props remain data.

Field, filter, list, span, and view selections are independent. Missing roles create no function. Filters use field definitions, lists use view definitions, and span retains input semantics. Sensitive and host-only fields contribute no wrapper metadata. Display functions accept generated `Public<Model>` values. Field/filter/span functions accept a root-owned `Draft` and an instance identity; callers must keep instance identities unique within the page.

`Input::Missing`, `Input::Null`, and `Input::Value` remain distinct. Missing creation values can use approved literal defaults; filters do not inherit creation defaults. Native text edits preserve empty strings, and Boolean controls preserve false. Presence selectors explicitly choose omission or null. Snapshotting a draft never submits a write or replaces host validation.

The current concrete catalog covers Input, Textarea, Checkbox, Switch, finite string-enum Select, escaped text, YesNo, and fixed uppercase/lowercase/capitalize views. The schema's larger vocabulary stays unavailable until a concrete adapter is registered. Rich editors, service-backed controls, and the remaining M2/M3 adapter families are not implemented by this milestone's catalog. Explicit selection diagnoses instead of silently substituting an input.

Browser consumers enable `hydrate` or `csr`; native SSR consumers enable `ssr`. Only the build tool enables `codegen`. The browser graph must not select stores, framework SQL, the Stackpress server, Idea, or Tokio. Platform-installed UI and CRUD remain later milestones.
