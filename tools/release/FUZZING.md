# Parser fuzzing before release

Run the three unpublished targets described in [fuzz/README.md](../../fuzz/README.md) for at least 300 seconds each, recording tool versions, total executions and any crashes in the release receipt. Minimize crashes and add owning-crate regression tests before accepting fixes.

The fuzz workspace owns its lock. Assert that root `Cargo.lock` bytes and all registry identity/source/checksum tuples remain unchanged; do not add fuzz dependencies to release packages. G62 permits only `libfuzzer-sys 0.4.10` and its mandatory `arbitrary 1.5.0` beyond the seeded root tuples, within the fuzz lock. Keep lock comparisons and fuzz logs with the benchmark evidence.
