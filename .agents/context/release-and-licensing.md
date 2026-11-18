# Release and licensing

Authority: current [workspace manifest](../../Cargo.toml), [release inventory](../../tools/release/common.py), G59–G63 and [release versioning workflow](../workflows/release-versioning.md). Load that workflow for an actual preparation task; it owns semver selection, coordinated edits, migration documentation and owner publication steps.

## Release set and version

The [crate inventory](source-structure.md) lists all **23** release packages, each with `publish = true`. Examples, blog plugins, consumers and fuzz crates are unpublished. The release packages use lockstep **0.2.1**. “Beta” describes maturity; the version has no semver prerelease suffix. The [root README](../../README.md), [changelog](../../CHANGELOG.md) and [migration guide](../../docs/migrating-0.1-to-0.2.md) document the 0.2 series and public breaks. The owner published all 23 packages at 0.2.0 on 2026-11-12. Task [00062](../specs/00001-stackpress-rust-foundation/tasks/00062-release-0.2.1.md) records the published archive baseline and additive 0.2.1 preparation; no migration guide is needed for this patch. Preparation does not establish that 0.2.1 has been uploaded.

Breaking public/generated API changes before 1.0 bump the minor (0.1.x → 0.2.0); apply a chosen version to workspace metadata, every internal version requirement, approved nested locks and generated/versioned docs. Keep registry tuples unchanged unless separately approved. Follow the workflow for the full semver and verification rules; never hand-edit managed generated output.

## Crate licences and attribution

| Release packages | Count | Declaration/text |
| --- | ---: | --- |
| `stackpress`, `stackpress-framework`, `stackpress-framework-view`, `stackpress-framework-language`, `stackpress-framework-schema`, `stackpress-framework-sql`, `stackpress-framework-admin`, `stackpress-framework-api`, `stackpress-framework-csrf`, `stackpress-framework-email`, `stackpress-framework-session` | 11 | `license-file = "LICENSE"`, shipping the [Stackpress Community License](../../LICENSE) (the owner's Stackpress License choice, G60). |
| `stackpress-lib`, `stackpress-server`, `stackpress-server-http`, `stackpress-cli`, `stackpress-idea-parser`, `stackpress-idea-transformer`, `stackpress-idea`, `stackpress-sql`, `stackpress-sql-sqlite`, `stackpress-sql-postgres`, `stackpress-sql-mysql`, `stackpress-leptos` | 12 | Apache-2.0 via the manifest's own or inherited `license` declaration, with Apache text shipped in each package. |

Licensing choices belong to the owner. Preserve third-party notices. [ATTRIBUTION.md](../../ATTRIBUTION.md) indexes vendored assets/data and existing Lucide/Feather, Frui and Nodemailer notices; it does not change crate licences.

[Dependency-licence check](../../tools/release/check_dep_licenses.py) compares root and nested source-lock registry packages against [dependency-licenses.toml](../../tools/release/dependency-licenses.toml). It reads cached manifests and the exact recorded metadata fallbacks without resolution/download/network. Unknown, missing or changed licence declarations fail; the snapshot is a frozen inventory for review, not a compatibility decision or a permission to broaden the allowlist. See [packaging documentation](../../tools/release/PACKAGING.md) for its exact scope and fallback rules.

## Verification and package boundaries

[Release gate](../../tools/release/gate.py) runs the current 30 named phases with logs and retained retry evidence: licence check, format, default/compatible Clippy/nextest/watchdog/doctest/Rustdoc, isolated consumers/graphs, generators, blog, controls, README/package/leaf dry runs and facade. It requires the caller's `CARGO_TARGET_DIR` and cached browser tool configuration. Use [fast build/test](../workflows/fast-build-and-test.md) and [benchmark recording](../workflows/build-benchmarks.md). Record verification against the prepared tree; a KB-only edit does not establish a fresh gate result.

Publication preparation also requires standalone Clippy for every candidate, live-inclusive >=90% line coverage for non-facade crates and the finite [fuzz runs](../../tools/release/FUZZING.md). The facade uses its compatible build/import/default-tree proof (G61), and startup-only browser entry exemptions follow G56. [Testing and quality](testing-and-quality.md) defines scope, meaningful tests and evidence limits.

Packaged tests use package-local fixtures/helpers and earlier-published internal dev-dependencies. [Test boundary checker](../../tools/release/check_test_boundaries.py) and [copy checker](../../tools/release/check_test_copies.py) enforce the declared inputs and pinned canonical copies; independent consumers remain repository proofs. Packaging checks archive contents, metadata/licence texts and ordered dependencies. The package driver uses dry-run publishing for its leaf probes; it does not upload.

## Owner publication and rate-limit pacing

[Publish script](../../publish.sh) reads the workspace version and enumerates the same 23 packages in dependency order. It checks crates.io read-only, skips already published versions, and publishes with `cargo publish -p ... --locked` only in its upload path. The upload preflight requires a clean checkout and HEAD matching fetched origin/main; the script never logs in or uses `--allow-dirty`.

The script models crates.io limits as a burst of 5 brand-new names then one per 10 minutes, versus a burst of 30 new versions then one per minute. Its margins are 610 / 65 seconds, with at most six rate-limit retries per crate. Thus 23 existing-package updates fit its initial update burst; recent account activity can still trigger a retry. This is the checked-in script's pacing policy, not a fresh verification of remote limits.

`./publish.sh --plan` performs registry reads and prints the pending names/kinds/wait estimate without upload. Publication, login, commits/pushes and tags remain owner actions unless explicitly authorized; no release preparation or validation pass grants upload authority.

G63 adds exactly seven approved registry tuples to the root (706 total, all 699 prior tuples preserved). The [blog lock checker](../../tools/examples/blog/locks.py) now enforces a pure root subset again; only G58's historical tuple-placement exception is retired. Blog dev reload remains separate from production realtime hosting.
