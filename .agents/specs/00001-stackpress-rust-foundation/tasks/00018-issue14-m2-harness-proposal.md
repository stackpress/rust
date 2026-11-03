# Issue 14 M2 inherited harness proposal

Status: proposed. Implementation stays pending coordinator/owner authorization.

The mandatory full-workspace nextest gate executes `examples/generated-profile/codegen/tests/consumer.rs`. Its existing consumer test calls `cargo generate-lockfile` three times and sets a separate temporary Cargo target. The first M2 gate finished before interruption and passed; this was initially treated as a standing-rule conflict. The repository lock is unchanged. The coordinator subsequently ruled that this existing disposable-consumer behavior conforms to the standing rules, so the completed gate counts.

The [coordinated workflow](../../../workflows/coordinated-crate-development.md) says: “Work discovered inside a task becomes a side quest ... It stays `proposed` until the owner approves its scope.”

Recommendation: authorize the behavior-preserving test-only change in the prepared patch (`/private/tmp/issue14-m2-harness.patch`, temporary artifact, since removed). Seed independent locks from the repository lock, resolve local membership via offline metadata, assert exact external identity/source/checksum tuples, and put disposable consumers beneath the supplied task target. Keep all existing consumer assertions and locked compilation. No runtime/public API changes. Rerun this fixture and the full nextest gate; retain the completed workspace Clippy/doctest evidence unless source changes justify repeating it. Include the test-only crate in coverage accounting.

The patch is prepared outside the repository; the existing test has not been edited. Other authorized M2 work can continue while this decision is pending.

Coordinator ruling (2026-10-29): do not apply this patch in issue 14. The prohibition protects the repository lock; this existing test only resolves disposable consumer locks and cleans its small temporary target. The coordinator will raise this proposed follow-up separately with the owner. M2 proceeds without this test change.
