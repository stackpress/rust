# Fast build and test

Use this workflow with the [build benchmark workflow](build-benchmarks.md) for every
development build and test, by implementers and coordinators alike. It keeps edit
loops short and spends full, slow checks only at milestone gates. Record every Cargo,
proof and clean command through the benchmark recorder as before.

## Measured basis (trial/3, 2026-10-29)

Measured on the owner's 10-core, 16 GiB Mac, recorded as `speedbench *` in the store.

| Change | Before | After |
| --- | ---: | ---: |
| Full workspace test run, already built | `cargo test`: 252–357 s | `cargo nextest run`: 61–65 s |
| Build all test targets into an empty target | 118–124 s | 66 s with a warm `sccache` (100% hits, 241 MB cache) |
| Edit one framework-view file, rebuild its tests | 3 s, incremental off | 2 s, incremental on |
| Workspace doctests | 39 s with `cargo test --doc` | unchanged (nextest cannot run doctests) |

Most past time went into cold rebuilds: each stage and each verification started from
an empty target and deleted it afterwards. The biggest wins are therefore a warm target
per effort, a shared compiler cache, and parallel test execution.

## 1. Tools

`cargo-nextest` and `sccache` are installed for the owner (2026-10-29). Install
nothing else without owner approval. If either is missing, fall back to `cargo test`
or no wrapper, and say so in the receipt.

## 2. Environment

Use one of two environment sets. Both share the compiler cache.

```sh
# Edit loop: fastest rebuilds while working on a task.
# Leave CARGO_INCREMENTAL unset: the dev/test profiles are incremental by default.
export RUSTC_WRAPPER=sccache SCCACHE_CACHE_SIZE=20G
unset CARGO_INCREMENTAL
export CARGO_PROFILE_DEV_DEBUG=0

# Gate: milestone, refactoring-round close, coordinator verification, coverage.
export RUSTC_WRAPPER=sccache SCCACHE_CACHE_SIZE=20G
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
```

- `sccache` caches non-incremental crates only. In the edit loop it serves
  dependencies; workspace crates use Cargo's default incremental compilation.
- Never set `CARGO_INCREMENTAL=1` with `sccache`: sccache 0.19 refuses it before
  compiling ("incremental compilation is prohibited"). Unsetting it gives the same
  incremental workspace builds, as verified on 2026-10-29.
- Leave `SCCACHE_DIR` at its default so every worktree shares one cache. The cache is
  keyed by source, flags and toolchain, so sharing it never reuses stale target output.
- Coverage builds (`cargo llvm-cov`) use their own instrumented flags and get few cache
  hits; that is expected.
- Keep `CARGO_BUILD_JOBS` bounded when several agents build at once.

## 3. Target directories

- Give each worktree, or each coordinator verification series, **one** target directory
  and keep it warm across milestones of the same effort. Do not delete it between
  stages; delete it when the effort is merged and finished.
- At the start of a fresh effort (a new worktree or issue), still run `cargo clean` and
  `cargo llvm-cov clean --workspace` once, per the
  [coordinated workflow](coordinated-crate-development.md). With a warm `sccache`, that
  first build costs about half of a truly cold one.
- Keep separate subdirectories when one target would mix incompatible outputs, such as
  Profile's native and WASM targets. Never point two concurrent builds at one directory.
- Give each disposable consumer workspace (a throwaway project a test or proof
  creates) its own subdirectory of the task target, such as `target/<task>/consumer-x`.
  Sharing the main warm target lets it overwrite same-named binaries; a later
  `cargo run` can then execute that stale binary instead of the one you meant.
- When the shared target nears its cap, first delete only its `debug/incremental`
  directory; that frees space while keeping the cached dependency builds.
- Watch disk. Keep each target under the size cap in the brief (default 8 GiB) and
  report the peak. Clear a target mid-effort only when it approaches the cap.

## 4. Two test tiers

### Edit loop (during a task)

Run only what the change can affect:

1. The changed crates, with the features the change touches:
   `cargo nextest run -p <crate> --features <...> --locked`.
2. Their direct dependents when a public API or shared behaviour changed. List them
   with `cargo tree -i <crate> -e normal --workspace --prefix none`.
3. Narrow further with nextest filters while iterating, for example
   `cargo nextest run -p <crate> -E 'test(stream)'`.
4. Build only the browser render mode being worked on (hydrate *or* CSR), and run only
   the browser proofs for the touched example.

### Gate (milestone, round close, coordinator verification)

Run the full set, unchanged in scope from before:

1. `cargo fmt --all --check` and workspace Clippy with `-D warnings`.
2. `cargo nextest run --workspace --all-targets --locked --no-fail-fast`, plus the
   cargo-test lanes in section 5.
3. `cargo test --workspace --doc --locked` for doctests.
4. Feature combinations, generators, separate SSR, hydrate and CSR builds, and every
   browser proof the change could affect.
5. Coverage per changed crate, per [testing and quality](../context/testing-and-quality.md).

A gate never narrows scope to save time. Only the edit loop does.

## 5. Nextest compatibility

The SQL custom harnesses support nextest discovery, exact case selection, and
runtime environment-gated ignored cases. The SQLite safety sweep includes explicit
native cancellation barriers and uninterrupted completions, so it does not require
a timing-flake exception. No package exclusions or cargo-test fallback lanes are
needed for these tests. Live PostgreSQL/MySQL cases still require their opt-in
`STACKPRESS_TEST_..._URL` variables; unset cases remain ignored.

Nextest does not run doctests; keep the separate gate command in section 4.

## 6. Reporting

Receipts state which tier each run used, the environment set, whether `sccache` was
on, and any test run with `cargo test` because of section 5. Label benchmark records
with the tier (for example `edit loop` or `gate`). Declare `--cache cold` for an empty
target even when `sccache` is warm, and note the warm cache in the label.

## 7. Writing new tests

- Prefer one integration-test binary per crate (`tests/main.rs` with modules) over many
  small files: each binary is linked and launched separately.
- Tests must not depend on scheduler timing or machine load; nextest runs them in
  parallel. Use explicit barriers or observable seams, never sleeps, for ordering.
- New custom harnesses must support nextest's `--list --format terse` and `--exact`.
