# Slice 2 design: decision checkpoint

Decision checkpoint for [task 00005](tasks/00005-slice-2-server-and-cli.md). The owner/coordinator accepted the design with G12–G16 and the DX changes recorded in the [decision ledger](decisions.md) and task Verification notes. Only M0 is released. The program remains Not Frozen; accepted scope does not imply later milestones are implemented.

## Answers after coordinator and owner review

| Question | Recommendation | Status / decision owner |
| --- | --- | --- |
| Q04 plugin identity/dependencies/conflicts | Validated unique IDs, explicit supplied order with missing/cycle/order errors; required and optional-present predecessor declarations; singleton config/service conflicts error; duplicate listeners/routes remain all-matching contributions; explicit handle/service replacement only. Existing priority/tie rules stay binding. | Accepted G15; M1 completes the policy implementation. |
| Q05 top-level versus nested lifecycle | CLI and HTTP both enter one host lifecycle; nested/raw dispatch never does. Abort skips remaining host stages, including Response after operation abort; Open output may still be delivered once. Failed Input/Request skips Operation. Typed Empty/status detection; no response-finally or implicit retry. | Accepted G12; M2 implementation remains gated. |
| Q05 error handling/finalization | Retain original failures independently; fresh public error response discards partial success body/headers/cookies/data. Error-listener failure preserves original and secondary, emits no recursive error, skips hooks, uses sanitized empty/generic 500 only while Open. Finalized/Attempted/Failed output never retries. Successful recovery requires an explicitly rebuilt successful response. | Accepted G12; no automatic retention of partial success output. |
| Q10 configuration | Package-owned typed sections + explicit bounded JSON layers; defaults first, supplied order thereafter, recursive object merge, arrays/scalars/null replace; deserialize/validate before startup; reject unclaimed sections and duplicate section type/name; package owns field validation/defaults. No implicit env/file/module/plugin discovery. | Accepted G15. M0 retains bounded inputs only; M1 supplies merge/decoding/unknown-section validation. Nest.merge stays shallow. |
| Q14 diagnostics/correlation | Existing lib Error/frames/causes plus per-top-level host-local checked correlation and host stage in report; exact nested propagation; no automatic raw contents/logging/tracing dependency. Error sources retain identity through Arc in explicitly named reports. | Accepted G15 for host-local correlation. M0 proves propagation with explicit fixture IDs; M2 allocates IDs. Distributed propagation/tracing remain open. |
| Q01 first vertical capability and native target | In-memory `profile-search` proof with `--id abc123` and explicit `GET /profiles/:id`, one named operation/service/plugin and identical CLI/HTTP result. | Accepted G15 for the capability; full proof is M3/M4. Generated model awaits S3; installed Tauri target awaits S5/owner. Native host measurements do not establish Tauri support. |

Exact approved state tables, including error NoListener/Completed/Abort/failure, sent-state rules, status normalization and public recovery tests are in [lifecycle](slice-2-design-lifecycle.md); load them when implementing or reviewing Q05.

## Additional scope/API choices

| ID / question | Recommendation and reason | Status |
| --- | --- | --- |
| S2-D01 include HTTP now? | Include optional buffered HTTP value adapter, no listener/runtime. This gives the long-standing CLI+HTTP same-capability proof and keeps default host dependencies narrow. | Accepted G13; adapter lane after M0. |
| S2-D02 how does a CLI find compiled app plugins? | Reusable stackpress-cli library plus an application-owned stackpress binary in examples/capability-host; no stock/empty tools/cli binary. Explicit function composition selects compiled functions, never cwd Rust imports. | G14 supersedes the original stock-binary recommendation. |
| S2-D03 argument/control/exit policy | Operator flags before emit, target args after event; JSON stdout only, progress/prompt/metadata stderr; exit 0 success/intentional successful recovery/empty Abort, 1 operation/output failure, 2 usage/config, 130 prompt cancel. `-b` selects bounded JSON data only. | Accepted G14; full command behavior remains M3. |
| S2-D04 where do streams/listeners/develop go? | Named S2-T follow-up for real HTTP gateway, streamed bodies/backpressure/serverless/native passthrough; S3 for build/watch supervisor and filesystem/templates. No dormant exports or support claims in S2. | Accepted G13. S2-T requires a new task; no networking is authorized here. |
| S2-D05 how broad is prompt support? | All inventoried pure prompt/control semantics plus optional exclusive native TTY driver. Native non-TTY stdin requires explicit/default/buffered input or returns NonInteractive; no blocked background stdin reader. | Accepted G13. M0 proves a real PTY guard; full driver/state/control lane remains later. OS restoration errors are explicit/best-effort on Drop, not a success guarantee. |
| S2-D06 permit lib changes? | Add borrowed generic context propagation/access only; reuse existing lib semantics. External Handler/TaskHandler composition is proved with existing sealed supertrait methods. | Accepted G16; no forwarding additions or signature rewrite were needed. |
| S2-D07 capture→data rule? | Host route wrappers atomically overlay named captures onto literal aggregate-data keys per matched action; route values win; query/post unchanged; positional args remain Invocation-only; mutations persist through nested dispatch/abort/cancel. | Accepted G15; M0 wrapper proves the collision/delegation seam. Raw lib behavior stays unchanged. |
| S2-D08 selected dependencies/features? | serde/serde_json and lib in server; optional http value types; Tokio limited to tools; optional crossterm native event driver and direct regex for prompts. No tracing/hyper/net/full-runtime default host edge. | Accepted G16. M0 locks http 1.5.0/crossterm 0.29.0, with default/http trees in its receipt. Further dependencies return for review. |

## Cross-slice consistency and limits

- Slice 1's immutable runtime, Send futures, stable ties, handles, local captures for raw lib registrations, checked parsers, one-attempt load/delivery, scoped cookie removals, deep ordinary Clone/explicit Shared and private diagnostics remain binding.
- Startup phases are builder-time TaskQueues, not ordinary runtime config/listen/route emits that mutate an immutable runtime. Emitting those names later is ordinary dispatch; it cannot restart bootstrap.
- The registration function remains the essential plugin contract. Plugin authors implement neither a trait nor a macro. The server-owned Invocation extension is imported for lookup only.
- Compile-time optional plugin exclusion must be proved in a selected/disabled application build; skipping registration is not compilation exclusion. Service/config sections do not load code.
- Buffered HTTP means HTTP semantics over native value types, not a socket/TLS/serverless/stream/backpressure proof. Native port listeners remain outside this task.
- Terminal prompt/native writer guarantees are bounded by explicit owned adapters. Arbitrary application callbacks or caller-provided synchronous Write implementations can block; the framework never promises to cancel arbitrary code.
- Existing KB setup-time statements about absent Cargo code are historical. Current manifests/source/tests were inspected; this design does not promote new implementation truth into context or refresh historical source archives.

## Deliverable coverage and review gate

| Requested checkpoint | Location |
| --- | --- |
| A upstream exports/types/test behaviors and all Slice 1 deferrals | [Host inventory](slice-2-design-upstream-host.md) and [adapter inventory](slice-2-design-upstream-adapters.md). |
| B layout/public API/examples, services/config, CLI and HTTP | [Host API](slice-2-design-api.md), [lifecycle](slice-2-design-lifecycle.md) and [adapters](slice-2-design-adapters.md). |
| C own risk inventory/prevention/proving tests | [Risks and verification](slice-2-design-verification.md), R01–R33. |
| D Q04/Q05/Q10/Q14 and Q01 status | This file plus G12–G16 in the accepted spec ledger; no context promotion. |
| E M0/milestones/watchdogs/coverage/rounds/parallel ownership | [Risks and verification](slice-2-design-verification.md). |

The original design checkpoint contained no implementation. M0 is now released under the [coordinated workflow](../../workflows/coordinated-crate-development.md); its receipt is appended to task 00005 without replacing that design pointer. Stop after M0 for coordinator re-verification. No M1–M4 work, rounds, listeners, commits/pushes, publication or context promotion is authorized by that release.
