# Slice 9 Rust overlap and ownership under G43

Status: Revised design only. Current Rust source remains unchanged. Source parity replaces the prior signed-reference/revocation/mandatory-protection plan; [G43 decisions](slice-9-design-decisions.md) and the [single API interface](slice-9-design-api.md) govern.

## Move, reuse or keep

| Existing Rust surface | Disposition / reason | Integration under original behaviour |
| --- | --- | --- |
| crates/lib/src/session.rs: Session, SessionValue, SessionRevision, CookieOptions, SameSite/Priority, cookie codec | Keep and reuse; low-level cookie data is distinct from JWT identity | framework-session reads configured normalized cookie and stages original set/delete operations. Do not add crypto, account policy or a revocation repository to lib. |
| Lib cookie options/revision/tombstone handling | Keep | Existing native response/global cookie behaviour stays; framework-session/CSRF do not impose __Host names, mandatory flags or new scope rules. Signout retains literal session deletion until owner changes source. |
| Lib Request.session/Headers parsing | Keep | Follow source normalized session/aggregate data semantics. No new duplicate-cookie/form rejection or Authorization fallback in Session.token. |
| Lib EventPattern/Method/MethodPattern | Keep existing router behaviour; assess reuse only where equivalent | Session permission helpers have their own source derived-regex/case/method/trailing-slash semantics. Do not replace them with a different escaped-literal matcher merely for reuse. Existing backend limitations are mechanics gates. |
| framework-view/src/access.rs: ActionId, AccessDenied, AllowedActions, HostPolicy, HostAccess | Keep unchanged | Its already explicit host policy is not the session plugin's default policy. Session missing/empty access remains permissive as source; no new automatic use of HostAccess on all routes. |
| framework-view/src/policy.rs: PublicLocation/BasePath/PageContract/status/assets/limits | Keep accepted contracts | Reuse page envelope/render mechanics where source data fits. User redirect_uri must not be silently narrowed to app-relative PublicLocation; preserve source redirects through appropriate native response API. G41 escaping/location rules elsewhere remain intact. |
| framework-view/preferences pure/action/locale/native settings/state | Keep G39/G40 | Locale, Light/Dark/unset and legacy system read behaviour are unrelated to identity, and are neither moved nor reset on signin/signout. |
| preferences/host.rs synchronous Admission(fn(&Request)->bool) | Keep app-owned signature | Optional explicit native caller slot can supply facts if an app chooses, but no mandatory cookie/CSRF/Origin policy is added by the session plugin. Existing preference origin/admission policy itself is not weakened. |
| framework-view/flash.rs and flash/host.rs | Keep | Source auth pages use literal flash JSON shape/type/message/close cookie; preserve that wire/state behaviour through an auth compatibility adapter. Do not silently replace it with a different bounded FlashNotice/id cookie merely for consolidation. Existing presentation flash remains independent. |
| framework-admin/src/pages.rs Snapshot.csrf / Command.csrf | Keep public token DTOs | Can carry source-generated CSRF strings. No default identity integration or signed/bound CSRF protocol is introduced. |
| examples/generated-admin/web host Config.csrf, x-admin-csrf, Origin guard, FixturePolicy | Keep labelled fixture | Source port does not rewrite unrelated finite admin policy. An app may opt into new Session/CSRF mechanisms; do not relabel the current markers as production authentication. |
| generated-admin host/transfer.rs | Keep | Existing import/confirmation fixture checks and generated operation policy remain separately configured; no change in this design. |
| framework-sql operations::OperationPolicy(&Request) | Keep existing synchronous API | A native caller may be explicitly filled before delegation and read through a coordinator-released generic Request slot. No global default-deny, mandatory actor or async-trait signature rewrite. |
| examples/review-preview gateway/setup | Keep | Existing review-local markers and UI proof are regression fixtures, not evidence of the full auth implementation or a reason to start a listener. |
| examples/ui-preferences web policy/lib | Keep | Header caller/CSRF fixture and explicit sessionless Admission remain explicit app choices; no session cookie or missing-config failure is inferred. |
| PageSnapshot<T>/SSR/hydrate/CSR | Reuse envelope and Leptos rendering mechanics | Carry source-intended me token/CSRF/enrollment secret/QR/form data where its page does. Test that native keys/resources are absent without deleting intentional source data under a blanket no-secrets rule. |
| Existing generated-admin Profile and metadata revision 3 | Keep separate | Built-in auth/profile schema and stores are implemented now with reused schema/SQL capabilities, not by mutating an unrelated demo model or frozen shared contract. |

No existing production file moves. New source behaviour belongs to framework-session/framework-csrf/framework-email; a narrow generic request-local carrier is the only proposed shared change and is not authorized yet.

## Coordinator-owned native carrier

Current Request has no native typed extension slot; server's immutable adapter_context is not directly available to OperationPolicy, whose signature has only &Request. The proposed generic Any+Send+Sync Arc type map belongs to **stackpress-lib** (`crates/lib/src/request.rs` and a focused module), with no framework dependencies. The [API interface section](slice-9-design-api.md) is the sole normative contract for filling/reading/forwarding it.

Coordinator must release the exact files and ownership serially before either Slice 9 or API lane writes them. Slot absence does not itself allow or deny anything; application policy decides. Preserve same-request emit and explicit selected transfer to newly constructed nested requests; no automatic root-global identity. JWT Session loading requires no SQL lookup, so the previous async-revocation mismatch is gone; API credential lookup can finish before filling its own native facts.

## Proposed package/feature boundaries

| Package/feature | Responsibility / dependencies |
| --- | --- |
| stackpress-framework-session default [] | Shared source-shaped types/helpers that need no host resources; no compile-time deployment seed/SMTP/SQL state in browser artifacts. |
| session native/auth/sql | JWT Session and source auth/profile actions/stores/schema/SQL adapter, helpers and all configured flows now. Reuse lib/framework-schema/framework-sql/SQL Engine and approved minimal native libraries. No session-record/epoch store. |
| session server | Ordinary composite auth→email→session registration, optional typed config gates and exact event/route/view contributions. No runtime code import. |
| session Leptos render features | Every source form/body/head/page/helper now; shared UI/source data with native page-loader/store modules separately target-gated. No new auth.methods config or partial page scope. |
| stackpress-framework-csrf shared/native/server | CsrfConfig/CsrfPlugin equivalent and pure public field data; native random/generate/clear/valid and Config registration. No session/view dependency needed by the mechanism. |
| stackpress-framework-email shared/native/server and transport features | Source options/result contracts, per-call transport factories, all source variants now. SMTP/pool/sendmail/JSON/stream/SES adapters enabled only when compiled/selected, with minimal dependencies; no transport bans as defaults. |
| Root stackpress facade | Explicit optional re-exports/features with unchanged defaults, one integration writer after coordinator release. A feature makes code available; source config gates still decide contributions. |

Native secrets/DB/provider resources must not be baked into a browser dependency graph; source runtime values intentionally serialized by a page are a separate parity obligation. Default/hydrate/CSR/SSR consumers and normal-edge trees prove the actual split after implementation. This design does not claim installed-app/provider acceptance.
