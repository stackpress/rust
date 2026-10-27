# Task 00013: Slice 8, more UI

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-26, in parallel with [task 00011](00011-slice-6-generated-data-layer.md) and [task 00012](00012-slice-7-production-servers.md). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Grow the UI layer beyond the first vertical. This covers the deferred work named S5-F1 to S5-F6 in [Slice 5 decisions](../slice-5-design-decisions.md):

- **S5-F1:** advanced components (disclosure/overlay/table/navigation and the remaining input/view families), each with accessibility and behavior proofs.
- **S5-F2:** rich content (sanitized Markdown/HTML, date/number/phone, and evaluated integrations such as editors, charts and media).
- **S5-F3:** schema-to-UI generation (admin CRUD screens from the generated data layer). **Depends on task 00011.**
- **S5-F4:** language/session integration (locale routing/persistence, theme/flash cookies, permission-aware layouts).
- **S5-F5:** native targets (Tauri mobile and other desktops, signing, updater).
- **S5-F6:** async UI rendering (component-owned SSR resources, suspense, and streamed rendering, which **depends on task 00012's streaming server**).

The design must propose an order and a breadth that fit the owner's preference for broad functionality without compromising safety. It must say plainly which parts can start now and which must wait for tasks 00011 or 00012.

## Implementation steps

1. Design checkpoint, with no production code. Write it as `slice-8-design-*.md`, per workflow §5.1. It covers:
   - the upstream inventory (references 00015, 00027, 00028, 00031, 00032, 00041 and the Slice 5 inventories);
   - a library and tooling assessment, using disposable probes, including any new installs (for example mobile toolchains) as an explicit request for owner approval;
   - the API sketch with DX examples;
   - the risk inventory: accessibility, XSS and sanitization, hydration and streaming, IPC on mobile, signing secrets;
   - **proposed visual acceptance criteria**;
   - open decisions written in plain language;
   - dependencies on tasks 00011 and 00012;
   - the G23 shared-crate requests;
   - a milestone plan for about two lanes.
2. Coordinator review and owner decisions, then M0 and the milestones, rounds 1–4, closeout, owner visual acceptance and a docs follow-up.

## Verification process

As in the coordinated workflow and task 00008: browser and packaged-app proofs, accessibility checks, the WASM dependency isolation, per-crate coverage of at least 70%, and owner visual acceptance.

## Acceptance criteria

Visual acceptance controls 00013-001 to 00013-013 in [Slice 8 verification](../slice-8-design-verification.md#proposed-owner-visual-acceptance), accepted by the owner on 2026-10-26 (G29). Controls for generated CRUD, mobile and streaming stay pending their dependency and target gates.

## Implementation notes

Design checkpoint delivered on 2026-10-26: start at [Slice 8 design decisions](../slice-8-design-decisions.md). Owner decisions recorded as G29/G30 on 2026-10-26; M0 released on 2026-10-27.

**Mobile installs (coordinator, 2026-10-27, owner-approved).** Rust `aarch64-apple-ios-sim` and `aarch64-linux-android` on 1.96.0. Temurin JDK 17.0.21+9 (ARM64) unpacked to `~/Library/Java/JavaVirtualMachines/jdk-17.0.21+9`; its published SHA-256 `78b08c7f…69ed1c15` matched. Note: `/usr/libexec/java_home` without a version now resolves to this JDK; Zulu 11 remains installed. Android command-line tools 19.0 (`commandlinetools-mac-13114758_latest.zip`, SHA-1 `c3e06a19…605f6f7c1d24`, matching Google's repository manifest) at `~/Library/Android/sdk/cmdline-tools/latest`. Via its sdkmanager, using only previously accepted licenses: `platforms;android-37.0` rev 2 (Google's current name for API 37), `build-tools;36.0.0`, `ndk;29.0.13846066` (rc3, as pinned by Tauri CLI 2.12.4), `platform-tools` 37.0.1, `emulator` 37.2.12 and `system-images;android-37.0;google_apis;arm64-v8a` rev 8. iOS 27.1 simulator runtime (24A94232, ARM64) via `xcodebuild -downloadPlatform`. With owner approval, the replaced iOS 15.4/18.5 runtimes, Android 30/33 system images and NDK 27.1 were removed; the owner's existing AVD definitions were left in place.

## Verification notes

**M0, coordinator review on 2026-10-27** ([receipt](00013-receipt-m0.md); merge after `d5cc21d`). Result: **verified**; merged into `trial/3`.

- **First checkpoint:** the implementer stopped on a real counterexample: pinned Leptos/any_spawner detaches Resource loaders with no cancel handle. The coordinator directed cooperative cancellation (Owner cleanup fires an `Abortable` handle) instead of reopening the executor design. It holds within bounded scheduler yields for yielding loaders; CPU-bound loaders remain bounded by host deadlines.
- **Checks rerun by the coordinator:** fmt; Clippy `-D warnings` for framework-ui `ssr,rich` and framework-language; framework-ui tests with `ssr` and `ssr,rich`; framework-language and framework-view tests; the gallery build and its Chromium proof (4/4 at root and `/demo/v1`, no listener). Merged workspace: 537 passed, 0 failed, 75 ignored; workspace Clippy clean. The only conflict was the root members list.
- **Coverage (implementer):** framework-ui 97.92%, framework-view 95.82%, framework-language 97.32%, gallery-shared 92.47%, gallery-web 90.22%.
- **Coordinator screenshot review notes for M1:** adjacent buttons render with no gap; in the full-page 375 px dialog capture the backdrop stops partway down the page (verify the viewport-fixed backdrop, not just the screenshot mode).
- **Lane B M1** ([receipt](00013-receipt-lane-b-m1.md); merge after `5e272a9`), coordinator review on 2026-10-27: reran fmt, Clippy `-D warnings` and tests for framework-language, framework-preferences and the ui-preferences example (16 passed), the example build and its Chromium matrix (PASS, no listener). Coordinator screenshot review led to two fixes before merge (whole-word responsive heading; 8 px action gap). Coverage (implementer): language 95.24%, preferences 97.94%, example shared 88.46%, example web 91.06%. Follow-up for Lane A: framework-view's PageContract admits only light/dark, so "system" theme is projected by the example's own shell; a framework-view system-theme projector is pending.
- **Lane A M1** ([receipt](00013-receipt-lane-a-m1.md); merge `c0e117c`), coordinator review on 2026-10-27: reran fmt, Clippy `-D warnings` and tests for framework-ui `ssr,rich` (44 passed), the gallery build and Chromium proof (PASS, no listener), and the unchanged `profile-ui/check.py`. Screenshot review confirms spaced action groups and a viewport-covering modal backdrop. **Security fix:** pinned Leptos 0.8.22 emits `<textarea>`, `<style>` and `<script>` children raw during SSR; Stackpress now escapes initial textarea content, and its existing title formatter and replay-JSON escaping cover the others (source audit in the receipt; to report upstream). Coverage (implementer): framework-ui 98.16%, gallery shared 89.84%, gallery web 90.22%.
- **Streamed SSR (M7 streaming half)** ([receipt](00013-receipt-stream.md); merge `5b289d0`), coordinator review on 2026-10-27: reran fmt, Clippy `-D warnings` (framework-ui `ssr,stream` and the ui-stream example), tests (48 passed; framework-ui `ssr` alone also passes), the example build and its Chromium proof (PASS; service-worker streaming on a `.invalid` origin, no listener). Screenshot shows shell text before the delayed section, an interactive button after hydration and a hostile `</script>` payload rendered as inert text. Known boundary: Leptos re-runs a panicking Suspense child in an internal effect task whose handle it discards, so that late-panic case ends through a configurable finite deadline rather than immediately. Coverage (implementer): framework-ui 94.23%, example crates 95–100%.
- **Rich content and formatting (M3)** ([receipt](00013-receipt-format.md); merge `c27a773`), coordinator review on 2026-10-27: reran fmt, Clippy `-D warnings` and tests for framework-ui `ssr,rich,format` (57 passed), rebuilt the example's eight native/WASM builds from source (`check.py build`) and ran its Chromium proof (PASS); framework-ui also compiles with `ssr,rich,format,stream` together after both merges. Coordinator decisions: ICU aligned to 2.4.0 so no existing lock version moves; formatting stays opt-in (about 699 KB gzip of WASM when enabled, mostly locale data; a data-pruning investigation is an optional follow-up); datetimes display host-supplied explicit UTC offsets (no IANA zone resolution; named-zone conversion is a follow-up); **specialized number/date/phone edit-state inputs are deferred** as a named follow-up with the precise contract in `examples/ui-format/README.md`. Coverage (implementer): framework-ui 98.50%, example 98.72%.
- **iOS native target (M6 iOS part)** ([receipt](00013-receipt-lane-b-ios.md); merge `a8fec86`), coordinator review on 2026-10-27: reran fmt, Clippy `-D warnings` and tests for framework-preferences (`native-file`) and the profile-ui shared/client/native crates (37 passed), unchanged `profile-ui/check.py`, and all eight profile-ui browser proofs at both mounts (PASS); no edits to profile-ui contracts/web/gateway or framework-ui/view. The installed iOS 27.1 simulator app passed its in-app functional assertions (search, detail/back, forged-command denial, failed save keeping the draft, French/dark persistence across restart, background/resume, damaged-file recovery); one inspected screenshot shows the app in French/dark. **Limitations:** Xcode 27.1 replaces Simulator.app with DeviceHub, and the iOS 27.1 runtime offers only the dual-display iPhone Duo; the combined simulator wrapper never completed one fully green end-to-end run (final attempts timed out on extra screenshot/restart steps after functional checks passed; later harness fixes were not rerun, by the coordinator's time box). Default web/desktop output is byte-identical (24 SSR cases). Coverage (implementer): preferences 93.97%, shared 84.43%, client 91.78%, native 88.37%.
- **Generated role wrappers (M4)** ([receipt](00013-receipt-admin-m4.md); merge `3cbc20e`), coordinator review on 2026-10-27: new `stackpress-framework-admin` crate (optional facade import `stackpress::framework_admin`) and `examples/generated-admin`, built over the frozen revision-2 schema contract with no change to framework-schema/sql/ui/view. After merging current `trial/3` (one root-members conflict), the coordinator reran fmt, workspace Clippy `-D warnings`, framework-admin `codegen` Clippy, the facade check, the full workspace (637 passed, 0 failed, 78 ignored; 891 doctests), the scoped suite, the example's build, Chromium proof (base hydrate/CSR, plugin and disabled-plugin: all pass, no listener, SSR/hydration parity), `checks.py` and the compile proof (byte-identical regeneration, rename/removal, stale-consumer failure, hostile labels). Screenshot review: lists, views and native field forms render; hostile `</script>` label text is inert; the plugin's replacement field and fixed placeholder appear. Code review: emitted strings use Rust's escaped literal spelling and component import paths are lexically checked. Only ten concrete definitions exist by default (text/textarea/checkbox/switch/select inputs and text/yes-no/case views); the rest of the catalog diagnoses rather than degrades and is a follow-up. Coverage (implementer): framework-admin 88.14%, tool 89.89%, shared 75.12%, web 100%.
- **License note:** `ammonia` brings `cssparser` 0.38.0 under MPL-2.0, the first file-level copyleft dependency; only in the opt-in `rich` feature. Reported to the owner.

## Acceptance notes

Pending.
