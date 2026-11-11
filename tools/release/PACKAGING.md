# Release manifests and packaging proofs

The release scope is the explicit 23-package list in `common.py`. Private examples
and consumers retain `publish = false`. Every candidate has version `0.1.0`.

Run these proofs through `.agents/scripts/build-benchmark.py` with the environment
and shared store specified in Task 00050:

```sh
python3 tools/release/check_packages.py
python3 tools/release/check_readmes.py
python3 tools/release/check_facade.py
python3 tools/release/package_workspace.py
```

The package checker obtains each file list from `cargo package --list`, checks all
Rust source references regardless of `cfg`, checks exact approved licence text,
metadata, publication-ordered internal edges, attribution, and the 10 MB limit. `--archives`
checks actual archive contents, sizes and normalized dev-dependency stripping.
Comments and string literals containing generated Rust are opaque. Literal includes
and `concat!(env!("CARGO_MANIFEST_DIR"), ...)` work; computed paths fail closed.
The scanner does not expand arbitrary macros. Package compilation supplies that
additional evidence. `python3 -m unittest discover -s tools/release -p 'test_*.py'`
runs its negative controls.

`check_facade.py` builds the default, each declared feature individually, then every
compatible native capability together with SSR. It compiles an independent consumer
with explicit imports for every advertised re-export and checks that the facade's
default normal dependency tree has only `stackpress-lib` among Stackpress dependencies.
This is the owner-approved G61 interpretation; it is not a `2^N` powerset. Hydrate and
CSR compile individually; the aggregate/session graph proofs also use the WASM target.
No renderer crate is built with `--all-features`.

`gate.py` combines the requested default/compatible workspace gates, existing consumer
and generator checks, blog gate, package proofs, README doctests and facade proof:

```sh
python3 tools/release/gate.py
```

Set `BLOG_PLAYWRIGHT` to the already installed Playwright module directory before
the gate. The blog uses in-process browser routing and starts no listener. If a
phase fails, `gate.py --only PHASE` reruns that phase and appends its evidence; the
effective result uses the latest attempt while preserving the original failure.

Before a commit, add `--allow-dirty` to the gate/package driver. It only acknowledges
local implementation changes. The owner runs release commands from a clean checkout.
The coordinator temporarily authorized `--pending-tests`: it lists only the exact 52
original file-reference omissions and 17 stripped-dependency references recorded in
Task 00050 evidence. Every other failure
remains strict. Omit that flag after merging the tests lane; a pending-mode pass is
not a claim that extracted package tests are self-contained.

Scratch README/facade consumers live under the caller's target directory. Their first
resolution seeds the root lock, runs one offline refresh, and asserts a registry tuple
subset. Later commands remain locked/offline. Existing nested locks are preserved
unless a locked command demonstrates that the approved one-time refresh is required.
No command logs in or uploads a crate; `package_workspace.py` always uses `--dry-run`
for its five leaf publish probes.

Normal/build dependencies and earlier-order candidate dev-dependencies have registry
versions. Later-order and unpublished dev-dependencies stay path-only and must disappear
from normalized archives. The checker rejects versioned later-order dev edges and Rust
references to stripped dependencies in any packaged source, including unit test helpers.
Comments and generated-source strings do not count as imports. Pending mode waives only
the exact approved test lines, never dependency declarations or library compilation.

The provisional manifest allowlists omit email's integration/generation tree except
its two embedded TLS assets, and SQLite's trybuild tree and host example. The exact
51 paths and eight later-order edges are listed in Task 00050's `dev-cycle-policy.json`.
The coordinator will reconcile those allowlists with the other workers' moves.

Workspace packaging remains locked/offline and includes archive lockfiles. The five
initial offline leaf dry-runs required HTTP; their final dry-runs permit anonymous
registry reads as authorized by the coordinator. They always retain `--dry-run` and
`--locked`. No credential is added. `--exclude-lockfile` was investigated and rejected:
it could hide first-release dependency ordering failures rather than resolve them.
