# From sources to owned output

A code generator needs to answer three separate questions: which schema wins, which compiled
functions may run, and which files it may replace. The transformer makes each answer explicit
before installing anything. The [quick start](getting-started.md) exercises the same boundaries
with one imported model and one output claim.

Contents: [1. Sources](#1-sources) · [2. Composition](#2-composition) ·
[3. Function plugins](#3-function-plugins) · [4. Output claims](#4-output-claims) ·
[5. Owned workspace](#5-owned-workspace)

## 1. Sources

Think of a loader as an immutable source snapshot provider. `SourceLoader::load` receives an
optional canonical importer, a specifier, and a remaining byte allowance; it returns a
`LoadedSource` with identity, logical display name, syntax kind, and UTF-8 text.

`ProjectLoader` canonicalizes an explicit absolute directory. `MemoryLoader` uses an absolute
virtual namespace and supports fixtures and aliases without disk IO. Both try exact file,
`.idea`, `.json`, `index.idea`, then `index.json`; explicit extensions skip the two extension
candidates. Extensionless exact files use Idea syntax. Executable extensions are rejected.

Ordinary imports resolve from the importing canonical file's parent. Entry paths and `@/`
aliases resolve from the configured root. Absolute and parent-relative imports work, so the
root is a resolution policy, not a sandbox. Neither loader changes the process directory or
reads process environment. Native size/mtime checks detect common acquisition changes; they
do not create a transaction with arbitrary outside editors.

## 2. Composition

Each `compose` call creates fresh state. It walks imports depth first, detects cycles on the
active canonical path, and reuses completed child snapshots in diamonds. Repeated acquisitions
of one identity must agree on the whole loaded snapshot. Child snapshots are never mutated by
a parent merge.

For example, these are the two input snapshots for a mutable extension:

```idea
// schema.idea
use "./base"
model Profile { name String }

// base.idea (a separate file)
model Profile { id String }
```

The result has columns `id`, then `name`: child-only columns prepend before local columns.
`ComposedSchema.config` consumes imports but keeps props and unresolved references. Only the
root file's plugins remain active. The parser owns [language and finalization](../../idea-parser/docs/README.md);
composition does not assign framework meaning to metadata.

### 2.1. G19 merge and replacement

Apply direct `use` declarations in source order to the current accumulated local schema.
Types and models use the same rules, with independent kind namespaces.

| Collision or absence | Result |
| --- | --- |
| Child lacks a declaration | Keep the local declaration. |
| Local prop or enum exists | Keep its whole local value; members are not merged. |
| Local prop or enum absent | Adopt the child value; child-first map spread determines key order. |
| Local type/model absent | Adopt the complete child, including its mutable flag. |
| Accumulated local type/model immutable (`!`) | Replace it completely with the imported child, including its flag. |
| Accumulated local type/model mutable | Keep local flag and colliding attributes; adopt missing child attributes. |
| Mutable local has a colliding column | Keep the entire local column, including all metadata. |
| Mutable local lacks a child column | Prepend child-only columns as a group in child order. |

The immutable replacement is intentional G19 behavior. After each import, recheck the
accumulated flag: an adopted immutable child can be replaced by the next child. In a mutable
accumulation, earlier conflicting values win; later child-only column groups prepend ahead of
earlier groups. Type/model declaration order is local names followed by first-seen imported-only
names. Attribute maps use child-first order with local values taking precedence.

### 2.2. Origins and real references

`sources` holds unique snapshots in first-acquisition DFS order, root first. `ImportEdge`
records importer, literal specifier, and canonical imported identity in encounter order;
diamonds retain distinct incoming edges. These are acquisition facts, separate from the
effective schema's winning origins.

`provenance` maps structural paths such as `model/Profile/columns/id` to source IDs. Declaration
roots, declaration attributes, and whole columns get separate entries. `references` carries
the parser's actual `Reference` values, including named owners, nested value paths, original
spans, targets, and source-local forward flags. Adoption moves them with their winning subtree;
replacement discards losing sites. Prepending a column does not invalidate a named site.

Call the parser's `finalize_with_references` after composition, passing both config and sites.
This resolves real references while preserving literal strings such as `${Name}`. Plain
normalized JSON supplies no inferred sites; a checked `CompiledSchema` JSON envelope can
preserve explicit ones and their original Idea filenames. Provenance still identifies the JSON
artifact through which they were acquired. A file-import cycle and a value-reference cycle
are different failures at different stages.

### 2.3. Finite work

`CompositionLimits` bounds source bytes, repeated acquisition bytes, unique files, active
depth, import edges, cumulative config/reference expansion, and JSON nodes. Defaults are in
the [composition reference](api/composition.md#2-compositionlimits). Active import depth has an
unraiseable ceiling of 64. Parser/data limits and an explicit environment snapshot arrive
through `CompileInputs`; JSON also spends the composition node budget.

Counters reset per run. Repeated diamond reads spend acquisition bytes, and copied child
config/sites spend expansion bytes even if a local collision will win. Budget failures return
errors rather than partial successful schemas. Retained source text can contain secrets;
reports should expose it only through deliberate developer diagnostics.

## 3. Function plugins

The application compiles generator functions and registers them in the intended execution
order. `TransformSpec` supplies a stable ID independent of the Rust function name, exclusive
first-component partitions, earlier prerequisites, optional-present predecessors, and exact
runtime contract revisions. Registration rejects invalid metadata and overlapping partitions
atomically; planning checks the whole selection before invoking any transform.

`TransformSelection` supplies contract revisions and immutable `TransformInputs`; it does
not choose executable plugins. `TransformContext` exposes the schema, current ID, optional
root plugin config, contracts, and declared snapshots. `input` records consumed logical names,
never values. Trusted Rust functions can still perform direct IO; these APIs are not a sandbox.

### 3.1. G20 selection table

Root plugin blocks configure compiled code; imported blocks are inert. The current planner
uses this complete table and never silently sorts callbacks.

| Root plugin blocks | Compiled registry | Behavior |
| --- | --- | --- |
| None | Nonempty | Execute every registered transform using defaults; `config()` is `None`. |
| None | Empty | Produce aggregate-only output, including an empty plan. |
| Present | Any | Require exact IDs and listed order, including every selected registration. |

Required IDs must occur earlier. Optional IDs may be absent, but must occur earlier when
selected. Later/self requirements and cycles fail. Required runtime revisions must match the
app-supplied selection exactly. No source string loads uncompiled code, and callbacks run
sequentially once after preflight. Callback panics propagate.

## 4. Output claims

An `OutputPlan` describes fresh files, each with one owner. Callback `write` calls must stay
under that callback's exact partition spelling; duplicate claims fail even for identical bytes.
The plan exposes files in path order, execution IDs in registration order, consumed input names
sorted and deduplicated, and module contributions in declaration order.

For ordinary Rust indexes, a plugin writes `contracts/profile.rs` and calls
`contribute_module("contracts", "profile")`. After all callbacks succeed, the registry's
aggregate owner writes `contracts/mod.rs` and root `mod.rs`. Every contribution must have a
matching file owned by its contributor. Plugins do not patch another plugin's files.

App-supplied `GeneratedFile` aggregates claim root-level files before callbacks. They cannot
collide with partitions or with automatic module indexes; supplying `mod.rs` while using
contributions is a conflict. With no contributions, the app can render its own root registry,
as [generated-profile](../../../../examples/generated-profile/README.md) does. The writer owns
the manifest independently of these app aggregates.

## 5. Owned workspace

`OutputWorkspace` binds a dedicated output root beneath an existing parent. A first root may
be absent or empty; a nonempty root must already have a recognized `.stackpress-generated.json`
marker. The version-1 marker records owners, SHA-256 digests, and installed byte lengths;
legacy entries may omit lengths. It grants cooperative cleanup authority, not authenticity.

### 5.1. Check and install

Both modes format `.rs` files through Rust 1.96 `rustfmt`, edition 2024, with child traversal
disabled. Non-Rust bytes are preserved. Formatting completes before acquiring a create-new
sibling lock. Checking briefly creates/removes that lock but changes no generated files or
manifest. Absent markers are stale even for empty plans.

Changed planned bytes or owners are differences; removed planned paths are stale output to
delete on install. Manual edits to installed files, missing promised files, unknown files,
manual directories, malformed markers, symlinks, and special files are errors. Checking cannot
silently repair them. `install` returns the pre-install differences; a subsequent check proves
the new output current. `install_report` instead classifies generated, unchanged, and removed
paths with selected IDs and consumed auxiliary names.

### 5.2. Cooperative locking and recovery

The lock lives outside the replaced tree, beside fixed stage, backup, and journal siblings.
Competing managed writers fail immediately. Ordinary returns and unwinding release the lock;
process death leaves it for operator inspection. A PID is informational and never authorizes
automatic reclamation.

Installation stages a complete formatted tree, publishes recognized intent, retains the old
tree, installs the stage, and cleans up with the ownership marker removed last. Interrupted
work blocks new check/install operations. `recover` validates retained trees before mutation:
keep a valid installed root; otherwise restore the complete backup; otherwise use the complete
first-install stage. Recognized cleanup intent permits already removed promised backup files,
requires a valid installed root, and never restores a partial backup. A stage without recognized
intent, an unknown journal, or a stale lock requires explicit operator inspection.

### 5.3. Limits of the writer

All writers must cooperate with one exclusive owner of the root. Outside edits can race
inspection/open/rename/delete; hashes and regular-file checks do not defeat that race. Directory
renames may leave a visibility gap. There is no crash-atomic replacement, power-loss durability,
network-filesystem, or unverified platform guarantee. Native proof currently covers
`aarch64-apple-darwin`.

Paths are portable relative ASCII slash paths, at most 240 bytes and 16 components, using
letters, digits, `_`, `-`, `.`, and `/`. Empty/traversal components, leading `.`, trailing
component dots, Windows device names, absolute paths, backslashes, and control characters are
rejected. Case aliases conflict even on case-sensitive hosts: `data/Models/a.rs` and
`data/models/b.rs` cannot coexist. Plans allow at most 1,024 files and 1 MiB combined contents,
checked again after formatting; contributions and several metadata collections also have
1,024-entry bounds. Special files are rejected before managed reads.

You now have the complete path: compose winners, finalize real references, preflight compiled
functions, claim fresh output, then compare or install under cooperative ownership. Use the
[API index](api/README.md) for exact operations and the [recovery guide](guides/recovering-output.md)
when an interrupted writer needs attention.
