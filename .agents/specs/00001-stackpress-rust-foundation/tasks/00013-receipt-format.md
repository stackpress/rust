# Slice 8 M3: rich content and formatting implementer receipt

Task 00013; branch `trial/3-s8-format` at base `7d30bb1d91946153f8b1e3ff5e011652f39f64e4`; 2026-10-27. Implementation evidence for coordinator review under [G29](../decisions.md), [the accepted component inventory](../slice-8-design-inventory-components.md) and [M3 risks/controls](../slice-8-design-verification.md). This receipt does not grant owner acceptance, publication or promotion into context. The named receipt was pre-approved in the lane brief.

## Delivered contract and source ownership

[Rich content](../../../../crates/leptos/src/rich.rs) retains the private checked `SafeHtml` constructor and optional `rich` feature. **Policy revision 2** explicitly replaces tag/attribute/value/content/scheme defaults and additionally rejects scheme-only HTTP paths, whitespace and control bytes in URLs. Only p/a/strong/em/ul/ol/li/code/pre/blockquote/br survive; only anchor href/title and fixed rel noopener/noreferrer remain. No scripts, style, events, SVG, MathML, forms, frames, images, comments, relative/protocol-relative/data/javascript URLs or automatic resource loading. HTML and Markdown share this one sanitizer. The existing 64/256 KiB input/output ceilings remain, including lowerable HTML ceilings and bounded Markdown intermediate output. Library parsing is bounded trusted work, not a CPU sandbox.

[The optional format module](../../../../crates/leptos/src/format.rs) owns safe error categories, retained original error causes and the closed English/French locale selection. Its three capability modules are [decimal](../../../../crates/leptos/src/format_number.rs), [Gregorian date/time](../../../../crates/leptos/src/format_time.rs) and [E.164 phone](../../../../crates/leptos/src/format_phone.rs). No formatting library is enabled by default. These are text/display APIs, with no unchecked HTML or deserialization constructor and no ambient locale, Intl, clock, OS timezone, browser-language guess or network acquisition.

- Decimal: `-?digits(.digits)?`, at most 128 ASCII bytes; no exponent, plus, whitespace, grouping or invalid-to-zero coercion. `NumberFormat::new` requires explicit 0..=18 fractional digits and grouping selection. Half-even rounding, explicit padding, canonical removal of integer padding and preserved negative zero; no binary float conversion.
- Dates: exactly `YYYY-MM-DD`, Gregorian years 0001..9999, checked leap/month/day rules. Local times admit `HH:MM[:SS[.1..9 digits]]`, reject leap seconds and 24:00, and display at explicit minute/second/millisecond/microsecond/nanosecond precision. Fractional display truncates and pads; it never carries to another day.
- Datetimes: typed local date/time plus explicit `Z` or `±HH:MM`, input at most 35 bytes, bounded to ±14:00. Unknown `-00:00` is rejected; all displayed datetimes include their UTC offset. The formatter preserves the supplied wall time/offset. **It does not resolve IANA zones or validate DST gaps/folds.** The authoritative host supplies the correct offset. Tests distinguish both repeated hours and values on either side of US/European spring/autumn changes. This is explicit-offset display evidence, not a timezone conversion-engine claim.
- Phones: canonical ASCII `+` followed by at most fifteen digits, no region guessing, spaces, extension loss or lenient normalization. Pinned metadata validation and exact E.164 round-trip are required. International display is the same for English/French. No assignment, ownership or reachability claim.

[The finite proof example](../../../../examples/ui-format/README.md) owns all native/browser/build fixtures. No ui-gallery, framework-view, streamed-rendering, profile-ui, server, SQL, schema, lib, CLI or Idea source was edited. Shared-file changes are only one root workspace member, lock resolution, and minimal framework-ui module/feature/dependency lines. No root facade alias was added outside the allowed root write set; an integration alias is coordinator-owned.

## Specialized editing: explicit proposed deferral

**Specialized decimal/date/phone edit-state delivery is deferred for coordinator review, not accepted or silently removed.** No formatted editing component or full Frui input parity is claimed. The selected display types reject incomplete drafts and must not be used as keystroke reducers.

The [precise follow-up contract](../../../../examples/ui-format/README.md#specialized-editing-deferral-for-coordinator-review) defines separate raw draft/committed value/selection/composition/error state; en/fr decimal/grouping and pasted Unicode separators; nullable/partial signs/points/dates/phone prefixes; strict min/max and scale bounds; UTF-16 caret/selection mapping; middle insertion/deletion and undo/redo; paste/clipboard admission; IME event order/cancellation/blur and a distinction between synthetic events and real OS IME evidence; pre-JS draft and focused-node hydration adoption; dirty locale changes; scoped listeners and no duplicate commits. It requires native transition tests, real Chromium caret/paste/composition proofs at both modes/mounts, bounded watchdogs and at least 70% controller coverage. Named-zone conversion/DST disambiguation requires a separately reviewed conversion/data capability. Integration with Lane A's ordinary inputs must go through the coordinator.

Reason: a display parser does not preserve partial editing state or native selection/composition. Adapting it to every input event would lose work. The brief expressly permits this precise written deferral if the complete edit-state proof is too large for M3. Rich editors, masks, charts and media remain separate G29 add-ons.

## Sanitizer and browser evidence

[The shared corpus](../../../../examples/ui-format/corpus.json) has 29 hostile/safe cases exercised through HTML and Markdown (58 outputs): entity/control/case/percent-encoded schemes, script terminators, malformed/nested anchors, SVG foreignObject/style namespace confusion, MathML mutation patterns, stylesheet imports, events/generic attributes, frames/srcdoc/template, images/srcset/media/object resources, forms/base/meta refresh, noscript/rawtext, comments/CDATA, NUL, double encoding, literal code/Unicode separators and valid HTTP(S) links. Native tests check repeat equality, re-sanitization equality, exact budget boundaries and Markdown expansion rejection. Actual WASM repeats the same checks.

`/private/tmp/s8-format/browser.json` and `browser.log`: **4/4 real Chromium cases pass**, hydrate/CSR at root and `/demo/v1`, comparing **177 exact native/WASM outputs** per case. Different ambient locales/timezones are used (en-US/New York versus fr-FR/Tokyo). Parsed initial SSR and hydrated DOM compare exactly, and hydration retains the original root node. Semantic SSR/hydrate/CSR DOM and per-fragment output remove **only empty Leptos comment anchors**; sanitizer/formatter payload strings compare exactly without normalization. A second browser parse remains stable with only allowed HTML-namespace elements/attributes/schemes. No script marker, popup, console error/warning or unexpected resource request occurs. Root disposal removes the component.

The page does not rely on CSP to make the corpus inert. [The browser driver](../../../../examples/ui-format/browser.cjs) intercepts only fixed `https://format.invalid/` fixture assets at the two mounts, aborts/fails any other request, uses ten-second action deadlines and closes contexts/browser in finally. [The parent](../../../../examples/ui-format/browser.py) kills/reaps the process group at **120 seconds**; timeout is failure. The first sandbox launch was denied by macOS MachPortRendezvous bootstrap; the finite proof was rerun with approved unsandboxed browser execution. The initial fragment comparison also exposed Leptos's empty anchors; the comparator now removes precisely those framework markers while retaining exact raw string comparison. Neither failed attempt is represented as passing evidence.

There are **48 screenshots** under `/private/tmp/s8-format/screenshots/`: for each `{root,mounted}-{hydrate,csr}`, rich/format/date views at 375/1280 CSS px and zoom 1/2. Geometry checks reject page-wide overflow. Desktop grouped French numbers and mounted CSR narrow/200% repeated-hour offsets were visually inspected: neutral grayscale, legible wrapping and distinct UTC offsets. CSS zoom is a geometry aid, not proof of every OS magnification mode. Owner control 00013-009 review and packaged WKWebView/real assistive-technology checks remain separate.

Tool versions: Rust/Cargo 1.96.0, aarch64-apple-darwin; actual wasm32-unknown-unknown builds; wasm-bindgen 0.2.129; Node v26.2.0; Python 3.11.4; Playwright 1.64.0; Chromium 156.0.8078.4. The installed Playwright package is supplied only by `UI_FORMAT_PLAYWRIGHT`, never hard-coded in repository scripts. No install or listener occurred.

## Dependencies, licenses and bundle budget for coordinator review

Direct new production dependencies are coordinator-approved `icu_decimal =2.4.0`, `icu_locale =2.4.0`, `icu_datetime =2.4.0` and `icu_calendar =2.4.0`, plus `phonenumber =0.3.10` (resolved metadata build `0.3.10+9.0.33`). The coordinator expressly approved alignment from the design's probed 2.3.0 to 2.4.0 to preserve the core URL/IDNA graph. Datetime uses defaults off/compiled data only; calendar defaults off. Existing Leptos and bindgen versions remain unchanged. `format` and `rich` are independent opt-ins; mode-specific default/disabled WASM trees contain none of the new formatting or rich libraries. Every enabled tree excludes server, SQL/SQLx, CLI, Idea, Tauri and Tokio runtime edges.

`dependencies.json` records exact resolved selected-WASM registry-manifest licenses and declared MSRV; `lock-delta.json` and `icu24-lock-audit.json` record the full lock delta. **Zero existing package versions move; no package is removed.** Existing ICU collections/locale_core/normalizer(+data)/properties(+data)/provider stay 2.4.0; URL/IDNA versions are untouched. New ICU formatter and data packages, including time_data, are all exactly 2.4.0. All audited ICU 2.4.0 manifests declare MSRV **1.88** and **Unicode-3.0**, below workspace Rust 1.96. Lock-only atomic-polyfill/critical-section are inactive for the selected native/WASM builds; no unused-target sources were fetched merely for metadata. Approved crate sources were downloaded, without installing tools. Superseded 2.3 attempt logs/assets are retained under `icu23-superseded/`; they are not current verification or accepted lock evidence.

| New/changed dependency group | License | Declared MSRV |
| --- | --- | --- |
| ICU calendar/datetime/decimal/locale/locale_fallback/plurals/time 2.4.0 and their calendar/datetime/decimal/locale/locale_fallback/plurals/time data crates 2.4.0 | Unicode-3.0 | 1.88 |
| icu_pattern 0.5.1 | Unicode-3.0 | 1.88 |
| fixed_decimal 0.7.3 | Unicode-3.0 | 1.82 |
| calendrical_calculations 0.2.4 | Apache-2.0 | 1.82 |
| phonenumber 0.3.10+9.0.33 | Apache-2.0 | 1.76.0 |
| libm 0.2.16; nom 7.1.3; quick-xml 0.41.0; strum/strum_macros 0.27.2 | MIT | 1.63; 1.48; 1.79; 1.66.1 |
| core_maths 0.1.1; regex-cache 0.2.1 | MIT | Unspecified |
| cobs 0.3.0; embedded-io 0.4.0/0.6.1; hash32 0.2.1; heapless 0.7.17; postcard 1.1.3; regex-syntax 0.6.29 | MIT OR Apache-2.0 | Unspecified |
| linked-hash-map 0.5.6; lru-cache 0.1.2; minimal-lexical 0.2.1; oncemutex 0.1.1 | MIT/Apache-2.0 | Unspecified |

Existing sanitizer dependencies retain the M0 audited licenses/MSRV: ammonia 4.2.4 MIT OR Apache-2.0/1.85, pulldown-cmark 0.13.5 MIT/1.71.1, cssparser 0.38.0 MPL-2.0/1.85, parser/atom dependency details in [M0's dependency receipt](00013-receipt-m0.md). No license or MSRV is inferred for crates with unspecified manifest fields. The private first-party example inherits workspace MSRV 1.96 and is not published.

Optimized post-bindgen WASM sizes, exact bytes; deterministic gzip uses mtime 0. Release settings: opt-level z, thin LTO, one codegen unit, panic abort, no debug info; incremental disabled. This fixture's selected functions/data are measured, including its corpus/case strings. These are **fixture-specific costs, not universal app overhead or a production size guarantee**. JS glue sizes, hashes and every artifact are in `sizes.json`.

| Mode/selection | WASM bytes | gzip bytes | WASM delta from base | gzip delta from base |
| --- | ---: | ---: | ---: | ---: |
| hydrate base | 194760 | 63049 | 0 | 0 |
| hydrate rich | 1384974 | 467906 | 1190214 | 404857 |
| hydrate format | 2815003 | 761973 | 2620243 | 698924 |
| hydrate both | 3974282 | 1156931 | 3779522 | 1093882 |
| csr base | 168314 | 55519 | 0 | 0 |
| csr rich | 1334480 | 454723 | 1166166 | 399204 |
| csr format | 2789192 | 754945 | 2620878 | 699426 |
| csr both | 3928912 | 1143808 | 3760598 | 1088289 |

**Coordinator decision remains:** accept the re-measured optional bundle costs, or request a separately scoped data-pruning/subfeature investigation. The ICU 2.4.0 alignment is already approved; the final lock changes only by additions. No default enablement, unapproved formatter library, runtime Intl replacement or silent weaker validation was introduced to conceal the cost. The table records the completed 2.4.0 build matrix; all 177 native outputs also equal the superseded 2.3 snapshot exactly.

## Four review rounds

1. **Logic/complexity:** reviewed decimal grammar/rounding/zero/precision boundaries, byte-before-parse ordering, ISO indexing and offset semantics, sanitizer URL/resource paths and repeat equality. The initial decimal test exposed preserved integer padding; display now trims integer padding while retaining signed zero. The post-baseline review added the explicit-authority/control-byte filter and revision 2 corpus cases. `round1-tests.log` passes selected tests/doctests.
2. **Responsibility/abstraction:** reviewed feature isolation, typed admission versus display, separate decimal/time/phone modules, closed sanitizer ownership, private checked insertion, host-authoritative timezone values and pure proof source versus thin browser adapter. Existing boundaries were cohesive; no additional trait/wrapper or extraction was justified. `round2-clippy.log` and subsequent selected `tests.log` pass. Coordinator review identified the core URL/IDNA graph impact; approved ICU 2.4.0 alignment now preserves every existing version, and all aligned native/WASM checks rerun. Editing and named-zone conversion remain explicit capabilities rather than being hidden inside a display parser.
3. **Consistency/Rustdoc/comments:** reviewed public error categories/causes, typed values, explicit precision, bounded grammar, comments at parsing/rounding/stateful boundaries, executable examples and mode-specific ownership. Applied rustfmt; documentation states half-even decimal rounding, truncated time precision, negative zero, fixed-offset limits and phone validation limitations. `fmt.log`, `tests.log`, `clippy.log`, `msrv.log` and final warning-denied Rustdoc provide verification.
4. **Safety/soundness:** reviewed UTF-8 slicing guards, fixed-width integer casts and multiplication bounds, input/output growth, immutable pinned metadata, closed rich insertion and no source-bearing public error messages. Added Unicode/control insertion at every date/time wire boundary and retained-cause downcast tests. Reviewed actual browser reparse, zero unexpected resources, context/Owner disposal and watchdog cleanup. Final native coverage suite and browser packet verify these paths. No unsafe code, RefCell callback/await state or new executor was introduced.

## Final verification and cleanup

All Cargo commands use `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`; only the two named owned compiled target roots are used. Never `--all-features`.

The current results below use aligned ICU 2.4.0, not the superseded probe graph. Round-one/two pre-alignment logs are historical; final aligned suites repeat the complete selected behavior.

| Gate | Result / evidence under `/private/tmp/s8-format/` |
| --- | --- |
| `cargo fmt --all -- --check` | Pass, `fmt.log`. |
| Selected framework-ui/example `--features ssr,rich,format` tests/doctests | 52 native tests plus 6 doctests pass, `tests.log`; exact all-177-output 2.3/2.4 equality in `icu24-native-parity.json`. |
| Baseline framework-ui `--features ssr` tests/doctests | 38 native tests plus 5 doctests pass, `ssr-tests.log`. |
| Selected native Clippy `--all-targets -- -D warnings` | Pass, `clippy.log`. |
| `cargo +1.96.0 check ... --all-targets --locked` | Pass, `msrv.log`; selected ICU crate/data declared MSRVs are 1.88. |
| Warning-denied selected Rustdoc | Pass, `rustdoc.log`. |
| Eight optimized actual WASM feature/mode builds and normal/no-proc-macro dependency trees | Pass, `build-final.log`, mode/selection build/bindgen/tree logs and `sizes.json`. |
| Actual hydrate/CSR WASM Clippy `-D warnings` | Pass separately, `hydrate-clippy.log` and `csr-clippy.log`. |
| Actual Chromium modes/mounts | 4/4 pass, `browser.json`, `browser.log`, 48 screenshots; driver checks complete aligned matrix and all artifact SHA-256 values. |
| `cargo test --workspace --all-targets --locked --offline` | Pass, `workspace-tests.log`; 588 reported passes/76 ignored, including child-watchdog reports (not a unique-test count). |
| `cargo test --workspace --doc --locked --offline` | 890 doctests pass, `workspace-docs.log`. |
| Agent Workspace validator | Existing 146 errors/22 warnings unchanged byte-for-byte in diagnostic comparison, `kb-comparison.json`; not a green gate. |

The earlier 2.3 workspace attempt failed an existing HTTP M1 `/trickle` elapsed-time assertion (`server-http/tests/m1.rs:491`, expected 80..300 ms); it is retained under `icu23-superseded/`. The complete aligned workspace run passes that test without any server source change. An initial coverage export also mixed a newer normal cdylib with instrumented objects; the final clean instrumented run replaces that diagnostic attempt rather than claiming its percentage as final.

Full `commands.json` retains exact finite build/check commands, times and statuses. [The coverage exporter](../../../../examples/ui-format/coverage.py) explicitly includes first-party example sources because cargo-llvm-cov's default report excludes examples; only exact production src paths are counted and inline cfg(test) modules/dedicated test files are excluded, with line cutoffs retained. Thin WASM DOM/start code has real browser evidence rather than a claimed native percentage.

| Fresh native production scope, `ssr,rich,format` | Covered / executable lines | Coverage |
| --- | ---: | ---: |
| framework-ui | 2306 / 2341 | 98.50% |
| ui-format example library and finite binary | 154 / 156 | 98.72% |

Coverage: installed cargo-llvm-cov 0.8.6, Rust 1.96.0, native aarch64-apple-darwin, fresh owned cleaning and `--no-rustc-wrapper` instrumentation for the selected tests and actual native binary. `coverage-final.log`, `coverage-tests.log`, `coverage-native.log`, `coverage-full.json`, `coverage-full.lcov`, `coverage-summary.json` retain results and exact per-file cutoffs/uncovered lines. New format/time/number/error logic and rich sanitizer each reach all executable lines except the defensive metadata-invalid phone rejection branch; the two uncovered example lines are fixture-failure fallbacks. Existing UI uncovered branches remain visible in the full report. No production path is excluded to raise coverage. Both crates exceed the 70% development floor; this is not publication or platform acceptance.

Fresh serial LLVM export still reports 81 unmatched dependency function records. A serial `llvm-cov show -dump -num-threads=1` audit identifies only tachys/quick_xml symbols and **zero first-party mismatch records** (`coverage-diagnostics.log`, `coverage-diagnostics-summary.json`). There is no newer-object warning in the final export. These tool diagnostics are disclosed; the verified first-party counts above do not rely on unmatched dependency records.

Both compiled target roots were deleted after verification: native removed **3295 files / 1.3 GiB**, WASM removed **6308 files / 1.8 GiB** (`clean-s8-native.log`, `clean-s8-wasm.log`). Thirty-four task-created workspace child logs were copied and byte-compared under `workspace-child-logs/` before their fixture directories were removed. The initial target directory was absent; these are this task's outputs. No external/shared build cache or installed runtime was deleted. `cleanup.json` records the exact cleanup audit. Python bytecode caches were also removed.

The KB validator baseline was already **146 errors and 22 warnings**, chiefly missing external TypeScript sibling paths in this isolated worktree (`kb-before.log`). No unrelated KB repair is authorized. Post-receipt diagnostics compare identically with that baseline. This receipt preserves requirements, sources, deferrals, versions, artifacts, rounds and limitations in one coherent Agent File; no context/tracker/index was changed.

No listener, devmetrics instance, tool install, commit, push, deployment, publishing, other-agent message or unrelated lane edit. Stop after this milestone's evidence packet and proposed deferral; coordinator review/owner visual acceptance remain separate.
