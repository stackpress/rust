# Bounded CSV transfer

`transfer` supplies pure CSV parsing, public projection and import progress state. It performs no authorization, file IO, database work or network requests. The [generated admin example](../../../../../examples/generated-admin/README.md#bounded-csv-importexport-m6-g36) supplies those boundaries through actual generated contracts and atomic operations.

```rust
use stackpress_framework_admin::transfer::{Limits, parse};

let limits = Limits { rows: 25, ..Limits::default() };
let records = parse(b"id,name\na,Ada", limits)?;
assert_eq!(records[1], ["a", "Ada"]);
```

This example's body returns `Result<(), stackpress_framework_schema::SchemaError>`; the same call is compile-tested as a module doctest.

## Limits

`Limits` has `bytes`, `rows`, `columns` and `cell` ceilings. Defaults are 1,048,576 bytes per file/output, 100 data rows, plus import-only 100 columns and 65,536 UTF-8 bytes per cell. `validate()` rejects zero or any increase above these maxima. Applications may only lower the defaults.

`parse(bytes, limits)` checks file size and UTF-8 before allocating cells; each cell/column/row append is charged. It returns the header plus every data row in input order, or a `SchemaError` for malformed quotes, lone record CRs, invalid UTF-8, exceeded limits, or no data rows. It accepts LF/CRLF records and quoted embedded line breaks. Blank records are data, not silently skipped.

`write(records, limits)` deterministically quotes every cell, doubles embedded quotes and uses CRLF. It charges output bytes before appending and refuses excess rather than returning partial output. Its row budget excludes the header. A header-only export is allowed; an import still requires at least one data row.

## Preview and export

`fields(model, import)` selects approved stored public fields. Import requires writable fields; export requires readable fields. Sensitive and virtual relation fields are always excluded.

`Upload` contains `csv`, `decode_export_guards` (default false) and an explicit `confirm` flag (default false). `preview(model, &upload, limits)` returns `Preview { headers, rows }`. Structural/header errors return `Err`. Cell conversion errors remain on each `Row`, preserving `number` (header is record 1), `raw`, decoded `values` and safe `errors`. Duplicate, unknown and nonwritable public wire headers are refused. `Preview::valid()` requires at least one row with no errors across the complete batch.

Preview conversion preserves omission/null/value independently. `\N` is null. Import-only `\O` omits a field, and is refused for required fields with no default. A doubled leading backslash represents one literal backslash, so `\\N`, `\\O` and `\\path` remain strings. Scalar String/enum/date/time/datetime cells are verbatim canonical strings; other types, including arrays, are JSON. Empty strings, false and zero retain their meaning. Actual generated create contracts must then validate every row; the pure CSV preview does not replace their checks, host policy, parent scope or database conflict checks.

`export(model, rows, limits)` emits public stored fields in descriptor order. Every projected member must be present; absence is an error, never an omission token. Null uses `\N`. Strings remain verbatim except the documented escapes; other values use JSON. Import-only column/cell ceilings do not restrict export. Row/byte excess refuses the whole export with a narrowing-filter message. The caller must supply all authorized filtered matches in selected order and independently refuse an incomplete/truncated result.

## Explicit guard decoding

Export prefixes an apostrophe to string cells beginning with `=`, `+`, `-`, `@`, tab or carriage return. An original leading apostrophe is doubled. Negative JSON numbers remain numbers. Leading backslashes use the separate escape rule above.

Ordinary import preserves apostrophes literally. With `decode_export_guards: true`, a leading `''` becomes the original single apostrophe, and a leading apostrophe before `=+-@`, tab or carriage return is removed. Other apostrophes remain unchanged. Both raw and decoded cells belong in the preview. Enable this explicitly for Stackpress exports to obtain exact round trips; do not guess the file's origin from its contents.

## Host and progress contract

The host must authorize before parsing, retain no uploaded server file, show preview before confirmation, and revalidate on confirmation. Create-only confirmation must reject existing/repeated identities and use the actual generated Batch operation in one transaction and input order. Related imports/exports must retain their verified parent scope. The example limits preview serialization separately to 1 MiB and refuses excess without dropping rows.

`Phase` distinguishes Idle, Previewing, Previewed, Applying, Imported(count), Failed(message) and Unknown. `locked()` covers pending work and Unknown. `reply(write, status, envelope)` accepts only consistent successful HTTP/public-envelope statuses and an acknowledged count from 1 through 100 as success. A missing/malformed successful write acknowledgement, or explicit unknown outcome, yields Unknown even if the HTTP status says success. `message()` never claims imported success for uncertainty. Keep drafts on known failures, require explicit confirmation after a fresh preview, disable resubmission after Unknown, and never retry automatically. The browser adapter must treat lost write transport acknowledgement as unknown and discard callbacks after root disposal.
