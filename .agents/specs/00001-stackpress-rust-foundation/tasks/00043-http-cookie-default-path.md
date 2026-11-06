# 00043 — Native HTTP cookie defaults

Implemented; awaiting coordinator verification. Started from the authorized `git merge --ff-only main`, HEAD `9c5f33c`. No commit.

## Contract and audit

Read-only Ingest commit `477ec47f8b5dafa5713b13f9c964d06c61c22938`: `ingest/src/http/Adapter.ts:122–124,161–184` and `ingest/src/whatwg/Adapter.ts:127,203–218`. The host default is **Path=/ only**. Host configuration replaces that object; explicit revision options replace the entire host object, including an empty object. No default HttpOnly, SameSite, Secure or lifetime is added.

`HttpOptions.cookie: stackpress_lib::CookieOptions` exposes that policy. Buffered and exclusive streamed output share its selection/serialization. Removal selects the same scope, retaining the existing Rust Max-Age=0 / epoch tombstone. Core Session serialization and manually encoded Set-Cookie headers are unchanged. Existing workspace HttpOptions literals compile unchanged through Default/spread; external exhaustive literals must supply the new field.

| Audited writer | Result |
| --- | --- |
| Session signin, Session::save, signout | Revisions without options now use the host scope. Source signout's literal `session` key remains unchanged. |
| Session account/challenge and API source-admin flash | Same shared revision path; no page/event changes needed. |
| CSRF generate / clear | Same shared revision path; successful native auth clears CSRF at Path=/. |
| Aggregate Language::save and locale middleware | Same shared revision path; no language changes needed. |
| framework-view preference locale/theme and flash adapters | Their encoded headers already use CookieScope's root/mount path for writes and clearing; preserved exactly. Initial-theme bootstrap performs no storage write. |
| Native Hyper host / WHATWG | Hyper delegates to StreamingHttpAdapter. No Rust WHATWG/serverless output adapter exists; the upstream WHATWG policy was checked for parity. |

Maintained native tests drive HttpServer::serve_io over bounded duplex IO: real SQLite signup/signin, verified ADMIN JWT at /admin/check, explicit host/revision options, and signout. Python's installed standard-library CookieJar selects the actual outgoing Cookie header without opening a URL. Its negative control removes Path and reproduces the old /auth/signin scope. Replay after signout sends no session and the real test route returns 401. Stream tests also cover genuine exclusive-body output.

| Rust-only expectation | Before → after / source anchor |
| --- | --- |
| server/tests/http_output.rs, cookies_and_multivalue_headers_are_never_folded | `first=one`, `second=two` → `first=one; Path=/`, `second=two; Path=/`; Ingest HTTP Adapter.ts:169–184. Manual headers and explicit /scope tombstone expectations unchanged. |

The first new native fixture attempt requested a GET without a page renderer; it was removed from the POST-process proof. Its test-owned admin handler also needed results before status so results() would not reset 401. No production workaround or existing expectation change followed. Failed edit records: `20261106T105153.485445Z-1e2909c1e70b`, `20261106T105223.517002Z-009a14ec76e3`, `20261106T105258.395561Z-231aeace2fd9`; corrected native suite: `20261106T105323.924564Z-b72575120238`.

## Three review rounds

| Round | Finding / verification |
| --- | --- |
| Complexity | One exhaustive revision match selects options and preserves list-codec failure / deletion lifetime. No further change justified; focused server cookie tests: 4 passed, `20261106T105137.971304Z-dddfc58cae5e`. |
| Responsibility / abstraction | Host fallback stays in HTTP, reusing the foundation serializer; no lib mutation, global state, lock or new dependency. Factored the native test fixture for separate configured-key and default-key flows. Native tests: 3 passed, record above. |
| Consistency / Rustdoc / comments | Documented whole-object precedence, constructor validation and API-guide behavior; aligned buffered/streamed paths. No further finding. Final focused round close: 8 passed, `20261106T105419.497488Z-f9257dbf9ea3`. |

## Final scoped gate

Canonical benchmark recorder; warm target/g50-m0, sccache, jobs2, debug0/incremental0. Cargo locked/offline, TRYBUILD unset; nextest threads2, tcp_watchdog excluded. Browser/WASM and coverage ran after nextest. All implementation/test/guide files precede the final gate start. Exact argv, feature selections and separate command/Cargo timings: `/private/tmp/task43/evidence.json`, sourced from the live benchmark ledger.

| Check | Result / benchmark record |
| --- | --- |
| fmt | passed; `20261106T105455.356879Z-04b1ce78b527` |
| Workspace Clippy default / compatible, -D warnings | passed; `20261106T105501.923894Z-80d77cd49368` / `20261106T105547.126149Z-c5cad102dde1` |
| Server, server-http, session, CSRF nextest default / compatible | 86 passed/8 skipped; 149 passed/11 skipped; `20261106T105636.499649Z-810249c4fc1a` / `20261106T105703.196375Z-8102de42a76b` |
| Server/session doctests | 87 passed; `20261106T105904.685359Z-c052fbea6283` |
| Workspace compatible Rustdoc, RUSTDOCFLAGS=-D warnings | passed; `20261106T105914.676346Z-41cd64835b73` |
| Eight aggregate consumers, including build-host --check | passed; `20261106T110224.683258Z-a4599791d88a` |
| Session direct / facade / plugin / disabled consumers | passed; `20261106T110447.784402Z-057381704bcb` / `20261106T110525.250081Z-f9c9e9e4b60c` / `20261106T110719.585428Z-b60412c6c2a2` / `20261106T110601.665203Z-a1e6680e0279` |
| Fresh SSR, hydrate/CSR and bindgen | passed; six build/page records in evidence.json |
| Original session hydrate / CSR browser matrices | 176 each + one no-JS check each; `20261106T111113.188678Z-95a7f3d952fe` / `20261106T111418.035984Z-9d95e5169e9f` |
| Native coverage: server HTTP/stream; session auth-server/SQL | server 2313/2504 = 92.37%; session 2808/3033 = 92.58%; `20261106T111819.128494Z-5312aa06e6bb` |
| Independent floors, exact scope, lock and resources | passed; `20261106T112138.471093Z-ebf2d2ecdea4` |
| Final KB and nine-file receipt scope | canonical 0 errors / 42 existing warnings; raw worktree 146 sibling-path errors / 42 warnings; `20261106T112325.015619Z-079167bbc98c` |

All 699 registry tuples and **all root/consumer lock bytes** are unchanged. No manifest, oracle, golden or .stderr change; nothing staged. No network, install or listener. Browser runs assert the approved cached Playwright path, launch its own headless Chromium with fresh contexts, and intercept requests. The cookie-jar proof covers path/expiry, without claiming browser SameSite enforcement. Live DB variables remained unset.

Only completed owned coverage/trybuild and consumer target outputs were removed; root debug dependency artifacts stayed warm. Measured target: 20.33 GiB before cleanup, 20.49 GiB at close, below guard23.4/cap24; final disk free141.30 GB. Cleanup records and exact commands remain in the ledger.

## Exact changed files

```text
A .agents/specs/00001-stackpress-rust-foundation/tasks/00043-http-cookie-default-path.md
M crates/framework/framework-session/tests/main/native_http.rs
A crates/framework/framework-session/tests/main/cookie_jar.py
M crates/server/server/docs/api/http.md
M crates/server/server/src/http.rs
M crates/server/server/src/http/common.rs
M crates/server/server/src/http/stream.rs
M crates/server/server/tests/http_output.rs
M crates/server/server/tests/http_stream.rs
```

## Coordinator verification

Committed as `037c60c` and merged into main as `5ae3ec9`. I reviewed the additive `HttpOptions.cookie` default (`Path=/`), explicit options replacing it, and the sign-out scope. Post-merge gate on a detached `5ae3ec9` (`/private/tmp/cookie-verify/run.sh`, 2026-11-06), exit 0:
- 699 tuples identical;
- fmt; Clippy, default and compatible;
- nextest: default 1,077 passed; compatible 1,244 passed; `tcp_watchdog` alone;
- doctests; Rustdoc with `-D warnings`;
- aggregate/session consumers and build-host `--check`;
- all freshness checks;
- wasm32;
- docs and ledger;
- the source-roles and editor browser proofs.
