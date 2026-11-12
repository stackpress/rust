# Task 00056: attribution and dependency licences

Implemented G62 on `hardening/attribution-licences`, based on `7213e77`.
Coordinator review/merge and owner licence-policy review remain separate.

## Delivered

- [Attribution index](../../../../ATTRIBUTION.md): three Lucide/Feather bundles
  (79 SVG masters and compiled copies), Frui switch/editor/source-control CSS and
  country/currency/colour data, Nodemailer 7.0.13 tables, and original example assets.
- [Offline checker](../../../../tools/release/check_dep_licenses.py), frozen
  [expression inventory](../../../../tools/release/dependency-licenses.toml),
  negative controls, the `dependency-licenses` release-gate phase, and
  [packaging reference](../../../../tools/release/PACKAGING.md#dependency-licence-snapshot).
- Source locks are discovered recursively, including consumers/examples/blog/fuzz
  when present. Counts deduplicate exact name/version/registry identities.

## Frozen expressions for owner review

22 locks; 705 unique registry packages; 38 exact licence expressions.
This records current declarations, including legacy slash syntax; it does not set
licence policy or assert compatibility. No expression was added speculatively.

| Exact expression | Packages |
| --- | ---: |
| `(MIT OR Apache-2.0) AND Unicode-3.0` | 1 |
| `0BSD` | 1 |
| `0BSD OR MIT OR Apache-2.0` | 1 |
| `Apache-2.0` | 4 |
| `Apache-2.0 / MIT` | 1 |
| `Apache-2.0 AND ISC` | 1 |
| `Apache-2.0 AND MIT` | 1 |
| `Apache-2.0 OR BSL-1.0 OR MIT` | 1 |
| `Apache-2.0 OR ISC OR MIT` | 3 |
| `Apache-2.0 OR MIT` | 59 |
| `Apache-2.0 WITH LLVM-exception` | 1 |
| `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | 5 |
| `Apache-2.0/MIT` | 5 |
| `BSD-2-Clause OR Apache-2.0 OR MIT` | 2 |
| `BSD-3-Clause` | 5 |
| `BSD-3-Clause AND MIT` | 1 |
| `BSD-3-Clause OR MIT OR Apache-2.0` | 2 |
| `BSD-3-Clause/MIT` | 1 |
| `BSL-1.0` | 1 |
| `CC0-1.0` | 1 |
| `CC0-1.0 OR MIT-0 OR Apache-2.0` | 1 |
| `CDLA-Permissive-2.0` | 2 |
| `ISC` | 3 |
| `ISC AND (Apache-2.0 OR ISC)` | 1 |
| `ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0)` | 1 |
| `MIT` | 164 |
| `MIT OR Apache-2.0` | 322 |
| `MIT OR Apache-2.0 OR BSD-1-Clause` | 1 |
| `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | 2 |
| `MIT OR Apache-2.0 OR Zlib` | 3 |
| `MIT OR Zlib OR Apache-2.0` | 2 |
| `MIT/Apache-2.0` | 30 |
| `MPL-2.0` | 6 |
| `Unicode-3.0` | 34 |
| `Unlicense OR MIT` | 10 |
| `Unlicense/MIT` | 2 |
| `Zlib` | 7 |
| `Zlib OR Apache-2.0 OR MIT` | 17 |

702 declarations came from cached registry Cargo.toml files. Three exact crates.io
entries use coordinator-authorized public API metadata supplied on 2026-11-12:

| Package | Expression |
| --- | --- |
| `atomic-polyfill@1.0.3` | `MIT OR Apache-2.0` |
| `curve25519-dalek-derive@0.1.1` | `MIT/Apache-2.0` |
| `fiat-crypto@0.2.9` | `MIT OR Apache-2.0 OR BSD-1-Clause` |

Each fallback records `source = "crates.io API, 2026-11-12"` in TOML. This worker
made no network request. Only these exact name/version/crates.io identities can use
metadata, only while source is absent. Cached declarations take precedence; malformed
cached sources and all other uncached packages fail. Metadata expressions still
must be listed. A licence-file-only declaration is read and fails for SPDX review;
no identifier is inferred from its text.

## Verification and review rounds

- Initial baseline: 41 release-tooling tests passed; licence inventory passed.
- Round 1, logic/complexity: extracted package-source selection to remove nested
  exception handling from the inventory loop; 41 tests passed.
- Round 2, responsibility/abstraction: reviewed lock discovery, manifest reading,
  exact fallback ownership and membership comparison. Added identity-mismatch,
  missing-declaration and missing-provenance controls; 44 tests passed. No further
  abstraction was justified.
- Round 3, consistency/documentation: reviewed docstrings, error/provenance output,
  all local links and the exact-expression/fallback documentation. No further
  behavioral changes were justified; all 44 release-tooling tests passed again.
- Final licence check: 705/705 declarations, three reported fallbacks, zero failures.
- Strict `python3 tools/release/check_packages.py --output /tmp/00056-packages.json`:
  all 23 release crates passed, zero failures, no pending-test waiver.
- Agent Workspace validator: identical inherited baseline before/after, **146 errors
  and 44 warnings**; not a clean validation. Full diagnostics were compared.
- `git diff --check` and public-document local-link checks passed. Hash comparison
  confirmed all 59 existing lock/NOTICE/LICENSE files unchanged. No Rust source,
  manifests, own licence fields or notices were edited. No network, install, listener,
  commit, push or branch/worktree consultation occurred.

## Recorder evidence

Store: `/private/tmp/00056-build-benchmarks` (kept within this task's writable scope,
without consulting or writing another worktree). All seven attempts passed; zero
compilation-emitting attempts or full-workspace rebuilds. Cargo stage times are
unavailable for these Python/inventory commands; no Rust coverage claim is made.

| Record | Command seconds |
| --- | ---: |
| `20261112T041740.766698Z-a4dc3e02e4b3` | 0.195910 |
| `20261112T041741.465677Z-6c42f5fbe003` | 0.660085 |
| `20261112T041819.298833Z-dc85cdd1c345` | 0.154101 |
| `20261112T041820.121717Z-999c235b147a` | 20.342370 |
| `20261112T041910.618857Z-0f14bf28f1eb` | 0.160280 |
| `20261112T041934.394726Z-ef95d7438698` | 0.171986 |
| `20261112T041934.394947Z-ead176a3886e` | 0.802996 |

Package report: `/tmp/00056-packages.json`. Validator snapshots:
`/tmp/00056-validator-{before,after}.log`. Full release gate/compilation was outside
this tooling-only task; package inventory and focused release-tooling checks passed.
