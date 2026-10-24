# Generate a Profile application

An application must be able to generate its first Rust source before that source exists.
This example uses an independent codegen package to turn a small Idea model into public
contracts and a working search operation. Rust 1.96 and its `rustfmt` executable are required.

## Generate and search

From the workspace root, prepare the application with one command:

```sh
cargo run -p stackpress-example-profile-codegen --locked -- emit generate
```

The command reads [app/schema.idea](app/schema.idea) and writes inspectable source in
[app/src/generated](app/src/generated). A successful JSON envelope reports `code: 200`.
Run the application to see the generated operation return Ada Lovelace:

```sh
cargo run -p stackpress-example-generated-profile --locked -- emit profile-search --id abc123
```

The result contains `[{"id":"abc123","name":"Ada Lovelace"}]` and `total: 1`. The same
compiled operation handles direct calls and buffered HTTP requests; no listener starts.
The [parity tests](app/tests/parity.rs) compare the complete envelopes with the Slice 2 app.

## Follow the generated code

The model declares the public fields:

```idea
plugin "profile-demo" { }

model Profile {
  id String
  name String
}
```

The app-owned `profile-demo` function generates `contracts/profile.rs` and
`server/profile.rs`. The contracts partition depends only on Serde. The server partition owns
the fixed demo rows, case-sensitive ID-prefix search, validated configuration and GET route
`/profiles/:id`. This generator accepts exactly the displayed fields and order, without
model/column metadata or plugin settings. General field mapping belongs to a later schema layer.

The app's aggregate function emits the root registry; the transformer owns its manifest.
The application calls `generated::register`; it never imports `Profile` by hand. Removing the model removes
its files, exports and registration while leaving a compiling application that returns 404.

The codegen tool composes sources through a `ProjectLoader` rooted at the app, then the
`stackpress_idea::generate` wrapper resolves typed references and plans the selected transforms.
[codegen/src/profile.rs](codegen/src/profile.rs) owns the Profile mapping and its declared
runtime-template snapshot. The CLI host owns path options, diagnostics and the awaited worker.

## Split a schema into files

Move the displayed Profile model into a sibling `types.idea`, then keep the root input small:

```idea
plugin "profile-demo" { }
use "./types.idea"
```

Run the same generation command. Imports resolve relative to the importing file; `@/` resolves
from the app's project root. The output is byte-identical to the inline model. The native
[generation tests](codegen/tests/generation.rs) exercise this change in a disposable directory,
including child-file diagnostics and unchanged output after a malformed import. Keeping the
checked-in model inline makes the first author example easier to read.

## Check freshness before building

Run the same planner without installing output:

```sh
cargo run -p stackpress-example-profile-codegen --locked -- emit generate -c
```

Current output returns exit 0; stale output returns exit 1 with a public 400 envelope. Check
does not update generated files or their manifest. It briefly acquires the cooperative writer
lock. CI should run this command before building the app: an ordinary Cargo build checks
compilation and selection compatibility, and requires this separate step for schema freshness.

The long spelling is `emit generate --check true`. All capability data follows `emit generate`;
the existing CLI grammar requires the value for a long boolean flag. Operator `-v` goes before
`emit` when developer diagnostics are needed. `-f` never grants permission to discard files.

## Use another input or output

Explicit paths are capability data. Relative explicit paths resolve from the command's working
directory; omitted paths always resolve from this example's app directory:

```sh
cargo run -p stackpress-example-profile-codegen --locked -- emit generate \
  --input examples/generated-profile/app/schema.idea \
  --output /private/tmp/my-profile-generated
```

The output must be absent, empty, or already owned by this generator. Source and output paths
are passed as separate argv values, so spaces in paths work when quoted by the calling shell.
Malformed Idea produces a public `file:line:column` error with exit 1; source excerpts and
environment values are omitted. Native filesystem and internal errors stay sanitized.

## Build without the profile plugin

Generator selection is compiled into the tool and application. To try the disabled selection,
generate its empty registry, then build the matching app:

```sh
cargo run -p stackpress-example-profile-codegen --no-default-features --locked -- \
  emit generate --input examples/generated-profile/app/schema-disabled.idea
cargo run -p stackpress-example-generated-profile --no-default-features --locked -- \
  emit profile-search --id abc123
```

The search now returns 404 and exit 1. The disabled tool contains no profile callback;
a `plugin "profile-demo"` block cannot select uncompiled code. Generation removes its prior
owned profile files. Both packages compile the same [selection.rs](selection.rs), and generated
const assertions reject a selection or contract-revision mismatch at app compilation.
Restore the selected example by running the first generation command again.

## Preserve and recover output

Treat `app/src/generated` as disposable generated source and edit the schema or generator instead.
The writer rejects unknown files, edited owned files, symlinks and overlapping managed writers.
It installs a staged, formatted tree and retains recovery state after an interrupted install.
It requires cooperative exclusive ownership and makes no crash-atomic or power-loss guarantee.

After an interruption, ensure no writer is still active and inspect the reported sibling lock
and recovery state. A stale writer lock is removed only by an operator after that inspection;
it is never reclaimed from a PID guess. The transformer exposes `OutputWorkspace::recover`
for recognized journal/backup recovery. Do not discard a backup or a whole output root until
you have reviewed which files it owns. The tool has no force-repair switch.

## Verify the complete example

The native integration suite creates disposable Cargo consumers and checks first generation
with no output, repeated bytes, Serde-only contracts, selection/revision compile failures,
removed-model cleanup and enabled-to-disabled cleanup:

```sh
cargo test -p stackpress-example-profile-codegen --all-targets --locked
cargo test -p stackpress-example-generated-profile --all-targets --locked
```

These tests use finite subprocess deadlines and real generated Rust. They verify the installed
native target; cross-platform filesystems, WASM, Leptos and Tauri need separate target proofs.
