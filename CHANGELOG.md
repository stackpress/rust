# Changelog

Changes are grouped by release in Keep a Changelog style. The 23 published crates
share one version. Before 1.0, breaking changes bump the minor version.

## 0.2.0 (beta) — 2026-11-12

This is a beta release: the API may still change before 1.0. It is a normal Cargo
version, without a pre-release suffix. See the [migration guide](docs/migrating-0.1-to-0.2.md).

### Breaking

- `Nest::entries()` now returns `stackpress_lib::Result<&DataMap<String, Value>>`.
- `EditPolicy::display()` now returns `Result<String, EditError>`. Custom policies
  and direct callers must handle presentation errors.
- `SourceStore::create()` now returns `Result<Create, Self::Error>`.
  `NativeStore::{create, paths, selectors, joins}` also return fallible results.
- Generated aggregate clients' `definition()` now returns
  `stackpress_lib::Result<Vec<stackpress_sql::TableDefinition>>`, replacing the SQL
  crate's result type. Regenerate application code and update explicit return types.
- API `views::management::import_csv` now takes `impl AsRef<[u8]>` instead of
  `&str`. Ordinary string calls still work; typed callbacks may need a wrapper.
- Admin `Upload.csv` is now `CsvBytes`, replacing `String`. Rust callers can convert
  strings with `.into()`. JSON serialization emits a byte array; deserialization
  accepts both byte arrays and legacy strings.

### Added

- `CsvBytes` conversions from `String`, `&str` and `Vec<u8>`, plus byte access,
  length, empty and clear operations.
- `RoleContext::from_draft()` for binding generated controls to existing draft state.
- Infallible defaults for `Language` and `SelectOptions`, and
  `PublicLocation::session_asset()` for the conventional session script location.
- Unpublished fuzz targets for Idea, admin CSV and query/form parsing; an attribution
  index and an offline dependency-licence snapshot check.

### Changed

- All 23 release crates enforce the six no-panic Clippy lints in library code.
  Named, tested constant-parser exceptions remain. Application callback unwind
  handling is unchanged; this policy is not a guarantee against every possible panic.
- CSV import detects UTF-8/UTF-16 BOMs, accepts valid UTF-8, and falls back to
  Windows-1252 or an Arabic-evidence Windows-1256 heuristic. It detects comma,
  semicolon and tab separators outside quoted fields. Both raw and decoded size
  limits apply. Export remains UTF-8 with commas.
- API CSV import accepts byte-oriented input through `impl AsRef<[u8]>`; the native
  admin handler preserves original upload bytes. Existing JSON row batches retain
  their own import path.
- Generator writes sync staged files before publication and attempt to sync parent
  directories after renames on Unix. Existing ownership and recovery rules remain.

### Fixed

- Malformed handwritten relation/unique metadata, inconsistent generator catalogs,
  and non-renderable admin fieldset/metadata/list/table children return errors or
  show the existing rejection state instead of panicking.
- Editor presentation failures preserve the current value and draft. Invalid private
  toolbar icon names no longer panic.
- Live database test fixtures reserve cleanup before CREATE, so an uncertain or
  failed CREATE still attempts to drop only its own database.

### Security

- No separate security fix is identified after the 0.1.0 baseline. The no-panic
  policy and bounded tolerant importer are described above.

## 0.1.0 — 2026-11-11

Initial coordinated release of 23 crates; publication completed on 2026-11-12 UTC.
It already included the standalone email HTTPS/TLS provider fix and HTTP output
conversion of native dates to ISO text, with redirect bodies retained (G57).
These are not new 0.2.0 changes. Core value serialization remains strict.

The published archives record commit `d563f8d` before the author-email rewrite.
Its file tree is identical to commit `25567fb`, the baseline used for this changelog.
