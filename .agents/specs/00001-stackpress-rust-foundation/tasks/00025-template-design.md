# 00025: tiny build-time templates and source-emitter migration

Design checkpoint, 2026-11-01. **Coordinator approved; shared API/grammar frozen
for M0. Engine proof released; emitter migration remains gated.**
Task summary: add reusable text templates to `stackpress-lib`, then migrate every
current Rust source emitter to readable structural templates. Acceptance criteria:
none; this library/refactoring task requires coordinator verification.

## Authority, sources, and checkpoint boundary

The coordinator's current brief authorizes this design document only. Work stays
in the current checkout on `trial/3`; no other trial branch is a source. This
checkpoint changes no production source, manifests, tests, build configuration,
generated files, context files, or existing benchmark records. No agents,
listeners, commits, pushes, installations, or owner-target cleanup are authorized.

Read the [program brief](../brief.md) for roles and scope; the
[original inventory](../design-upstream-inventory.md) and
[Slice 3 deferrals](../slice-3-design-inventory.md) explain that template support
was explicitly deferred, rather than implemented by the existing emitters.
This request releases that capability's design; it does not freeze the whole spec.

Current Rust HEAD inspected: `62768feafeb40b13663751b288c62c83398dd843`.
Upstream `../lib` HEAD: `e2cc6b8cad77b5ad18be8a94e0d40e72e2f6183a`.
Canonical source: `../lib/src/Template.ts`, SHA-256
`83c93e811a3b69c5e41844dbf1a953b217fd4a62712e9d7f7ae9b271a64e0402`;
types: `../lib/src/types.ts:477-495`, SHA-256
`61a81ebe7c56bd962829a759f9bb338207594b3fa3da9845324217ec59234383`.
These are source provenance, not promises of JavaScript compatibility. No
template-specific upstream test file was found in `../lib/tests`.

The [Rust workflow](../../../workflows/rust-programming.md),
[coordinated gates](../../../workflows/coordinated-crate-development.md),
[style](../../../context/rust-code-style.md), and
[quality policy](../../../context/testing-and-quality.md) govern subsequent work.

## Recommended minimum and upstream disposition

Implement a synchronous parser/interpreter for text, paths, conditions, loops,
and compiled inline helper functions. A small parsed template tree is necessary
to match nested blocks; this is not an AST for Rust or a general expression language.
Use existing `serde`/`serde_json` and the standard library; add no third-party crate.

| Upstream surface/behavior | Proposed disposition and reason |
| --- | --- |
| `TemplateEngine`, default `render` export and named `render` | Implement an engine, reusable compiled template, and default-options free `render`; plain Rust functions/types replace classes. |
| `{{path}}`, custom delimiters | Implement strict dotted paths and explicit delimiter options; inserted values are never reparsed. |
| `#?:path`, `#!:path`, `{{\|}}` else | Implement boolean-only conditions and nearest-block else. No implicit JavaScript truthiness. |
| `#@:path item key`, arrays/objects | Implement ordered arrays and ordered object entries, optional aliases, lexical parent/root access, and iteration metadata. |
| `@this`, `@key`, `@index`, `@first`, `@last`, `@odd`, `@even` | Implement explicit loop bindings. Do not flatten item properties into the enclosing object or discard parent context. |
| Helpers `A`, `a`, `.`, `\|` | Implement Unicode upper/lowercase, character-count clipping, and explicit scalar fallback with documented semantics below. |
| `TemplateHelper`, `helpers` registry | Implement explicitly registered compiled inline function pointers; snapshot the registry when compiling. |
| `TemplateHelperOptions` body/engine/Nest access | Replace with borrowed resolved arguments. No callback access to an engine/body, custom block language, or recursive template evaluation. Built-in blocks cover current generation needs. |
| `TemplateResolver` | Fixed strict lookup; arbitrary resolver callbacks are intentionally omitted to keep missing paths, scopes, and limits predictable. |
| `TemplateOptions` | Delimiters and limits implemented; helper registration is an engine method. |
| `container` and arbitrary `%s` substitution | Omit; author the wrapper in the actual template or a compiled helper. No generator needs a second substitution language. |
| `escapeRegex`, public `block`/`method`/`variable` regexes | Intentionally not ported; literal delimiter scanning plus structural parsing replaces regex implementation details. |
| `=` arithmetic / `Function`, dotted method calls | Intentionally not ported; compute values in compiled Rust and pass results as data. Templates cannot evaluate code. |
| Missing/null/unknown helpers silently becoming empty | Deliberate strict Rust contract: errors, with explicit null allowed only as a helper argument. |
| Body `.trim()`, join arguments, automatic coercion | Preserve template whitespace exactly; separators are authored text/conditions. Join arguments and implicit collection-to-text coercion are rejected. |
| Same-name nested block limitation | Fix structurally: match the nearest opener by block kind and exact path, regardless of repeated names. |

No helper or upstream export is silently marked implemented by this checkpoint.
No larger template language is deferred as mandatory work; unsupported surfaces
above are deliberate recommendations for this tiny engine.

## Proposed public API

Canonical module: `stackpress_lib::template`. Re-export the named types from
`stackpress_lib` and the existing `stackpress` facade for consistent convenience;
the free `render` stays namespaced to avoid colliding with UI rendering.
The sketch is the approved M0 contract; implementation evidence is recorded separately:

```rust
pub type TemplateResult<T> = Result<T, TemplateError>;
pub type TemplateHelper =
    fn(&[&serde_json::Value]) -> TemplateResult<String>;

pub struct TemplateOptions {
    pub delimiters: [String; 2],
    pub limits: TemplateLimits,
}
pub struct TemplateLimits {
    pub max_template_bytes: usize,
    pub max_data_bytes: usize,
    pub max_output_bytes: usize,
    pub max_nesting: usize,
    pub max_data_depth: usize,
    pub max_iterations: usize,
    pub max_steps: usize,
}

pub struct TemplateEngine { /* options and compiled helper registry */ }
pub struct Template { /* owned source, parsed nodes, options, helper snapshot */ }

impl TemplateEngine {
    pub fn new(options: TemplateOptions) -> TemplateResult<Self>;
    pub fn register_helper(
        &mut self, name: &str, helper: TemplateHelper,
    ) -> TemplateResult<&mut Self>;
    pub fn compile(&self, source: &str) -> TemplateResult<Template>;
    pub fn render(&self, source: &str, data: &serde_json::Value)
        -> TemplateResult<String>;
    pub fn render_pairs(&self, source: &str, pairs: &[(&str, &str)])
        -> TemplateResult<String>;
}
impl Template {
    pub fn render(&self, data: &serde_json::Value) -> TemplateResult<String>;
    pub fn render_pairs(&self, pairs: &[(&str, &str)]) -> TemplateResult<String>;
}
pub fn render(source: &str, data: &serde_json::Value) -> TemplateResult<String>;
```

`TemplateEngine`, `TemplateOptions`, and `TemplateLimits` implement `Default`.
`render` compiles once for that call. `compile` owns its input so an external
`include_str!` asset or a transient source buffer works equally well. A compiled
template is immutable, `Send + Sync`, and reusable with independent render state.
Registering a helper later does not alter an already compiled template.

`render_pairs` makes a fresh object of string values, rejecting duplicate keys;
it is the ergonomic scalar-map path for short snippets. Typed conditions,
numbers, arrays, and objects use `serde_json::Value`, normally via `json!`.
No conversion through Stackpress rich/native `Value` or JS coercion is implied.
Check pair key/value byte accounting before cloning into this engine-owned map;
the convenience path must not allocate an oversized object and check it afterwards.

```rust
use stackpress_lib::template::{TemplateEngine, TemplateOptions};
use serde_json::json;

let snippets = TemplateEngine::default();
let ty = snippets.render_pairs("Vec<{{inner}}>", &[("inner", "String")])?;
assert_eq!(ty, "Vec<String>");

let engine = TemplateEngine::new(TemplateOptions {
    delimiters: ["[%".into(), "%]".into()],
    ..TemplateOptions::default()
})?;
let contract = engine.compile(include_str!("templates/contract.rs.tpl"))?;
let source = contract.render(&json!({
    "model": "Profile",
    "fields": [{"member": "id", "ty": "String", "wire_literal": "\"id\""}]
}))?;
```

Options are validated at construction. Delimiters must be nonempty, distinct,
at most 16 UTF-8 bytes each, contain no whitespace, and not be prefixes of one
another. Zero byte/work budgets and nesting/depth outside `1..=128` are invalid;
all other positive budgets may be chosen explicitly. No unlimited sentinel.

## Exact grammar and lookup

The following grammar uses the default `{{` / `}}` pair; custom pairs replace
only those boundary tokens. ASCII whitespace around tag content is insignificant.

```text
value       := {{ path }}
helper      := {{ helper_name : path atom* }}
if          := {{#?:path}} body [{{|}} body] {{/?:path}}
unless      := {{#!:path}} body [{{|}} body] {{/!:path}}
each        := {{#@:path [item_alias [key_alias]]}} body {{/@:path}}
atom        := path | JSON-string | JSON-number | true | false | null
path        := root_segment ("." segment)*
segment     := ASCII-identifier | canonical-nonnegative-decimal-index
```

`root_segment` is an ASCII identifier or a reserved loop/root name beginning
with `@`. Identifiers use `[A-Za-z_][A-Za-z0-9_]*`; decimal indices are `0` or a
nonzero digit followed by digits. On objects, numeric segments are exact keys;
on arrays, only numeric segments index elements. No bracket syntax, empty
segments, arbitrary-key quoting, parent operator, comparisons, operators,
function calls, or escaping identifiers. Normalize unusual data keys into a
template view model. Unknown tag/block forms are syntax errors.

Lookup searches innermost loop bindings, enclosing bindings, then the root
object; `@root.path` explicitly selects the original root. The root must be an
object. Item members are explicit (`field.member`), never auto-flattened.
Default aliases are `@this` and `@key`; `@this` is always the current value and
`@key` is an array index or object key. Optional aliases must be distinct ordinary
identifiers; they may shadow outer bindings but cannot use reserved `@` names.
Metadata belongs to the innermost loop; `@index` is zero-based for both kinds.
Arrays retain element order; objects retain their `serde_json::Map` entry order
under the existing `preserve_order` feature. No hidden sorting or hash iteration.

Interpolation emits strings verbatim, numbers as JSON numeric text, and booleans
as `true`/`false`. Missing values, null, arrays, and objects cannot interpolate
directly. Conditions require actual booleans, including for `!`; loops require
arrays or objects. Empty loops visit no body nodes. Extra aliases, loop else,
duplicate else, stray/mismatched/unclosed blocks, and malformed atoms fail.

Only the selected conditional branch performs lookups or helper calls. The same
rule applies to an empty loop body. All branches are still parsed and checked
for syntax/unknown helper names at compile time. A missing `field` inside an
unselected branch succeeds; a selected missing `field` fails.

Inline helpers receive resolved arguments in source order, starting with the
path after `:`. JSON literal arguments support JSON escapes and spaces within
double quotes; single-quoted strings are unsupported. The closing delimiter
ends a tag, including inside an argument: pass delimiter-bearing strings as data
instead. Paths in helper arguments are strict, including all fallback arguments.
Only inline helpers are extensible; `?`, `!`, and `@` are fixed block syntax.

| Helper | Contract |
| --- | --- |
| `A:path`, `a:path` | Exactly one string argument; Rust Unicode uppercase/lowercase mapping. |
| `.:path count [suffix]` | String, nonnegative integer character count, optional string suffix (default empty). Keep at most `count` Unicode scalar values; append suffix only if truncated. Count zero produces suffix for a nonempty input. No grapheme-width promise. |
| `\|:path atom*` | First non-null/non-false/nonzero/nonempty-string scalar, formatted as interpolation; empty string if none. Arrays/objects are type errors. Existing null is allowed; missing paths still error. This explicit helper alone has fallback truthiness. |

Custom helper names use the identifier grammar. Existing built-ins and duplicate
registrations cannot be replaced. Helpers are trusted compiled Rust functions,
receive borrowed values, and return text or `TemplateError::helper` with a static
message. They cannot receive template bodies or engine state through this API.
No helper is called during compilation. Helper output is inserted once verbatim.

## Literal delimiters, whitespace, and actual source templates

Text outside tags is byte-preserved: no trimming, indentation changes, newline
normalization, standalone-tag stripping, or automatic separators. A doubled
opening delimiter emits one literal opener: `{{{{` emits `{{`, and `[%[%` emits
`[%`. Closing delimiters outside tags are ordinary text. Thus `{{{{name}}`
renders literal `{{name}}`; malformed undoubled openers fail instead of vanishing.
Use `[%` / `%]` consistently for source templates so all Rust braces, including
format strings, are literal without brace doubling:

```text
//! Owned [%model%] contract generated from finalized Idea.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct [%model%] {
[%#@:fields field%]    /// Declared stored value.
    #[serde(rename = [%field.wire_literal%])]
    pub [%field.member%]: [%field.ty%],
[%/@:fields%]}
```

For a real generated control fragment, embedded runtime formatting stays exact:

```text
let context = RoleContext {
    id: format!("{}-field-{}", instance, [%wire_literal%]),
    field, role: Role::Field, value,
};
[%#?:labeled%]controls.push(view! {
    <div class="sp-role-value"><span>{label}": "</span>
        <[%module%]::[%symbol%] context/>
    </div>
}.into_any());
[%|%]controls.push(view! { <[%module%]::[%symbol%] context/> }.into_any());
[%/?:labeled%]
```

Migrations use adjacent external `templates/*.rs.tpl` files via `include_str!`
for structural Rust declarations, methods, bodies, and repeated fields. Small
named expression/type templates may be inline when that improves navigation.
Template loops/conditions must visibly own actual source structure; replacing
an existing formatted function with `{{source}}` is not a migration.

Generic lib never validates Rust identifiers, imports, types, literals, HTML,
SQL, output paths, or permissions. Each generator first normalizes/selects its
existing metadata and validates language atoms. A local checked fragment wrapper
can mark a successfully rendered expression/type for later insertion; it grants
no authority to arbitrary schema strings. Fragments are text, never recursively
evaluated or implicitly treated as templates.

Keep Rust string/char/numeric literal encoding in generation, preserving current
escaping exactly (JSON string escaping is not a Rust literal encoder). A narrowly
named literal encoder may retain primitive Debug/number formatting; filename,
diagnostic, registry-key, identifier-normalization, and runtime formatting also
remain ordinary Rust. Complete type/expression/declaration fragments migrate to
templates, including small `Vec<T>`, `Option<T>`, and qualified-path compositions.
The final residual-site inventory must justify every retained construction.

## Limits, errors, and implementation shape

| Default limit | Value and accounting |
| --- | --- |
| Template source | 1 MiB UTF-8 bytes, checked before copying/parsing. |
| Render data | 1 MiB compact JSON-encoded bytes, including keys/escaping/punctuation; depth checked before bounded counting serialization. No unbounded intermediate serialized buffer. |
| Output | 1 MiB UTF-8 bytes in the engine's accumulating result; checked with overflow detection before each append, including helper/fragment output. |
| Block nesting / data depth | 64 each; first block / root object is depth 1. Construction rejects values above the implementation ceiling 128. |
| Iterations | 100,000 total visited entries across all loops in one render, including nested loops. Charge before visiting a body, even if it emits nothing. |
| Steps | 1,000,000 total node visits, path-segment lookups, and helper calls per render. A condition charges only its visited branch; all counters are render-local. |

These bounds cover engine-owned work/output and validated input, not code running
inside a trusted helper, allocations already made by the caller, process memory,
or wall-clock time. A helper can allocate, panic, or perform effects; register
pure bounded helpers in generators. No sandbox or panic-recovery guarantee.
Engine code invokes no arbitrary evaluator, file loader, network, or process.

Use a delimiter scanner and bounded structural parser, with explicit opener
identity and source byte spans. Conditions hold separate child lists; loops hold
their own child list. Bound recursion before descending, including during data
inspection, and ensure tree destruction cannot exceed the accepted depth.
Render into one checked buffer, borrowing data and using lexical scope frames;
never clone the complete root per iteration. No global mutable registry/cache,
unsafe code, async work, locks, or includes/partials/imports in the engine.

`TemplateError` implements `std::error::Error` and exposes `kind()` plus an
optional source byte `span()`. Proposed kinds: `InvalidOptions`, `Syntax`,
`UnknownHelper`, `DuplicateHelper`, `InvalidData`, `MissingValue`, `InvalidType`,
`Limit`, and `Helper`. Diagnostics use fixed context/kind/span, never interpolated
data or entire template text. `TemplateError::helper(&'static str)` is the public
custom-helper constructor; callers own the suitability of that static message.
Report the first error in source/visited execution order; budgets charge before
the action. Invalid options precede parsing; compile errors precede render-data
validation. A failed render returns no partial output and changes no caller data.
Existing generator errors wrap template errors with a cause and template identity,
preserving existing ownership/write/codec error classifications where applicable.

## Migration inventory and dependency boundary

This is a surface inventory, not a claim that every `format!` is source emission.
Coordinator inventory reports roughly 329 production codegen string/format sites;
the residual-site audit, rather than that approximate count, decides completion.

| Existing owner | Source-emission scope |
| --- | --- |
| `crates/framework/framework-schema/src/codegen/{emit,create,inputs,query}.rs` | Contracts/enums/public DTOs, create/patch/query inputs, conversions, validation/default/preparation expressions, metadata functions, type compositions. Normalization/projection/registry policy stays Rust. |
| `crates/framework/framework-sql/src/codegen.rs` and `codegen/{bounds,catalog,conflicts,operations,queries,selection,writes}.rs` | Store types/columns/table definitions, codecs/bounds, relations/include views, reads/writes/conflicts, operation functions and registration. Runtime SQL construction is excluded. |
| `crates/framework/framework-admin/src/codegen/{emit,table,pages}.rs` | Role invocations, descriptor functions, tables/forms/pages, routes and registration, plus coherent use of the existing `pages_host.rs.in` asset. Catalog/binding lookup formatting is excluded. |
| `crates/idea/idea-transformer/src/plan/mod.rs::finish_modules` | Root/partition module declarations with contribution order unchanged; filesystem paths, ownership checks, and writer behavior remain Rust. |
| `examples/generated-profile/codegen/src/profile.rs` | Contract and aggregate registration emitters, selected/disabled shapes; retain the declared immutable runtime-source asset and input provenance. |
| Other `.rs` emission found in example/tool/proof scans | Migrate real executable generator callbacks and generated consumer-source builders; ordinary test inputs/diagnostics are not automatically emitters. Record any discovered additional scope before release. |

SQL and admin agents currently own design-only inventories; this implementer must
not edit their scopes. Expected later ownership here is engine, schema emitters,
transformer aggregation, and Profile codegen, subject to coordinator release.
The coordinator owns shared migration tracking and final lane/API integration.

`stackpress-lib` remains independently usable, executor/host/Idea/UI independent.
The transformer currently lacks a lib dependency: add the existing workspace
crate when aggregation migrates. Schema/admin gain optional lib dependencies
enabled by `codegen`; SQL's existing optional lib dependency is enabled by its
`codegen` feature as well as `server`. Profile already depends on lib. Confirm
normal-edge feature graphs and no reverse dependency/cycle at M0/integration.
No generator-only dependency is added to generated DTOs, pages, or stores.
No Cargo dependency/manifests change is part of this checkpoint.

## Focused risk proofs and verification process

| Risk | Prevention and proving test after release |
| --- | --- |
| Same-name nesting / wrong else | Parsed child structure; nested identical `?`, `!`, and `@` paths, mixed blocks, adjacent siblings, inner/outer else, mismatch and EOF cases. |
| Missing values in unselected branches | Lazy branch/empty-loop evaluation; selected missing path fails, unselected missing path succeeds, unselected helper invocation counter remains zero. Unknown helper/syntax still fails compilation. |
| Rust braces / delimiter corruption | Default escape and custom-pair tests with `{{x}}`, `format!("{}", x)`, doubled Rust braces, quotes, raw strings, CRLF, Unicode, trailing newline, and inserted delimiter-bearing fragments; byte-equality assertions. |
| Scope capture / order drift | Nested shadowed aliases, `@root`, root-parent access, primitive/object items, empty arrays/objects, metadata and insertion-ordered object entries; no implicit item spread. |
| Hidden coercion or fallback | Type matrix for missing/null/false/zero/empty/string/collection; helper arity, quoted arguments, Unicode clipping/count-zero, missing fallback argument, duplicate registrations and pair keys. |
| Expansion/parse denial of service | Exact limit and limit-plus-one cases; multibyte bytes, nested aggregate iteration charging with empty output, overflowing configured counters, depth ceiling, empty malformed opener; subprocess timeout for hostile nesting corpus. |
| Cross-render leakage | Reuse one compiled template sequentially/concurrently with different data and failed renders; assert `Send + Sync`, independent budgets and immutable helper snapshots. |
| Source injection / unsafe fragments | Existing identifier/type/literal/import checks remain before view-model construction; hostile labels/keys/quotes become encoded data; inserted `{{...}}`/`[%...%]` is never executed. Compile resulting fixtures. |
| Helper overclaim | Compiled helper errors/large returned text hit output checks; document trusted allocation/panic boundary and do not assert a sandbox or deadlock containment. |
| Migration changes generated behavior | Compare output path sets and raw/formatted bytes, clean and repeat generation, freshness, rename/removal/disabled cases, generated consumers and runtime registration. Preserve event order/default/null/permission behavior. |

The coordinator supplied a **190-file expanded baseline** and reported workspace
freshness passing. Its manifest is
`/var/folders/w2/g12vs0ts64g_3k6kjf_dd_bc0000gn/T/stackpress-template-baseline-19tcw996/generated-hashes-expanded.json`.
This checkpoint has not independently rerun that proof. Before migration, confirm
the manifest still matches the agreed base; preserve it. Compare all baseline
files after regeneration and audit files outside that baseline too. Do not
update goldens merely to conceal a template behavior difference.

## Implementation steps and gated sequence

1. **Design review (current stop):** coordinator reviews this API/semantics with
   SQL/admin inventory findings and returns the shared API plus milestone release.
2. **M0, only after release:** implement generic parser/render core and focused
   nesting, strict-selected-branch, delimiter, output/budget and helper proofs;
   prove external assets, scalar pairs, standalone lib and facade use. Stop for
   coordinator re-verification before broad migration.
3. **M1–Mn:** complete engine helpers/errors/docs, then migrate released emitter
   groups using actual structural templates. Each group's tests and output parity
   pass before advancing. Shared manifests/facade have a single assigned owner.
4. **Pre-round stop:** provide working tests, per-changed-crate coverage, residual
   emitter inventory, generated-byte/path parity, and feature/dependency evidence.
5. **Round 1, test-verified:** review parser/block matching, lazy evaluation,
   branches, counters, error precedence and migration behavior; refactor findings.
6. **Round 2, test-verified:** review generic engine/generation separation,
   fragment ownership, view-model cost, template navigation and dependency edges.
7. **Round 3, test-verified:** review consistent APIs/templates/errors, Rustdoc,
   executable examples, per-block/branch comments, formatting and warning-clean docs.
8. **Dedicated safety/soundness round:** inspect malformed input, recursion and
   destructor bounds, checked arithmetic, helper trust, allocations, concurrent
   reuse, failures and output-writer interaction. Repeat targeted rounds while
   material findings remain; record justified no-change findings without filler.
9. **Closeout:** coordinator independently re-verifies; crate template docs use
   chrisai-docing, and generated consumer docs change only where the API requires.

On authorized M0, use only `CARGO_TARGET_DIR=target/template-refactor`, including
benchmark-recorded `cargo clean` and `cargo llvm-cov clean --workspace` before its
first build. Never clean the owner's other targets. Keep this effort warm and at
or below 12 GiB; isolated disposable consumers get distinct subdirectories.
Follow [fast builds](../../../workflows/fast-build-and-test.md) for `sccache`,
incremental edit loops and non-incremental gates; install nothing.

Record all later Cargo/proof/clean/check/lint/doc/coverage attempts through the
[benchmark recorder](../../../workflows/build-benchmarks.md), wrapping an outer
proof once, without double-counting nested Cargo. Receipts include record IDs,
counter changes, failures, peak target usage, `command_seconds`, and separately
observed `cargo_build_seconds` (unavailable/partial remains explicit).

Use narrow nextest tests during edits. At gates run workspace nextest all-targets,
separate workspace doctests, rustfmt, Clippy `-D warnings`, warning-clean Rustdoc,
each affected generator/freshness/consumer proof and valid codegen feature matrix.
Build distinct SSR/hydrate/CSR consumers wherever changed emission affects them;
generated HTML source checks alone do not prove browser/installed-target behavior.
Any browser proof requiring a listener uses devmetrics under coordinator release.
Measure executable line coverage per changed crate, at least 70% development;
90% applies only to publication preparation, which is not this task. Keep generated
execution evidence and unmeasured target limits visible. New tests use explicit
seams/barriers, never scheduler sleeps; hostile nesting probes fail via bounded
subprocess watchdogs, with no locks/async/deadlock machinery added to this engine.

## Implementation notes

Design-only inspection completed against the current checkout and upstream source.
No public/product question requires an owner answer: the recommendations above
were approved as the frozen shared M0 contract; subsequent proof lives in the M0 receipt.
Before editing, workspace validation already failed on 11 missing historical
temporary screenshot links, with inherited line-cap/link-description warnings.
Post-write validation retains those same 11 errors; its only diagnostic for this
file is the permitted 201-500-line coherence warning. Keep this checkpoint in one
file so reviewers can assess API, grammar, bounds, migration, and proofs together.
New-file whitespace checking reports no defects. Concurrent edits to task status
belong to coordinator tracking; this implementer wrote only this checkpoint.

## Verification notes

Coordinator review pending. No Rust code tests, builds, cleanup, generated
freshness checks, coverage, or implementation/refactoring rounds were run by this
design checkpoint. Static document validation does not prove engine behavior.

## Acceptance notes

Acceptance criteria: none. Coordinator approved this checkpoint and released the
engine-only M0 proof. Emitter migration requires a follow-up release. Context
promotion remains skipped.
