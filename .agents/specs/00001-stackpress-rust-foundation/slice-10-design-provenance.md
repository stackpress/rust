# Slice 10 source provenance and G43 revision receipt

Draft / Not Frozen; [entry](slice-10-design-decisions.md). Revised 2026-10-30 Asia/Manila, following coordinator instruction and G43 read from the exact main-checkout path. The prior nine-file draft was reconstructed and saved outside the repo at `/private/tmp/slice10-probes/design-before-g43.json` before editing; it is superseded planning, not accepted behavior.

## Authority and source coverage

Worktree `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-slice10`, branch trial/3-slice10-api, HEAD `11e98b102c8c6931dc750e476be7057448cd2acf`. G43 main checkout HEAD `3cf407f`; its `.agents/specs/00001-stackpress-rust-foundation/decisions.md` SHA-256 at read was `99ed6b0d24f12e9fb7d769ebe796da70aa864606a9d3e8d700d17c21dd4fae04`. Main G43 was read only; local historical decisions/status/context records were not changed. No other trial branch or stale target was inspected.

Coordinator-supplied `/Users/cblanquera/server/projects/stackpress/packages/stackpress-api/src` is absent. Canonical available upstream root is `/Users/cblanquera/server/projects/stackpress/stackpress`, HEAD `a71d683051ba8350fdd12d6b5a33f268fdcc285f`, package API version 0.10.8. Source remains in place read-only; no archive into .agents. Re-read every API src module plus schema, styles, package exports and aggregate registration; inspect actual template endpoint/population/admin usage before assigning intent.

Retained KB routing: [framework supplements](../../references/00041-framework-package-supplements.md), [configuration catalog](../../references/00023-upstream-configuration-catalog.md), [interface examples](../../references/00032-upstream-interface-exposure-examples.md), [command and plugin lifecycle](../../references/00024-upstream-cli-and-plugin-contracts.md). G43 overrides earlier hardening proposals/imported inconsistencies. G39–G42 and #14 direct/plugin conventions remain. The older source-structure framework-ui wording does not revive that removed crate after G41.

## API source hashes

Paths are under canonical upstream `packages/stackpress-api/`, except aggregate row.

| Path | SHA-256 |
| --- | --- |
| `package.json` | `edfb4dd192d6fc064daf3838443dc34d81783d37cd7a325f0d4e217adc7741d3` |
| `src/index.ts` | `234ccd65796fe1cef621c6d634c1ba70d4b189259b71df777a72261fbfefbeca` |
| `src/types.ts` | `124747ac4b0e6abc4297bea08b2ec5ee8904820f3cf1da646d5879d017ee7ba9` |
| `src/Exception.ts` | `c40e8f34c75227b01e1effd7ed5e914a7ea0511ed8c206cbe048fa5d663d81b9` |
| `src/plugin.ts` | `a63a25af8cd1288420fc9bb7451230573acaa60b99e9103f6720a3c5d353e600` |
| `src/helpers.ts` | `4748a377184f34038aec20353e5d6331575cdfb395013bedf6613466eda81a7b` |
| `src/pages/oauth.ts` | `52bc29b52c36825d6d7be7a1f68a7c62de1d82b7087f0bfcc9f2cf31b6d5ce6a` |
| `src/pages/token.ts` | `4a9b185f4a64adfc5046e3eddfd576bc302db621d6a23d5b0c1253662a97a7a8` |
| `src/views/oauth.tsx` | `e2750f64ff717179909c4dca2020f61448eede64bc66a9f3ad8be164cd98b4cd` |
| `schema.idea` | `5bddbdfa9c58be140cdfdab4c9cb9ec670e3dec4fc625c098c97ea5a6a2adebf` |
| `styles.css` | `defded866948d4b37df1c90584194734277a231688deeb4975ea942abc64779e` |
| `README.md` | `35031b61a4e3ab0f89cc68e608313a7278821f86db341a5b00a43fbaa732b91f` |
| `packages/stackpress/src/plugin.ts` (aggregate, root-relative) | `0c3da5b8a8a6ac91c737fb9fad7d81da03d3e2d46ffb9dd57a9ebcc7736d3ee8` |

Additional read anchors: template blog common.ts menu 52–61/API66–143/population280–285,419–424; website store/populate20–25; Session.ts123–179,193–228 and events/session6–10; Ingest Router382–396; lib data/Nest117–149; SQL events/index18–29,126–128/detail/create and store/select515–548. Current source was used rather than interpreting schema descriptions alone as enforcement. No source API tests directory exists; manifest test glob was not treated as passing tests.

| Additional path relative to source family | SHA-256 |
| --- | --- |
| stackpress `templates/blog/config/common.ts` | `46f41f5c98e70d6ab0447b45452b4fd16247b1a5a19b4079ab48a6f926b599f2` |
| stackpress `templates/website/plugins/store/populate.ts` | `958723196cad3644194030a11c8460a775e535b7cdb05f162acdc89645423149` |
| stackpress `packages/stackpress-session/src/session/Session.ts` | `b6d71977591e879be76a69178321326566278f104b49ec2d550dd60d4b94d5f4` |
| stackpress `packages/stackpress-session/src/session/events/session.ts` | `5c9605d097f68c295dae8094d9a76064bcdf365db2ba47e6c666471f5ca1c5cb` |
| ingest `ingest/src/Router.ts` | `d285c2ea1d0ba292305bf807696c33683592a097ae2506a20e1045c73b8766e9` |
| lib `src/data/Nest.ts` | `edd5e306006a30b98ba66c34bea964bd6a3d38465c5ba2a46b0b2416b8dfc6bb` |

## Explicit revision dispositions

The complete document replaces A10-01..09 open recommendations with Decided under G43. Removed defaults: mandatory verified credential on GET, all-required/registry scopes, exposure allowlist, opaque Caller/grant/code model, PKCE/atomic code consumption, registered redirects/new TTLs/CSRF/can gates, extra metadata rejection/redaction, standardized token errors/headers, new CORS rules, HTTP egress restriction and webhook outbox/worker/signing/retry/dedup. Their source behavior now appears in parity rows. No source defect is asserted as a default fix.

Removed S-API-SQL/S-API-Admin deferrals: complete SQL adapter/schema/defaults/events and original generic app/key management are M1/M5 now. Removed API trusted-caller slot/SQL-policy blockers: original profileId forwarding uses Request.data; JWT me remains Slice 9-owned. Kept unaffected provenance, source inventory/export coverage, G39–G42, function/direct DX, target ownership, no-listener/build rules, review rounds, watchdogs, coverage and separate owner visual acceptance. Old proof records remain unchanged as historical scope, not current acceptance criteria.

## Probe records and validation

Shared store `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`; all unique records appended only through the required recorder. No old record edits.

| Record ID | Result / command wall / observed Cargo stage |
| --- | --- |
| `20261030T014018.465187Z-3cd7a71ccbdc` | G43 original-source VM oracle: 23 passed, 0 failed; 0.467702 s / unavailable. |
| `20261030T014045.678283Z-f8e4000c522c` | G43 approved primitive/fetch mechanics: 4 passed, 0 failed, 0 doctests; 30.612120 s / 29.87 s. |
| `20261029T164447.844995Z-5a187b0f5d05` | Prior dependency fetch: passed, 6.602174 s / unavailable. |
| `20261029T164617.624903Z-9729cd05122e` | Prior policy probe: failed 10/11 (literal-origin oracle), 61.740703 s / 60 s. |
| `20261029T164750.120172Z-056a57363c28` | Prior corrected policy probe: 12/12 passed, 2.297463 s / 1.61 s; recommendations superseded by G43. |

Revision contributes 2 attempts, both passed, 1 emitting compilation, 0 repository/workspace rebuilds. Accumulated revision command time 31.079822 s; Cargo stage subtotal 29.87 s overlaps it, not additive. Overall Slice 10 probe contribution is 5 attempts: 4 passed/1 failed, 3 emitting compilation, 0 repository rebuilds; shared counters can change in other lanes. Native Cargo/Rust 1.96.0, sccache, jobs2, no installed tools. Final observed external target size 220 MiB (no continuous peak sampling), below 8 GiB.

G43 probe SHA-256: manifest `b549302c4e0c2021f43371c1e65656b271f1f19a8738aeadd504ea27082d0a16`; external lock `b40c9573fd01b9b03dcd3353a2385b10121c88442021b073b2d0c9e582c50932`; Rust tests `a2e03c969acb24184d5eea1a8143211be92629bf3da8f461b1f8e2d3ff259442`; source oracle `148f4a2b84a2e5eae5ba3c4177d598d4b8a5cb4dab46f0a6f0261b01a3ef92f0`; runner `95182446d29c5046ae58c2ab1c3467a5d233cd0cd984c0ce73ab21124e23fa00`. Probe code/dependencies stay in /private/tmp and are not archived into .agents.

Revision baseline validator exit1: 146 errors / 27 warnings, saved `/private/tmp/slice10-kb-g43-before.txt`. Prior pass had the same baseline; existing sibling links/long historical records are not repaired here. Final validator exit1 is unchanged: 146 errors / 27 warnings, no added/removed diagnostics, saved at `/private/tmp/slice10-kb-g43-after.txt`. Mechanical checks passed for all nine topics: one H1, tagged balanced fences, whitespace, local links/anchors, complete export/type inventory, 26 risk IDs and six UI controls. Complete revised draft/old-to-new disposition was assembled externally before the in-place lossless split; all nine final bodies reassemble byte-for-byte. No historical task/status/context promotion. Git scope remains exactly the nine design files, no tracked production diff.

Not verified: production API/SQL/generated management, exact CUID provider/shared-reference implementation, actual JWT integration, browser/installed/remote fetch/TLS/proxy behavior, live backend acceptance, workspace builds or coverage. Next is coordinator review/reconciliation of only remaining mechanics questions, then explicit M0 release. Stop at this revised design checkpoint.
