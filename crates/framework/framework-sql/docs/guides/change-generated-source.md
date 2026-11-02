# Changing generated Rust source

SQL source generation lives in `src/codegen.rs` and `src/codegen/`. It prepares
checked metadata for 19 checked-in `src/codegen/templates/*.rs.tpl` assets. Change
source declarations and function bodies in those assets, then update the smallest
owning context builder. The public [codegen API](../api/codegen.md) stays responsible
for build-tool selection and output-plan authority.

## Choose the owner

| Source group | Context builder | Assets |
| --- | --- | --- |
| Store identity, DDL, aggregate definitions and scalar/key expressions | `codegen.rs` | `store-header`, `table`, `schema`, `scalar-type`, `key-argument` |
| Storage-length guards | `bounds.rs` | `bounds` |
| Query/relation metadata and finite public/trusted views | `catalog.rs` | `catalog`, `views` |
| Native selectors, filters, sorting and reads | `queries.rs` | `queries` |
| Timestamp preparation, writes and active transitions | `writes.rs` | `mutable-columns`, `timestamp`, `writes`, `active` |
| Default selection projection | `selection.rs` | `selection` |
| Complete-key and unique conflict targets | `conflicts.rs` | `conflicts` |
| Selected host handlers and explicit registration | `operations.rs` | `operations-header`, `operations-open`, `operations-handler`, `operations-registration` |

Asset names above omit `.rs.tpl`. `templates.rs` owns this lane's renderer adapter,
Rust literal encoding and small shared metadata records. Context/asset key names
are coupled locally; strict lookup makes an omitted selected key fail generation.
There is no public template configuration or process-global renderer cache.

## Keep data inert and source readable

Assets use `[%` and `%]` delimiters so Rust braces remain ordinary source. For
example, this pattern emits real declarations from ordered metadata:

```text
[%#@:fields field%]pub [%field.member%]: [%field.ty%],
[%/@:fields%]
```

Use explicit boolean conditions and ordered loop records. Resolve schema policy,
relation mappings, operation selection and errors in Rust. The renderer evaluates
only the selected branch; it does not compare names, execute Rust, reparse inserted
text or infer source escaping. Do not place a preassembled function body in one
interpolation. A rendered tiny type/key expression can be inserted as trusted
source into its owning asset; its syntax still belongs to a checked-in asset.

Every wire string, serialized policy and arbitrary text must pass through
`literal`, which preserves Rust Debug string-literal escaping. Checked names and
fixed codec/type atoms come from schema normalization or explicit emitter matches.
Never insert raw schema text as an identifier or string literal. Native private
defaults retain their named-provider policy; templates do not decide exposure.

Seven original `format!` sites remain lexical: output filenames and reserved module
escaping, two constraint-name constructions, unique wire tags, default-provider IDs
and selection-path prefixes. They create names, not Rust structure. The new shared
literal encoder also uses `format!`; the sole source `push_str` appends a completed
successful render. Runtime SQL query builders and formatters are outside this
source-generation boundary.

## Respect limits and error layers

The SQL adapter uses the engine's default 1 MiB template and per-render output
limits. It allows 4 MiB of encoded JSON context, independently of output limits,
because JSON re-escapes Rust literals and duplicates metadata keys. Default finite
depth, step and iteration budgets still apply. `OutputPlan` retains its 1 MiB
aggregate raw planned-source cap across all files. The managed writer separately
enforces a 1 MiB aggregate formatted-artifact cap. Rendering several small assets
or files does not bypass either aggregate budget.

`template_error` keeps the asset label and concrete `TemplateError`. Template
`Limit` maps to emitter `Limit`; other kinds map to `Callback`. Registry planning
adds its existing outer `Callback`. Do not replace concrete causes with context
payload dumps, change common schema error policy or raise global writer caps.

A failing asset appends no partial source. A later emitter failure may leave
earlier claims in the in-memory plan; discard that failed plan. Only the managed
workspace can install output, after its validation and formatting checks.

## Verify compatibility

Run the focused `codegen` tests without `server`, then storage-generation tests
with `codegen,migrations`. The permanent fixtures cover real Idea composition,
native/handler selection, hostile literal data, private policy and relation views,
repeat/check installation, and actual near-limit and oversized schemas. Generic
grammar failures belong to the engine tests.

Compare generated files and the managed manifest after the unchanged rustfmt path,
using disposable roots. Include the default app, alternate native/type matrix and
reserved-name fixtures with handlers both absent and present. Do not edit checked-in
generated roots to make a source test pass. Exercise generated native consumer
operations as well as generation itself; SQL rendering snapshots alone do not
prove live PostgreSQL/MySQL behavior.

Whitespace can affect accepted raw size and final bytes. Keep the conditional
blank line after timestamp assignments in `writes.rs.tpl`. The long policy
declarations and JSON macro members in `active.rs.tpl` and `timestamp.rs.tpl`
preserve legacy compact spacing that rustfmt retains. `bounds`, `views`,
`scalar-type` and `key-argument` intentionally have no newline after their outer
closing tag. `active` ends with a newline and is rendered only when an active
field exists. Check final bytes and real near-boundary inputs after layout changes;
token equivalence alone is insufficient.

The retained boundary fixture was accepted by the original emitter at 1,004,844 raw
bytes. The structural templates produce 1,034,170 raw bytes and the same 1,035,095
formatted bytes, all below 1 MiB. Its oversized companion preserves the direct
emitter `Limit` beneath the registry's outer `Callback` and leaves installed output
unchanged. These are compatibility cases, not permission to increase output caps.
