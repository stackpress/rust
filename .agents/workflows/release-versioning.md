# Release versioning and publishing

Use this workflow whenever the Stackpress crates are prepared for a new crates.io
release. The release scope, licences and packaging rules come from decisions G60–G62
and the [publication plan](../specs/00001-stackpress-rust-foundation/tasks/00047-publication-plan.md).
Publishing itself, `cargo login` and pushing tags stay with the owner.

## 1. Choose the next version

All 23 release crates share one version (lockstep, G61), set in
`[workspace.package] version` in the root `Cargo.toml`. Pick it from what changed
since the last published release, using Cargo's semver rules. Before 1.0, the
**leftmost non-zero** number is the "major" one.

| Since the last release | Before 1.0 (`0.x.y`) | From 1.0 (`x.y.z`) |
| --- | --- | --- |
| Any **breaking** change to a published crate | bump the minor: `0.1.3` → `0.2.0` | bump the major: `1.4.2` → `2.0.0` |
| Only **additive** changes (new items, new optional features) | bump the patch: `0.1.0` → `0.1.1` | bump the minor: `1.4.2` → `1.5.0` |
| Only **fixes** with no API change | bump the patch | bump the patch |

Cargo treats `0.1.x` and `0.2.x` as incompatible, so a dependency written as
`"0.1"` never picks up `0.2.0` by itself. That's why a breaking change must bump
the minor before 1.0: a patch bump would silently break people who depend on `"0.1"`.

**Breaking** includes, for any public item of any release crate:

- removing or renaming an item, module, feature or re-export;
- changing a function's parameters or return type (for example `T` → `Result<T, E>`);
- adding a required trait method, or a field to an exhaustive public struct or enum;
- tightening trait bounds, or changing the default features;
- changing generated code that applications compile against;
- raising the minimum Rust version (`rust-version`). Treat this as breaking unless
  release notes say otherwise.

Behaviour changes that applications can observe (different errors, input accepted or
rejected differently) are not semver breaks by themselves, but they belong in the
release notes, and in the migration guide when apps may need to react.

Find the changes by diffing the public API against the commit the last release was
published from. Check the crates.io `published_by`/`created_at` data, or compare a
downloaded `.crate` with the git tree. Review every `pub` change across the 23 crates
and their generated templates. Optional tooling, such as `cargo-semver-checks`, needs
owner approval to install.

## 2. Apply the version

1. Set `[workspace.package] version` in the root `Cargo.toml`.
2. Update every internal `version = "..."` requirement between release crates
   (normal, build, optional, target-specific and earlier-order dev-dependencies) to
   the new version.
3. Refresh the root lock (workspace package versions only; the registry tuples must
   stay identical) and every nested consumer, example or blog lock with the approved
   root-seeded offline procedure. Never `cargo update`.
4. Regenerate managed output that embeds the version, using the official generators,
   and run every freshness `--check`.

## 3. Update the docs

- `CHANGELOG.md`: a new section for the version and date, grouped as **Breaking**,
  **Added**, **Changed**, **Fixed** and **Security**, written for users.
- A migration guide in `docs/` for every breaking release: before/after code for each
  breaking change.
- The root and crate READMEs: version numbers in install snippets, plus the release
  status (for example "beta").
- The [publication plan](../specs/00001-stackpress-rust-foundation/tasks/00047-publication-plan.md)
  runbook, if the process changed.

## 4. Verify

Run the release gate once on the final tree, from a clean checkout:

```sh
python3 tools/release/gate.py
```

That includes packaging all 23 crates, the dependency-licence check and leaf dry runs.
Also run every candidate's standalone Clippy, plus live-inclusive coverage of at least
90% per non-facade crate (G61), as in task 00057.

## 5. Publish (owner)

1. Commit, then push `main`.
2. `./publish.sh --plan` to see what will be published and the rate-limit waits.
3. `cargo login`, then `./publish.sh`. It publishes in dependency order, skips crates
   already at that version, and paces itself to crates.io's limits (new versions:
   burst of 30, then 1 per minute; brand-new crates: burst of 5, then 1 per 10 minutes).
4. Tag the release (`git tag v<version>` and push the tag).

A published version can't be re-uploaded. Fix mistakes with a new patch release, and
`cargo yank` a broken version if needed.
