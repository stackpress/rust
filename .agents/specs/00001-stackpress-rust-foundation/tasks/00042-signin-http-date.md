# Task 00042 — sign-in Dates at native HTTP output

Status: implemented and locally verified; ready for coordinator review. Base: `git merge --ff-only main` → 2f46c97. Owner approval: [G57](../decisions.md). No commit or listener.

## Behavior and source audit

Original references, read-only: Stackpress a71d683051ba8350fdd12d6b5a33f268fdcc285f; Ingest 477ec47f8b5dafa5713b13f9c964d06c61c22938; lib e2cc6b8cad77b5ad18be8a94e0d40e72e2f6183a.

| Decision / original anchor | Before → after |
| --- | --- |
| G57; original lib src/router/Response.ts:313–318; Ingest src/http/Adapter.ts:189–230 and src/whatwg/Adapter.ts:176–194 | Valid sign-in retained rich results, but HTTP failed at results.consumed → HTTP preserves the 302, Location, cookie and JSON body, converting nested Dates to JavaScript UTC ISO text with millisecond precision and Z. Redirect never removes results upstream. |

The shared server HTTP converter projects a separate envelope copy. Core Value/Response serialization remains strict, and downstream events retain native Dates. Bytes, files and SharedNative still fail with qualified paths; projected strings must fit existing response limits. Text/bytes bodies, stream ownership, HEAD, forbidden-body statuses and lifecycle order are unchanged. No new guard or route policy. Dates outside JavaScript's representable range fail explicitly.

| Pages/events inspected | Result |
| --- | --- |
| auth-signup / signup page | Rich Profile/Auth dates survive success/redirect; covered by actual native HTTP signup. |
| auth-signin / username, phone and email-password | consumed and joined Profile dates reach the same HTTP boundary; username regression uses actual SQLite, registered processes, matching incoming CSRF, Hyper duplex IO and a verified session cookie. |
| magic-link, OTP, 2FA | Native consumed remains available for challenge computations; retained Auth records and final auth-signin results use the shared conversion. |
| signin chooser / signout | Stock claims/status-only redirects introduce no Date. |
| account profile, security, update, password, removal, TOTP and account SQL events | Native nested results/applied envelopes use the shared converter when HTTP selects JSON; explicit HTML/CSV output remains byte-preserving. |
| API Application/Session SQL events | Existing ResponseValues::Json source codec already writes ISO dates; no event/model change. |
| API OAuth / token | OAuth retains application-detail data through redirect; custom native results are covered centrally. Token already recursively projects its selected user fields, including created. |
| source-admin | Existing project_results uses explicit source codecs; any rich results retained by a redirect are handled at the shared HTTP boundary. |

No session/API production change was needed. The maintained HTTP tests pin pre-1970 dates, zero milliseconds, leap day, year zero, signed extended years, JavaScript range endpoints, submillisecond clipping, nested arrays/objects/errors, insertion order, retained redirect body, HEAD framing, budgets and refusal of every other native variant.

## Three review rounds

| Round | Finding / verification |
| --- | --- |
| Complexity | Gregorian-era arithmetic, negative epoch division and TimeClip bounds reviewed; no further refactor justified. JavaScript control verified the negative-year input; the initial new test timestamp was corrected, without changing its expected ISO value. |
| Responsibility / abstraction | One private HTTP-only projector; no framework-session dependency in server, global state, lock or await. Existing envelope serializer owns ordinary JSON and qualified native-value errors. No further extraction justified. |
| Consistency / comments | Updated the HTTP API guide, retained existing Rustdoc, and strengthened the regression to verify the returned JWT and persisted consumed value. Final narrow run: 3 passed, 20261106T085922.593713Z-ed5680bbbdb2. |

The first narrow attempts exposed only two new-test mistakes: a one-day-off negative-year input (Node control 20261106T085637.265284Z-9f768de883f8) and treating a service Result as Option. Both were corrected before the final gate. Existing expectations, Node goldens and .stderr snapshots are unchanged.

## Final scoped gate

Canonical benchmark recorder; warm target/g50-m0, sccache, jobs2, debug0/incremental0. Cargo locked/offline; nextest threads2 and tcp_watchdog excluded. Browser/WASM jobs run after nextest. No source edits after the final gate began.

| Check | Result / record |
| --- | --- |
| fmt | passed; 20261106T090047.391346Z-296c095b2eb8 |
| Workspace Clippy default / compatible, -D warnings | passed; 20261106T090056.906119Z-c209c57a51cc / 20261106T090217.797097Z-1f1ae6ea4958 |
| Server, native HTTP and session nextest default / compatible | 78 passed/8 skipped; 138 passed/11 skipped; 20261106T090401.958522Z-851f03067312 / 20261106T090547.143137Z-0ca646d6874e |
| Server/session doctests | 87 passed; 20261106T091201.512570Z-47c6cb8b52b1 |
| Workspace compatible Rustdoc, RUSTDOCFLAGS=-D warnings | passed; 20261106T091258.573568Z-19883d54a29e |
| Aggregate / session graphs | passed; 20261106T091545.124716Z-2a2190d2e434 / 20261106T092021.925973Z-bff65eefb6e2 |
| Eight aggregate consumers, including build-host --check | passed; 20261106T092228.602880Z-cade2b6005f1 |
| Session consumers: direct / facade / plugin / disabled | all passed; 20261106T094340.093349Z-395e03587ed5 / 20261106T094435.896735Z-72dde6c372af / 20261106T094528.703999Z-4f0f0bfa4206 / 20261106T094853.461284Z-1e0bccd8284f |
| Fresh SSR, hydrate/CSR builds and bindgen | passed; exact command/build records in /private/tmp/task42/evidence.json |
| Original session hydrate / CSR matrices | 176 each, plus one no-JS check each; 20261106T095152.457420Z-e3cac4ab5889 / 20261106T095506.550232Z-14a5915ecedd |
| Native coverage: server HTTP/stream; session auth-server/SQL | server 2280/2470 = 92.31%; session 2808/3033 = 92.58%; new HTTP projector 58/58 = 100%; 20261106T095906.670529Z-6048f0452d6c |
| Per-crate threshold, exact scope, lock and resources assertions | passed; 20261106T100348.789269Z-935aaf2dd1a9 |
| Final KB validation | canonical 0 errors / 42 warnings; raw worktree 146 sibling-path errors / 42 warnings; 20261106T100511.026287Z-33dc644c7e34 |

Approved lock delta: only the framework-session → stackpress-server-http dev-dependency edge. All 699 registry tuples and registry package blocks are byte-identical (immediate assertion 20261106T085248.568100Z-c34320b46da7; closing assertion above). No existing Rustdoc, root manifest, consumer lock, oracle or snapshot change. All changed source/manifests precede the final gate start; nothing staged.

Standalone consumer targets were rebuilt separately with shared sccache. Browser runs assert the exact approved Playwright path, use its own headless Chromium and fresh contexts, and intercept every request. No system Chrome or listener. Native HTTP regression uses finite joined duplex futures; no detached test server task. Live MySQL/PostgreSQL tests were not selected; opt-in variables stayed unset.

Only completed owned coverage, trybuild, consumer and WASM output was cleaned; root debug dependency artifacts stayed warm. Final target 19.77 GiB (<23.4 GiB guard / 24 GiB cap); disk 140.28 GB free. Browser artifacts and command/build timings remain in /private/tmp/task42.

## Exact changed files

```text
A .agents/specs/00001-stackpress-rust-foundation/tasks/00042-signin-http-date.md
M Cargo.lock
M crates/framework/framework-session/Cargo.toml
M crates/framework/framework-session/tests/main.rs
A crates/framework/framework-session/tests/main/native_http.rs
M crates/server/server/docs/api/http.md
M crates/server/server/src/http.rs
M crates/server/server/src/http/common.rs
A crates/server/server/src/http/json.rs
M crates/server/server/tests/http_output.rs
```

## Coordinator verification

Committed as `710ccfc`, then merged into main as `9c5f33c`. Reviewed:
- the HTTP-only `json::envelope` projection: Date → JS `toISOString` form, with the lib kept strict;
- the session test-only `stackpress-server-http` edge, a workspace edge only.

Post-merge gate on a detached `9c5f33c` (`/private/tmp/g57-verify/run.sh`, 2026-11-06), exit 0:
- 699 tuples identical;
- fmt; Clippy, default and compatible;
- nextest: default 1,073 passed; compatible 1,240 passed; `tcp_watchdog` alone;
- doctests; Rustdoc with `-D warnings`;
- aggregate/session consumers and build-host `--check`;
- freshness for generated-workspace/admin, models, API checkpoint and composite fixtures;
- wasm32 admin/leptos;
- docs and ledger;
- the source-roles and editor browser proofs.
