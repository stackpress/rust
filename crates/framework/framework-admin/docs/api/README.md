# Framework admin API

Use [Draft and RoleContext](draft.md) for public root state, the [component catalog](catalog.md) for exact adapter shapes/limits, [page/search state](pages.md) for CRUD transport and URL contracts, [bounded CSV transfer](transfer.md) for limits, public projection and explicit guard decoding, and [code generation](codegen.md) for the build-host catalog and emitter. Concrete adapters live in `components`; their complete prop signatures are the values returned by `codegen::components()`.

See [generated native page-host registration](pages-host.md) for Mount, dependencies, migration and direct composition.
