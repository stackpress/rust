# Change generated admin source

Change an admin wrapper's Rust structure in its checked-in template, then verify
the generated consumer. The build tool still selects and validates the fields,
roles and component imports; the template spells the resulting Rust.

## Start with a generation check

From this monorepo, use your assigned target and the contributor benchmark setup.
The existing example provides a complete schema, catalog and generator registry:

```sh
python3 .agents/scripts/build-benchmark.py run \
  --label 'admin generation check' --kind check --scope selected --cache warm \
  -- cargo run -p generated-admin-tool --locked -- --check
```

A successful check confirms that the selected generation matches the managed root.
It does not install changed output. Choose a disposable `--output PATH` to inspect
an intentional change before regenerating application-owned files.

## Edit the structure at its owner

The assets live under `src/codegen/templates/`. `roles.rs.tpl` owns descriptors and
role functions; `control.rs.tpl` owns each field context and component invocation;
`table.rs.tpl` owns public tables. `pages.rs.tpl` and `dispatch.rs.tpl` own the shared
page adapters. `pages_host.rs.tpl`, `routes.rs.tpl` and
`pages_host_registration.rs.tpl` own explicit native wiring.

For example, the control template retains runtime ID formatting while substituting
the compiler-selected role and escaped wire name:

```text
id: format!("{}-[%role_name%]-{}", instance, [%wire_literal%]),
```

The braces belong to generated Rust. `[%wire_literal%]` already contains a complete
Rust string literal, encoded with Rust's Debug-string spelling. The generic engine
inserts it once; delimiter-looking text inside the value remains data.

## Prepare data before rendering

Use explicit Boolean flags for conditions and arrays for ordered records. Keep
normalization, role admission, public type naming and import validation in Rust.
Add source declarations or repeated statements to the appropriate template, rather
than passing a whole formatted function as a value.

Nested loops use their declared aliases, such as `model.module` and
`parameter.ty`. Only the selected conditional branch reads its values. This allows
an empty selection to omit unavailable registrations and their data entirely.
Each emitter owns a local renderer that compiles a used asset once and reuses the
immutable template with fresh render data and budgets.

## Verify the observable result

Check role-only output, selected pages, omitted roles, removed models and plugin
replacement/disable. When changing controls, include labels and props containing
quotes, backslashes, newlines, Unicode and `[%...%]`. Compare final rustfmt bytes,
module order and the managed file set; compile the actual consumer afterward.

Use the existing real-schema near-limit tests when changing data preparation.
Descriptor JSON becomes a Rust literal and is encoded again inside a template
context. The renderer allows 8 MiB of compact encoded context; rendered output,
aggregate planned source and formatted artifacts retain their separate 1 MiB caps.
Increasing context headroom does not increase the output allowance.

The [code generation reference](../api/codegen.md) describes emitter inputs,
errors and integration order. The [native host reference](../api/pages-host.md)
covers app-owned mount, service and SQL registration requirements.
