# Slice 10 dependencies and disposable evidence

Draft / Not Frozen; [G43](slice-10-design-decisions.md) approves the listed reused libraries and reverses the prior hardening/library requests. Source behavior determines library fit. Probes live outside the repository and prove only their declared seams.

## Approved library assessment

| Library / exact pin | Use and minimal feature proposal | Evidence / boundary |
| --- | --- | --- |
| ring 0.17.14 | Native HMAC-SHA256/randomness shared with Slice 9; default-features=false, std | New external probe executes RFC4231 vector/verification and SystemRandom. Ring is not used to add OAuth grant machinery or webhook signatures absent from source. Its own internal getrandom dependency is transitive mechanics, not a direct new request. |
| subtle 2.6.1 | Reuse only where existing session/comparison code needs it; default-features=false, std | Equal-slice primitive exercised. Do not change source accepted credentials/equality conditions under a constant-time policy label. |
| zeroize 1.9.1 | Reuse local temporary buffer lifecycle; default-features=false, alloc | Local buffer clears in probe; no claim all copies disappear and no new omission of intentionally displayed/stored secrets. |
| base64 0.22.1 | Existing JWT/encoding support; default-features=false, alloc | Encoding probe; API ID:secret parser is not changed to Basic/base64 just because this library exists. |
| url 2.5.8 | Existing URL and form-query encoding facilities | Query state vector reproduced; source redirect split behavior is retained rather than normalized into a new registered-URI policy. |
| reqwest 0.13.4 | Native Fetch adapter; default-features=false with rustls for source HTTPS capability | Construction test permits configured HTTP localhost and source envelope, no send. Adjust redirect limit to 20 for fetch mechanics; no custom restrictions/deadlines. TLS transitive provider requirements are library mechanics, not a replacement for shared ring HMAC/randomness. |
| Existing lib/server/SQL/schema/admin/view/Leptos/serde/serde_json/Tokio | Reuse current package versions and target features; no new dispatcher/runtime | Existing sources establish their ownership. Typed sections, Send adapters, generated SQL and Leptos are required mechanics adaptations, not a reason to alter authorization. |

Withdraw the prior direct hmac/sha2/getrandom requests and production/test adoption requests for oxide-auth, oauth2 and tower-http CORS. The custom source OAuth/token flow is not an OAuth-server SDK port; existing CORS header logic is small and must retain its exact semantics. Previously downloaded probe crates stay outside the repository; they are not added to production.

Remaining dependency question Q10-CUID: @paralleldrive/cuid2 uses SHA3-512 and base36 conversion; ring lacks SHA3. Request owner approval for minimal `sha3 = 0.10.8` if the packaged provider implements source-compatible CUID2 using ring randomness. Existing framework-schema ValueProviders can alternatively accept an app-supplied compatible implementation. No new cuid2/rand crate is approved or needed by default; inspected [cuid2 0.1.6 source](https://docs.rs/cuid2/0.1.6/src/cuid2/lib.rs.html) uses its own rand/SHA3 stack, so it is not silently selected under the shared-ring instruction. [sha3 0.10.8 docs](https://docs.rs/sha3/0.10.8/sha3/) establish the candidate's SHA3-512 support; no download/build/audit of that candidate occurred.

Fetch mechanics are checked against the primary [Fetch Standard redirect algorithm](https://fetch.spec.whatwg.org/#http-redirect-fetch), including its 20-redirect limit and method/body changes. The Request constructor rejects GET/HEAD with a body; reproducing that existing fetch error is transport mechanics, not a new API method whitelist. The construction-only reqwest probe does not establish these network behaviors by itself.

## G43 source oracle

`/private/tmp/slice10-g43-probes/source-parity.cjs` reads original API helpers/plugin/OAuth/token TypeScript files from the canonical checkout and transpiles them with the already installed upstream TypeScript compiler into disposable VM modules. Imports are supplied as fake action/Exception/isObject services; host routes/config/model events/fetch are fakes. Original function bodies execute, not rewritten policy sketches. There is no listener, database or real fetch. The harness does not prove the full source project or actual upstream Response/model implementation; those remain integration obligations.

Final oracle: 23 passed / 0 failed. Cases:

1. Header scheme ignored; GET ID-only accepted.
2. HEAD/non-GET requires secret presence.
3. Extra colon components ignored by the parser.
4. App GET skips secret/active/expiry checks in the endpoint and accepts any matching scope returned by its detail event.
5. Empty endpoint scopes denies with 401.
6. Non-GET compares API Session secret exactly.
7. Session fixed data then profileId writes the original request.
8. Nested emit false is not returned by the endpoint wrapper.
9. Missing api config contributes nothing.
10. Empty endpoints still registers webhook listener but no OAuth routes.
11. Nonempty endpoints mounts ALL OAuth/token routes and view -100.
12. CORS compares URL origin and emits the source header names/values.
13. Preflight sets 200 without Abort even when no origin matches.
14. validData object subset/list membership/missing fields.
15. Complex array entries compare by shared-reference identity.
16. OAuth input spread preserves supplied model flags/secrets/scopes; overrides app/profile/expiry and echoes state.
17. expires zero remains zero, not source omission default.
18. OAuth response retains application secret and selected config defaults.
19. OAuth post-create branch reads current response code, not fresh result code.
20. Token exchange requires secret presence, never compares it, and repeats with same code.
21. Token expiry equality passes and strictly earlier expiry denies.
22. Webhook sends unfiltered source envelope to configured HTTP localhost.
23. Non-200 skips; resolved HTTP500 is ignored; fetch rejection propagates once without retry.

## G43 approved primitive probe

External `/private/tmp/slice10-g43-probes/Cargo.toml` pins ring/subtle/zeroize/base64/url/reqwest exactly as above, plus existing tokio 1.54.1 for async construction tests. Separate workspace/lock/target. Four tests pass: ring HMAC known vector/verify/tamper, SystemRandom + base64 + local buffer zeroization, form-query/state encoding, and reqwest HTTP/redirect-policy request construction without execution. No signature is added to the actual source webhook envelope.

Both proofs are recorded through the branch's benchmark recorder at the required shared store. Runner sets CARGO_TARGET_DIR outside the repository, incremental off, debug=0, jobs=2 and sccache, and has a 240-second process-tree watchdog with terminate/kill/reap. Cargo used --offline and only the external manifest; no dependency download was needed. External Cargo.lock was created and retained. No repository Cargo command or stale target was consulted.

```sh
python3 .agents/scripts/build-benchmark.py \
  --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks \
  run --label 'slice10 design probe G43 source parity' \
  --kind other --scope other --cache unknown \
  -- python3 /private/tmp/slice10-g43-probes/run.py source

python3 .agents/scripts/build-benchmark.py \
  --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks \
  run --label 'slice10 design probe G43 approved ring fetch mechanics' \
  --kind test --scope other --cache cold \
  -- python3 /private/tmp/slice10-g43-probes/run.py rust
```

## Prior probe disposition and evidence limits

Prior `/private/tmp/slice10-probes` and its recorded 12-case corrected run remain historical evidence, unchanged. PKCE/one-use-code/all-required-scope/standard CORS/queued-signing recommendations from that draft are superseded by G43 and are not acceptance requirements. Its initial literal-CORS oracle failure remains in its failed record; it is neither erased nor used to justify a source change.

No probe proves SQL adapter correctness, CUID2 compatibility, browser layout, generated management, full JWT/account behavior, remote delivery/TLS/proxy/redirect interaction or cross-platform acceptance. HMAC vector source is [RFC4231](https://www.rfc-editor.org/rfc/rfc4231.html); the prior OAuth security BCP/RFC7636 research is not authority over owner-selected source semantics. Production must be proved with the real contracts at milestones; no production coverage is claimed here.
