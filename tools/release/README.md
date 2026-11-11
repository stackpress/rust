# Release test boundaries

Run `python3 tools/release/check_test_boundaries.py` for all 23 candidates. During
Task 00051, `--pending-coverage` reports the five separately owned crates as pending.
All Rust under `src/` is scanned, including feature-gated test modules; `tests/`,
`examples/`, and `benches/` are scanned too. Nested unpublished consumer packages
remain repository proofs and cannot supply packaged test inputs.
Support scripts are scanned for fixed reads; Python `__file__`/parent anchors
and JavaScript `__dirname` joins are resolved without executing the script.
Caller-supplied source paths and temporary directories are not fixed repository inputs.
The facade's
repository application collection is scanned only if it gains Cargo example targets.

Run `python3 tools/release/check_test_copies.py` to merge and verify every
`tools/release/test-copies*.json` manifest. Duplicate destinations are errors.
Schema: `{"copies":[{"source":"repo/path","copy":"repo/path","sha256":"source SHA-256","provenance":"optional exact first comment line","divergence_reason":"optional nonempty reason","copy_sha256":"required for intentional divergence"}]}`.
Paths are repository-relative. Without a divergence reason the copy must equal the
source bytes, optionally preceded by `provenance` and one newline. Data fixtures
omit `provenance`, preserving every byte. A source change always fails its pinned
hash, including when the copy intentionally diverges. Refresh copies and hashes
together after reviewing the canonical change; the checker never rewrites files.

The checker regression suite is `python3 -m unittest discover -s tools/release`.
Source scanning complements the coordinator's extracted-package verification;
it does not certify normalized dev-dependencies or platform-specific behavior.
