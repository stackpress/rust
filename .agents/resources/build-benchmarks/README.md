# Build benchmark records

This directory holds persistent, per-command records and logs created by
`.agents/scripts/build-benchmark.py`. Tracking starts with adoption of the
[build benchmark workflow](../../workflows/build-benchmarks.md); historical
receipts are not counted automatically.

Run `python3 .agents/scripts/build-benchmark.py summary` from the repository to
see counters and individual durations. No records means zero recorded attempts,
not zero historical builds. Keep the `runs/` records when deleting Cargo targets.
The recorder uses this directory in the main checkout for linked worktrees too.
Tests use temporary stores and never add synthetic builds here.
