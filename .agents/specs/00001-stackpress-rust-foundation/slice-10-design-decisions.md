# Slice 10 API design checkpoint

Status: Draft / Not Frozen; revised under binding G43, 2026-10-30 Asia/Manila. Design only for [task 00021](tasks/00021-slice-10-api.md). Rust worktree remains at `11e98b102c8c6931dc750e476be7057448cd2acf`; G43 was read from `/Users/cblanquera/server/projects/stackpress/rust/.agents/specs/00001-stackpress-rust-foundation/decisions.md`, main checkout `3cf407f`. No production code, repository build, manifest/lock change, listener, commit or other-agent message is released here.

## Reading paths

- [Upstream inventory and intent](slice-10-design-inventory.md): load for every export/type/model/page and the rationale behind its source behavior.
- [Configuration and endpoints](slice-10-design-configuration.md): load for every config key, missing-config deactivation, request precedence, scopes, CORS and chain behavior.
- [Overlap and session interface](slice-10-design-boundaries.md): load before reconciling Slice 9; records exactly what API passes downstream and who owns any request-local mechanism.
- [Public API and DX](slice-10-design-api.md): load for ordinary plugin registration, direct calls, SQL adapter and management composition.
- [OAuth, SQL, management and webhooks](slice-10-design-execution.md): load for full-scope persistence/page/delivery contracts and UI acceptance proposals.
- [Dependencies and disposable probes](slice-10-design-assessment.md): load for approved pins, source-parity evidence, mechanics limitations and the remaining CUID dependency question.
- [Behavior tests and milestones](slice-10-design-verification.md): load for accepted developer responsibilities, parity tests, implementation risks, explicit file ownership, watchdogs and coverage.
- [Provenance and revision receipt](slice-10-design-provenance.md): load for source lines/hashes, old-to-new disposition, benchmark records and baseline-preserving document checks.

## Decided under G43

Every original line is presumed intentional. Plugins protect only the processes they are configured to protect; absent config disables their effects. API does not insert a global permission system, default-deny gate, exposure allowlist, credential hardening or new lifetime. Each selected route/event decides its own process. G42 remains binding: priority then FIFO, multiple handlers per route/event, cooperative Abort, no framework ownership/arbitration.

| Earlier question | Binding disposition |
| --- | --- |
| A10-01 credentials | Port `authorize`: GET needs ID; other methods need secret presence, and app/session handlers compare it. Do not replace with opaque grants or standard-scheme-only parsing. |
| A10-02 scopes | Any-match with endpoint scopes; absent/empty permits yields 401 for app/session. Registry labels do not restrict stored/requested scopes. |
| A10-03 OAuth | Keep API Session ID as code and access_token, access_secret output, original token-page checks, state/redirect handling and `api.expires` calculation. No PKCE, one-use code store or new expiry defaults. |
| A10-04 storage | SQL adapter, Application/API Session models, generated operations and original administration/key management are all in Slice 10 now. No S-API-SQL/S-API-Admin deferral. |
| A10-05 activation | Missing `api` disables the plugin. OAuth routes/view mount only when endpoints are a nonempty array. Webhook registration remains independently in Listen after API activation. |
| A10-06 delivery | Await one inline fetch of the source envelope after status 200 and validData. No outbox, worker, signature, delivery-ID, retry, deduplication or commit attestation. |
| A10-07 destinations | Use configured URI/method and fetch behavior, including HTTP/private destinations and redirect behavior. Add no API egress restriction. |
| A10-08 dependencies | Approved reuse: ring, subtle, zeroize, base64, url, reqwest, pinned/minimal features. Share ring with Slice 9 for HMAC/randomness. Withdraw hmac/sha2/getrandom/oxide-auth/tower CORS requests. Other required dependencies remain explicit questions. |
| A10-09 pages/management | Consent page, token page handler, SQL adapter and generated app/session/key administration all land now. No invented separate key vault/rotation console is substituted for the source. |

## Short source-parity table

`A` means canonical `packages/stackpress-api/src/`; exact checkout and hashes are in provenance. Rationale beyond explicit comments is labeled interpretation in the inventory, not an asserted author quotation.

| Behavior | Upstream source line | Rust mapping | Mechanics deviation |
| --- | --- | --- | --- |
| Config disables plugin | A `plugin.ts:19` | Optional typed `api` section; return without contributions when absent | Typed decode of present data |
| ID reads / secret writes | A `helpers.ts:14–33`; `plugin.ts:120–124,156–160` | Source split/trim parser and exact comparison branches | Result/Option in place of JS false/undefined |
| Alternative scopes | A `plugin.ts:127–129,163–165` | `iter().any`; empty intersection denies 401 | None |
| Fixed data then profileId | A `plugin.ts:132–134,168–169,180–181` | Top-level set overlay; session writes profileId; raw nested emit | Nest updates; no caller slot |
| CORS and nonterminating OPTIONS | A `plugin.ts:66–101` | Same URL-origin comparison, headers/status and priorities | Compiled route callbacks |
| Endpoints gate OAuth | A `plugin.ts:48–51` | ALL token/OAuth page handlers; view priority -100 | Static function/page bindings replace import paths |
| OAuth obtains JWT-backed browser user | A `pages/oauth.ts:49,59,70–75` | Resolve `me` with original request, application-detail with fresh `{id}` + shared response, session-create with copied data | Explicit Rust fresh-request/status helper |
| Session code/token exchange | A `pages/token.ts:21–49` | Same persisted row lookup, app ID/expiry checks and result payload | Typed Date milliseconds and row DTO |
| Inline webhook | A `plugin.ts:24–41`; `helpers.ts:45–58` | Same predicate/envelope, await fetch, priority -200, ignore HTTP status | Send future + reqwest adapter |
| Keys are model secrets | `schema.idea:58–63,164–169`; blog config `common.ts:52–61,280–285` | Ordinary stored/displayed secret defaults and generated Application/Session administration | Explicit SQL adapter and Leptos controls |

G39/G40 presentation behavior remains unchanged (Light/Dark; no persistent System mode); G41 uses stackpress-leptos and library-owned rendering rules. No API-specific theme, router, TLS host or second dispatcher is introduced. Owner-approved source behavior is not relabeled a Rust mechanics necessity.

## Questions for the owner

No source defect is asserted and no security-policy choice is reopened. Two mechanics questions remain; the source behavior stays the default while they are reconciled.

| ID | Exact source and remaining question | Recommendation |
| --- | --- | --- |
| Q10-CUID | `schema.idea:11,60,123,166` uses cuid(); schema generator `transform/column/index.ts:130–136` imports @paralleldrive/cuid2. Ring does not implement its SHA3-512 step. Should the bundled native provider add `sha3 = 0.10.8`, or receive a source-compatible CUID provider through existing ValueProviders? | Approve minimal SHA3 support for a source-compatible CUID2 provider using ring randomness, shared by both slices; otherwise require the explicit provider and report its absence. Do not replace cuid() with random base64 or UUID. No extra crate is approved by this proposal. |
| Q10-REF | `helpers.ts:50` uses JS includes reference identity for object/array elements. How should direct Rust callers represent deliberately shared container references across validity/results? | Parsed independent JSON containers compare unequal, exactly as separately created JS objects; if shared-reference use must be available directly, use an explicit Arc-backed input carrier and pointer identity. Do not substitute structural object equality or reject complex arrays as hardening. |

The coordinator reconciles the session-owned JWT service/event names and shared Profile/schema provider ownership using the boundary document. The API does not need the earlier trusted-caller bridge or an identity slot to forward profileId. Implementation remains unreleased until the revised checkpoint is reviewed.
