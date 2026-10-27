# Explicit storage length proof (G38)

This fixture uses the existing app-owned tool to generate a disposable consumer from
[schema.idea](schema.idea). It compiles and runs [main.rs](main.rs) against real SQLite.
The tracked workspace, quickstart and admin outputs are not changed by this proof.

Run from the repository root under the task's parent watchdog and 8 GiB target cap:

```sh
CARGO_TARGET_DIR="$PWD/target" \
TMPDIR="$PWD/target/tmp" \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
G37_PROOF_OUT=/private/tmp/s6-g37/generated-proof \
python3 crates/framework/framework-sql/tests/proofs/lengths/prove.py
```

The target and its `tmp` directory must belong to this effort. The script bounds each
compiler/native command, kills and reaps its process group on timeout, clears both network
URL gates, checks complete generation freshness, and removes the disposable consumer.
Evidence retains exact commands, generated hashes, format-3 artifacts and rendered SQL for all three dialects.
No network database is opened by this proof.

The proof also runs `emit revision --mysql-collation utf8mb4_bin --mysql-page-bytes 16384`
and `emit migration-plan --dialect mysql` using the existing CLI grammar. Both target options
are required together; they are checked before source/history IO and frozen into the revision.

The generated MySQL definitions require explicit `utf8mb4_bin` / 16 KiB expectations.
They render InnoDB DYNAMIC/utf8mb4 clauses, checked whole-key byte budgets and a conservative
all-inline record budget. Off-page record-layout estimation is outside this admitted subset;
unknown layouts refuse. These facts describe operator expectations, not discovered server
settings. PostgreSQL/MySQL migration application remains refused.

Lengths count Unicode scalar values. The fixture proves ASCII, emoji, combining marks,
embedded NUL and trailing spaces; default/provider preparation; private trusted encode/decode;
patches; and whole-batch preflight. Private rules use an explicit host validator; the native
store independently checks storage bounds, including externally written private values.
Browser contract/metadata files omit storage declarations and private length arguments.

SQLite `VARCHAR(n)` has TEXT affinity and does not enforce n. The raw SQL insertion of an
overlong value deliberately succeeds; the generated decoder rejects that row. SQLite resize,
FK-group, sequence, loss-approval, lock and scan-budget checks live in
[storage_length.rs](../../storage_length.rs); fault/cancel/unknown-commit resize cases live in
[migrations/faults.rs](../../../src/migrations/faults.rs).

This is a finite generated-DDL/guard proof. The separate G26 native MySQL case must execute
this fixture's unchanged generated DDL through task 00012's harness before making a native
MySQL DDL claim. It does not establish generated MySQL CRUD or migration application.
