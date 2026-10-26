# Task 00008A: UI documentation implementation receipt

Implementation notes for [task 00008A](00008A-ui-docs.md). Branch `trial/3`, base/current
HEAD `9290b404cdb38c519fa71ce956dafe1777dca978`, 2026-10-26. Documentation is complete for
coordinator review; this receipt does not set task status or claim independent verification.
Acceptance criteria: none, as specified by the task.

## Scope and source authority

Created only the two crate documentation trees listed below, plus this separately approved
receipt. No source, test, manifest, lockfile, facade, example, task tracker, or context promotion
change. No commit, staging, push, install, listener, app/browser launch, or inter-agent message.
The coordinator's `target/` and `examples/profile-ui/dist` were not touched.

Read AGENTS/context, the coordinated-development docs stage, the task, installed chrisai-docing
and its guided-learning/API/formatting references, and server/Idea/SQL documentation precedents.
Learning pages were drafted before references; an editorial/formatting pass followed. The
review gates covered first success, ownership of concepts, explained examples, public-only
lookup structure, parameters/defaults/returns/errors, orientation, and working links.

Claims were checked against current framework source/tests and the canonical profile example,
the Slice 5 design packet, [G25](../decisions.md), and the implementation receipts, especially
[lane A rounds](../slice-5-receipt-lane-a-rounds.md) for Head/Owner restoration and
[lane B rounds](../slice-5-receipt-lane-b-rounds.md) for the single-owner signal-root boundary.
Historical API sketches were not treated as current signatures.

## Page inventory

52 Markdown pages: View has 24 (3 entry/concept, 3 guides, 18 API including index); UI has 28
(3 entry/concept, 4 guides, 21 API including index). Related types/functions share their owning
public surface page, following the existing reference convention. Both READMEs supply reading
paths and link the canonical [Profile UI README](../../../../examples/profile-ui/README.md).
No Profile integration source or README was copied into the docs.

### framework-view/docs

All paths below are relative to `crates/framework/framework-view/docs/`:

 - `README.md`
 - `api/README.md`
 - `api/base-path.md`
 - `api/errors.md`
 - `api/page-assets.md`
 - `api/page-contract.md`
 - `api/page-id.md`
 - `api/page-limits.md`
 - `api/page-load.md`
 - `api/page-loader.md`
 - `api/page-renderer.md`
 - `api/page-route.md`
 - `api/page-snapshot.md`
 - `api/page-spec.md`
 - `api/public-location.md`
 - `api/public-status.md`
 - `api/rendered-page.md`
 - `api/route-catalog.md`
 - `api/server-builder-view-ext.md`
 - `concepts.md`
 - `getting-started.md`
 - `guides/add-a-page.md`
 - `guides/contract-recovery.md`
 - `guides/share-routes.md`

### framework-ui/docs

All paths below are relative to `crates/framework/framework-ui/docs/`:

 - `README.md`
 - `api/README.md`
 - `api/alert.md`
 - `api/button.md`
 - `api/field-control.md`
 - `api/initialize-ssr.md`
 - `api/input.md`
 - `api/language-context.md`
 - `api/language.md`
 - `api/layout-provider.md`
 - `api/layouts.md`
 - `api/leptos-renderer.md`
 - `api/link.md`
 - `api/list.md`
 - `api/loader-container.md`
 - `api/loader.md`
 - `api/notifications.md`
 - `api/notifier-container.md`
 - `api/prepared-page.md`
 - `api/routing.md`
 - `api/theme.md`
 - `api/translate.md`
 - `concepts.md`
 - `getting-started.md`
 - `guides/compose-a-root.md`
 - `guides/hydrate.md`
 - `guides/tauri.md`
 - `guides/translate.md`

## Public coverage and behavioral review

View's API index maps all 22 explicit source-declared public items, plus their public methods,
fields, variants, checked conversions, serde/default behavior, and traits. Coverage includes
PageId/PageLoad/PageSnapshot, contracts/limits/constants, public status/location/base/assets,
routes/catalog/encoding, renderer/future/output/errors, and all three native-server surfaces.

UI's API index maps all 41 explicit source-declared public names, their public methods/props,
root modules and dependency/View re-exports. It also identifies public generated
LanguageProviderProps/TranslateProps; consumer checks verify those type names. Related types
include ButtonType, ListItem, Notice/NoticeKind/NotificationOptions, Theme, and PluralCategory.
Private modules/helpers/constants are not presented as caller APIs. Re-exported dependency
manuals are not duplicated.

A name scan found no missing explicit public names; this complements manual field/method,
variant, prop, feature, and trait review rather than claiming semantic completeness by text
search alone. `public-name-coverage.json` records the names/counts.

Specific review covered separate SSR/hydrate/CSR features and facade names; native-only server
and render APIs; explicit/idempotent Tokio initialization; eager owned SSR Head and deferred
reactive SSR Head (S5-F6); cleanup/caller Owner restoration before metadata awaits; trusted
component panic versus caught Head panic; and the single-owner UI execution context.

View docs distinguish logical and encoded JSON ceilings, script escaping from HTML escaping,
strict envelope versus app DTO validation, public status versus dispatch outcome, native
all-matching routing, encoded captures/query ownership, page-ID startup conflicts, builder
discard after registration error, and the request/operation/response/error/finalization table.
The guides cross-link these owners rather than duplicate their contracts.

## Example and gate evidence

All evidence is retained in `/private/tmp/stackpress-docs-00008a/`. Rust 1.96.0,
`aarch64-apple-darwin`; installed `wasm32-unknown-unknown`. No new tool or dependency installed.

| Gate | Final result / evidence |
| --- | --- |
| Mechanical scan | Pass: 52 pages, 46 Rust blocks, 245 relative links/anchors; one H1, tagged/closed fences, no empty parent headings, no double blank lines; `mechanical.json`, `check_docs.py`. |
| Whitespace | Pass including untracked docs via individual `git diff --no-index --check`; no trailing whitespace; `whitespace.json`. Working/staged diff checks clean; staged set empty. |
| Rust formatting | All 46 fences formatted with Rust 1.96 rustfmt, edition 2024. |
| Native SSR consumer | `cargo check --lib --offline --locked`: 45 applicable Rust blocks pass; `ssr-check.log`. |
| WASM hydrate consumer | Same check with `--target wasm32-unknown-unknown`: 38 blocks pass; `hydrate-check.log`. |
| WASM CSR consumer | Same WASM check: 37 blocks pass; `csr-check.log`. |
| Complete fence coverage | 46 distinct blocks, 120 mode/block cases; `blocks.json`, `example-evidence.json`, `verify_examples.py`. Every fence is checked, without hidden source additions to its contents. |
| First-success execution | Three independent native bins pass: safe snapshot roundtrip, in-process ServerBuilder page, complete SSR document; `first_success_0.log` through `first_success_2.log`. |
| Workspace doctests | Exact gate `cargo test --workspace --doc --locked`, offline environment: 855 passed, zero failed/ignored, 23 result groups; `workspace-doc.log`, `doctest-summary.json`. |
| KB validator | Baseline passes with zero errors and 21 existing warnings (`kb-before.log`); final post-write output/comparison is retained in `kb-after.log` / `kb-comparison.json`. |
| Build cleanup | Exactly the owned `native/` and `wasm/` target directories removed; both absent; `cleanup.json`. Consumer sources, locks, scripts, logs, inventories and final doc SHA-256 manifest remain. |

Consumers are disposable independent packages, with lockfiles seeded from the workspace lock.
Offline metadata resolution adds the temporary package and prunes unrelated members; an exact
registry name/version/source subset assertion prevents dependency drift. Actual checks/runs
then use `--offline --locked`. Only one native and one WASM target were used; no root cargo
clean, example asset build/check command, or coordinator artifact mutation was performed.

The first consumer probe failed because Leptos macro expansion needs a direct `leptos` crate
name despite framework re-exports. The docs now specify a direct pinned, defaults-off dependency
with the same selected mode. Final checks use exactly that documented setup. After formatting
and additional API examples, all 120 cases and three programs were rerun successfully.

WASM evidence is compilation, not browser execution. This task did not rerun visual acceptance,
packaging, permission/cancellation proofs, numeric coverage, or runtime/platform acceptance.
Those remain the separately scoped Slice 5 evidence, not new documentation-task claims.

## DX observations, without fixes

 - Leptos macro consumers need a direct pinned dependency in addition to framework re-export imports; demonstrated by the initial probe and documented in setup/API conventions.
 - SSR Head's eager-owned requirement and the single-owner signal execution model are behavioral contracts not fully enforced by handle types. Docs state both; reactive SSR Head remains S5-F6, and no concurrent queue is promised.
 - PageSnapshot.location remains String; basic validation does not establish page/catalog/query agreement. Each app must validate its DTO and canonical location before mounting.
 - Language's standalone locale syntax is broader than page-contract locale syntax. Page users must satisfy both. NotificationOptions serde checks unknown fields, while the store constructor checks ranges.
 - Generic List currently emits `data-profile-id`; documented as current markup without renaming or changing the frozen API.

## Stop

Implementation is ready for the coordinator's docs-stage review and independent re-verification.
No task status or acceptance state was changed. No context or memory update was performed.
