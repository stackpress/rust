# Stackpress parser fuzzing

This unpublished nested workspace has its own `Cargo.lock`. Root Cargo commands do not include it. Its lock is seeded from the root; only `libfuzzer-sys 0.4.10` and its required `arbitrary 1.5.0` are additional registry tuples. Root dependencies and lock must remain unchanged.

Install the approved tools once:

```sh
cargo install cargo-fuzz --locked
rustup toolchain install nightly --profile minimal
```

Run from the repository root, using a dedicated target directory and the repository benchmark recorder:

```sh
CARGO_TARGET_DIR=target/parser-fuzz CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 RUSTC_WRAPPER=sccache \
python3 .agents/scripts/build-benchmark.py run --label 'parser fuzz Idea' --kind test --scope selected --cache warm -- \
  cargo +nightly fuzz run idea_parser -- -max_total_time=300 -max_len=1048577 -timeout=10 -print_final_stats=1
```

Replace `idea_parser` with `admin_csv` or `lib_query_form` for the other targets. Each run spends 300 seconds fuzzing after compilation. The maximum length crosses the one-MiB boundary; individual slow inputs time out after ten seconds. Error returns are expected. Panics, sanitizer failures and harness assertion failures produce a crash artifact.

| Target | Public entry and checks | Seed provenance |
| --- | --- | --- |
| `idea_parser` | `stackpress_idea_parser::parse` for valid UTF-8, with default parser budgets | `crates/idea/idea-parser/tests/fixtures/{type,prop,model}.idea` |
| `admin_csv` | `transfer::parse` includes raw admission, decode, delimiter sniff and structure; successful `write` output must round-trip; a second parse uses lower ceilings | `framework-admin/tests/transfer.rs` and `transfer_tolerant.rs` cases |
| `lib_query_form` | `stackpress_lib::data::{parse_query, parse_body}` with URL-encoded media type and default budgets; both entries must agree on acceptance for valid UTF-8 | `crates/lib/tests/data_contracts.rs` query/form cases |

Tracked seeds stay small. LibFuzzer may add files to `fuzz/corpus/<target>`; retain useful minimized regression seeds deliberately. Build output, crash artifacts and coverage directories are ignored.

For a crash, minimize before fixing:

```sh
cargo +nightly fuzz tmin admin_csv fuzz/artifacts/admin_csv/crash-<hash> -- -max_total_time=60
```

Add a test in the owning crate that fails on the pre-fix code, then fix and rerun. Route lib or Idea parser crashes to their owner with the minimized bytes. Keep the original artifact and complete benchmark log as evidence. A finite fuzz run is additional crash evidence, not exhaustive correctness or platform proof.
