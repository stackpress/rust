# Slice 11 aggregate framework design checkpoint

Status: **Draft / Not Frozen; design only**, 2026-10-31 Asia/Manila, for [task 00023](tasks/00023-slice-11-aggregate.md). Started from confirmed clean af9cc01b9d9ab9ab73af5998ad8a8376bb6b20f7. Slice 9 docs are merged; that does not release Slice 11 implementation. No source, manifest, lock, facade, build or listener change is made here.

## Reading paths

- [Package and export inventory](slice-11-design-inventory.md): load for all upstream files, export/type subpaths, scripts, config, assets and backend dispositions.
- [Complete export ledger](slice-11-design-exports.md): load for exact named types/values, origins, aliases and source-only subpaths.
- [Rust overlap and features](slice-11-design-constituents.md): load before selecting registrars or changing manifests; existing symbols/features, missing bridges and facade gaps.
- [Composition and public API](slice-11-design-api.md): load for the proposed crate, compiled-client catalog, exact order, direct use and source activation.
- [Targets and configuration](slice-11-design-targets.md): load for native/SSR/hydrate/CSR/build-host selection and lock policy.
- [Risks and verification](slice-11-design-verification.md): load for prevention/parity tests, consumers, file ownership, milestones, watchdogs and coverage.
- [Provenance and checkpoint evidence](slice-11-design-provenance.md): load for complete source hashes/lines and canonical-layout KB checks.

## Recommendation

Use the accepted crates/framework/framework location, package stackpress-framework, opt-in facade stackpress::framework. Keep the root a facade; lower libraries never depend back on it. All constituent registrars and direct libraries remain independently usable. No plugin trait, dynamic loader, runtime enable/disable list, global service state, second dispatcher or route arbitration.

Proposed register(&mut ServerBuilder) -> lib::Result<()> supplies the stock aggregate without a generated application client. Proposed register_with(&mut ServerBuilder, &'static CompiledClient) supplies statically linked model-listen/admin callbacks through a named application registration function. This replaces module import mechanics, not source configuration or policy. Missing client retains source nullable lookup; no invented empty client is reported as a successful import. These APIs are uncompiled design sketches.

Order is **server → schema → language → CSRF → SQL → view → session → API → admin**. Session calls the existing **auth → email → identity** composite. A five-call wrapper is insufficient: native command events, compiled-client access and source locale-prefix dispatch need substantive aggregate bridges. Schema/language libraries stay plugin-free; generated admin stays application-owned and is called last from the catalog. No placeholder registrars stand in for those stages.

Existing SQL/presentation integrations retain their required Engine/Policy/Contributions/Admission/config. Their additional contracts are explicit selections, not new defaults imposed on every upstream application. M0 proves the catalog, phase ordering and gaps before broad implementation.

## Binding contracts

| Authority | Design constraint |
| --- | --- |
| Accepted source structure | Aggregate crate separate from facade; explicit optional re-exports, no dependency cycle. |
| G42 | Multiple handlers, priority then registration order, any may Abort; no route ownership/arbitration. Existing local config/service/inventory checks remain. |
| G43 and brief | Every upstream line intentional; missing config retains source deactivation. No global gate/default-deny, stronger credentials, response clearing, egress/redirect restriction or new lifetime. |
| G39–G41 | Preserve Light/Dark/unset and library-owned checks; Leptos replaces React/Vite mechanics, no new System theme. |
| G44/G45 | Existing JWT/CUID2/optional-signup-phone/AES contracts unchanged. |
| #14 | Separate runtime/build-tool registration; app resources/policy and generated admin remain explicit. No empty plugin for a support library. |
| Dependency budget | Reuse current external tuples only; internal edges need later shared-file grant. No new dependency requested. |

## Coordinator design decisions

Recommendations: accept the compiled-client seam; put new native source adapters in the aggregate instead of server dependencies in pure schema/language; use default=[]; preserve existing facade feature meanings and use a distinct aggregate-forwarder prefix. M0 must freeze exact descriptor/phase mechanics with real types. These are recommendations for review, not already implemented APIs.

## Open owner product questions

1. **Embedded PostgreSQL/PGlite:** upstream exports a PGlite connector, but Rust has no PGlite execution host. Should a later application require that exact embedded backend? **Recommendation:** explicitly assign it to **S11-PGLITE, PGlite execution-host follow-up (not released)**; do not add a dummy feature, substitute native PostgreSQL for it, or add registry tuples here. This carries the existing portability question and does not block composition of current backends.
2. **License metadata for future redistributed source assets:** upstream package.json says MIT while LICENSE/README identify Apache 2.0. Which declaration should accompany copied assets? **Recommendation:** change neither declaration and copy/distribute no such assets in this slice; ask the owner before a later distribution. This is a packaging provenance question, not a runtime fix.

No security choice is reopened. The seven Slice 9/API questions remain **open question — current behaviour follows upstream**. Finite aggregate consumers do not resolve those questions or prove PGlite, live mail/network backends or installed Tauri.

## Stop

Only these design files, the new task and one task-status row are released. No implementation stage, shared-file edit, generated output change or provider/host alteration is authorized. Coordinator review is next.
