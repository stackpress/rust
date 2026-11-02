# Native API index

See [runtime and emitted API](runtime.md) for the callable surface and [concepts](../concepts.md) for transaction/outcome semantics. The schema crate owns pure descriptor/DTO/query revision 3. Framework-sql's runtime compatibility constant remains 1; M2 adds native APIs inside `stores` without changing the selected pure shapes.

Use [revisions and migrations](migrations.md) for immutable history, exact plan approvals, SQLite reconstruction and the app-owned operator commands.

See [explicit string storage length](storage-length.md) for G38 declarations, target facts, validation and format-3 migration compatibility.

See the [native SQL plugin](plugin.md) for startup services, contribution bounds and errors.

See [build-host code generation](codegen.md) for transform registration, source
emission and storage definitions. Its `codegen` feature is independent of `server`.
