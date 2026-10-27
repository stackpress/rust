# 00013 Lane B — iOS preferences and installed-app evidence

Status: implementation evidence ready for coordinator review, with the combined simulator supervisor limitation below. No acceptance, commit or push. Android, generated CRUD and import/export were not started. Worktree `trial3-s8-lane-b`, branch `trial/3-s8-lane-b`, base/current HEAD `1d1cbee8214390fd8ff491e63816f3f02d5e7e48`.

## Result and boundary

- Installed iOS ARM64 simulator application exercised the real Tauri commands and shared Profile UI: launch, permitted search, detail/back navigation, forged/generic command denial, preference failure preserving dirty input/current content, successful persistence, restart and background/resume.
- Native preferences load before the first CSR root. French/dark restart assertions observed the first root as `Profils`/`dark`; no English/light root was mounted first. The OS launch screen and Duo chrome are outside this root assertion.
- Host-managed, nonsecret, fixed app-config file `ui-preferences-v1.json`, revision 1, maximum 1,024 bytes. Canonical English/French and light/dark/system only. Fixed `load_preferences` / `update_preferences` commands; no caller path, shell, arbitrary event, filesystem-plugin grant, token or role authority.
- File reads use safe `OpenOptionsExt::custom_flags(O_NOFOLLOW | O_NONBLOCK)`, then metadata on the opened handle verifies a regular file within the cap before a second bounded read. Corrupt/truncated/oversized/symlink/FIFO input recovers explicitly without rewriting or following it.
- Updates validate, write a create-new mode-0600 fixed staging file, sync, rename atomically and sync the directory. One fixed staging name bounds crash leftovers. Startup removes that abandoned name without following symlinks. The host exclusively owns trusted directory/ancestors and serializes the store; this is not cooperative external-writer storage. A late sync/bridge failure is unconfirmed persistence, not rollback; no automatic retry or exactly-once claim.
- Shared components contain no filesystem/Tauri dependency. Desktop/web default startup, labels, layout and initial English/light stay unchanged. Contracts/, web/, gateway/, framework-ui/, framework-view/, ui-gallery/, server/, SQL and root facade source were not edited.

## Exact optional hook API

Exported by `stackpress_example_profile_ui_shared`:

```rust
pub enum PreferenceChange { Locale(String), Theme(Theme) }
pub struct DisplayPreferences { pub locale: String, pub theme: Theme }
pub type PreferenceCompletion = Arc<dyn Fn(Result<DisplayPreferences, ()>) + Send + Sync>;
pub type PreferenceTransport = UnsyncCallback<(PreferenceChange, PreferenceCompletion)>;
```

Here `Theme` is `stackpress_framework_ui::state::Theme`, `Arc` is from std, and `UnsyncCallback` is from Leptos. `ProfilePage` / `ProfileSearch` add optional `preferences: Option<PreferenceTransport>` (None) and `preference_recovered: bool` (false). Omission preserves immediate legacy behavior and default markup.

The adapter completes on the originating UI context after persistence. Only a successful validated acknowledgement publishes locale/theme in one batch; one pending writer and a generation ticket reject competing/older duplicate completions. Disposed roots ignore late callbacks. Failure retains input and current locale/theme, and announces `Preferences could not be saved. Your draft is preserved.` Invalid-file startup recovery announces `Saved preferences could not be loaded. Defaults are in use.` after initial search admission so bootstrap does not clear it.

Existing sync WASM `start()` remains the default. New CSR-only async `start_native()` loads/admits native preferences before mounting and returns the same disposer. The iOS-only bundled bootstrap selects it. System colors resolve before mounting; saved system mode survives locale-only writes. Native bridge acquisition/write deadlines are 10 seconds. No existing search/page wire shape changed.

Pure framework-preferences DTOs: `LoadedPreferences { revision, preferences, recovered }` and `NativePreferenceUpdate { revision, preferences }`, both rejecting unknown fields. The Unix-only opt-in `native-file` feature uses the explicitly approved already-locked libc 0.2.191 constants through safe Rust. Cargo.lock adds four dependency edges and no package/version.

## Simulator evidence and the time box

**DeviceHub replaces Simulator.app in Xcode 27.1. iOS 27.1 runtime supports only the iPhone Duo (dual display); app evidence captured from display LCD (`1`, `primary`).**

- Inspected real simulator image: `/private/tmp/s8-lane-b-ios/ios-display-1.png`. It visibly shows the app, `Profils`, dark theme, French search controls, `3 profils`, Ada Lovelace and Grace Hopper. This is a direct `simctl io <owned-uuid> screenshot --display=1` capture, not a mock/browser render or desktop window capture.
- `LCD-1` (`3`, `primary-1`) was black. Earlier primary captures were the home screen; those are diagnostic failures and do not count as app evidence. Wireless/TVOut/Resizable outputs are not app evidence. No other iPhone model was attempted.
- Functional assertions completed headlessly through failed save, successful French/dark save, Grace Hopper search, detail/history return, actual Suspended/Resumed events with the same input node and dirty draft, French/dark restart first-root checks, and truncated/oversized/corrupt/symlink recovery without modifying hostile input/target. Forged shell/emit and forged search/preference fields were rejected. The runner reached its final display phase only after these assertions passed.
- DeviceHub attachment interfered with installation/foreground operations on timed attempts. Ordering installation and functional execution headlessly, then attaching the owned display, produced the inspected LCD app image.
- **The combined `launch-ios.py` supervisor is not green.** The image-producing run exceeded its 420-second parent watchdog while attempting an inactive display capture. The last permitted repeat completed the required headless cases, then a five-second `ps` query timed out during the extra final visual-state restart, before its functional JSON checkpoint. There is no passing `ios-launch.json` or saved `ios-functional.json` claimed for those attempts. Functional evidence is the completed in-app assertion path; visual evidence is the inspected LCD image from the image-producing run.
- Per the coordinator's explicit time box, no further simulator attempt was made. Subsequent source fixes prevent headless work from scanning/claiming display PIDs, limit process polling, capture only the known LCD, persist a functional checkpoint before display work, attempt shutdown/delete even after a terminate timeout, and fail closed if a display/booted device already exists. Three supervisor safety unit tests pass. These final harness fixes were not rerun as a combined installed-app proof; this remains a review limitation.
- No owner DeviceHub instance was reused or quit. Owned UUIDs were uniquely named, created one at a time and deleted. The last image-producing device was `FD8F4353-3EE0-4558-92EB-764F6672816E`; last permitted repeat was `7095AC10-A237-4DF5-AB1D-43C58EF077FE`. Final read-only audit found no DeviceHub/DevicesTrampoline process, no booted device and no `Stackpress-LaneB-iOS-` device. Apple's shared idle CoreSimulator services were left alone.

Artifacts are under `/private/tmp/s8-lane-b-ios/` and may disappear on restart. This receipt preserves the conclusions; screenshots have not been copied into the KB.

## Builds, versions and G32

- Rust/Cargo 1.96.0; Tauri CLI 2.12.4; tauri 2.12.5; tauri-build 2.7.4; Xcode 27.1 (27A9275); iOS 27.1 ARM64 runtime 24A94232, `com.apple.CoreSimulator.SimRuntime.iOS-27-1`; DeviceHub 27.1, `com.apple.dt.Devices`, trampoline `DevicesTrampoline`.
- Installed xcodegen 2.46.0, libimobiledevice 1.4.0_2 and CocoaPods 1.17.0; wasm-bindgen 0.2.129; cargo-llvm-cov 0.8.6; Node 26.2.0; Playwright 1.64.0 supplied through `PROFILE_UI_PLAYWRIGHT`. GEM_HOME/GEM_PATH were unset only in task environments to use the installed CocoaPods.
- Guarded `ios init --ci --skip-targets-install` succeeded before restart, with no installer attempted. The build helper now also checks help before regenerating an absent ignored project; its automatic fresh-project branch was syntax-checked, not re-exercised after cleanup.
- Successful unsigned simulator archive command: `cargo tauri ios build --ci --debug --target aarch64-sim --features app --no-sign --archive-only -- --locked`. Bundled CSR build passed; frontend config has `devUrl: null` and local bundled assets only. No dev command or preview/listener was started.
- [Accepted G32 mobile build exception](../decisions.md): the build-only token-authenticated WebSocket options service **did run**. The build preceding simulator proofs observed `127.0.0.1:57140` and closed its service/descriptor before those proofs. The final cfg-guarded compile, after the simulator time box, observed `127.0.0.1:55230`, took 58.81 seconds for its mobile build phase and exited 0. Its service also closed and descriptor was removed; final audit found the port closed and descriptor absent. No simulator was launched after that compile. No authentication token was printed or retained in the receipt.
- CLI source: tauri-cli 2.12.4 `src/mobile/ios/build.rs` calls `write_options`; `src/mobile/mod.rs` binds the service to `127.0.0.1:0` with a random token. cargo-mobile2's archive path supplies `-allowProvisioningUpdates`; the task-local xcodebuild wrapper strips it and forces signing off, empty identity/team and in-worktree derived data. No signing identity/team/provisioning or key was used. Tool installers were guarded; none ran.

## Verification

All commands used locked dependencies, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, and only `target/s8-ios-native` / `target/s8-ios-wasm` plus ignored platform output.

| Check | Result |
| --- | --- |
| Targeted tests: preferences 12, shared SSR 15, client 6, native host 2 | Passed, including bounded FIFO child and existing native child watchdogs |
| Preferences doctest | 1 passed; other selected doctest scopes empty |
| `cargo test --workspace --locked` | Passed, including workspace doctests |
| Targeted Clippy `--all-targets -- -D warnings`, native `app` Clippy | Passed |
| Rustdoc `RUSTDOCFLAGS='-D warnings'`, scoped `cargo +1.96.0 check` | Passed |
| CSR/hydrate WASM checks and normal/no-proc-macro dependency trees | Passed; no server/SQL/DB/CLI/Idea/Tauri/Tokio/libc runtime leak |
| `python3 examples/profile-ui/check.py` and `--sqlite` | Both passed serially, unchanged scripts |
| Original Chromium matrix: four proofs × two mounts × two backends | 16 passed serially, unchanged scripts, `.invalid` interception only |
| Default SSR body markup vs current HEAD baseline source | 24 byte-identical cases: two mounts × 0/1/2 rows × EN/FR × light/dark, hooks omitted |
| macOS bundled debug app, `--no-sign` | Built; unchanged navigation/window-close/app-quit packaged proofs all passed |
| New native-hook Chromium fixture | Passed system light/dark first-root acquisition, persist-before-publication, system-mode retention across locale-only save, failed-save draft/root preservation |
| New supervisor safety unit tests | 3 passed: no prelaunch PID claim, timeout still cleans device, never signal unrecorded display |
| rustfmt, `git diff --check` | Passed |

An earlier concurrent memory/SQLite executable build race invalidated initial regression attempts. The recorded passing check.py/browser results are subsequent serial runs after feature-changing checks completed. No source was changed to bypass those proofs. The native-hook fixture uses a closed mock bridge to prove the client adapter; native permission/filesystem authority is covered separately by native tests and installed-app assertions.

## Coverage and review rounds

Fresh LLVM instrumentation, native target, `native-file` plus shared `ssr`, tests/all-targets; exported actual test executables and merged child profiles. First-party production executable lines, excluding test code:

| Crate / measured scope | Lines | Coverage |
| --- | ---: | ---: |
| framework-preferences (including Unix file adapter) | 421 / 448 | 93.97% |
| profile-ui-shared (SSR, controllers and optional hook) | 423 / 501 | 84.43% |
| profile-ui-client (native pure seed/DTO admission) | 67 / 73 | 91.78% |
| profile-ui-native (host and preferences service, app feature off) | 228 / 258 | 88.37% |

All meet the 70% development floor. `coverage-summary.json`, `coverage.lcov` and profiles were retained under the artifact directory. Browser-only `browser.rs` and Tauri/iOS `app.rs`/proof bindings are outside this native line scope; real Chromium, macOS package and bounded iOS evidence above cover those boundaries. No whole-WASM/iOS numerical coverage, physical-device, Android or publication readiness claim.

1. **Complexity:** qualified preference acknowledgements by generation; stale/duplicate completions cannot publish into a later write. Failure/success/duplicate/disposal and invalid-ack tests pass. The real simulator caught recovery warning being cleared by initial search; moving warning publication after bootstrap fixed it, with an SSR regression and later real recovery assertions.
2. **Responsibility/abstraction:** separated pure DTOs, opt-in Unix file adapter, serialized native service, optional shared UI transport and mobile-only bootstrap. Replaced accumulating crash temp names with one fixed staging child and tests for stale symlink/collision cleanup. Real restart and system-mode roundtrip tests pass.
3. **Consistency/Rustdoc/comments:** documented file ownership, error/cancellation/order semantics, optional API defaults and native boundaries; kept production definitions before tests. Rustdoc/doctests, Clippy, MSRV and SSR byte comparisons pass. Existing desktop proof source behavior is unchanged.
4. **Safety/soundness:** reviewed opened-handle metadata, byte caps, no-follow/nonblock, atomic rename/uncertain sync, concrete error causes, authoritative window admission, no native browser dependencies, root disposal and process ownership. No unsafe Rust added; libc constants only. Unix-only command/state bindings are cfg-guarded so non-Unix hosts retain the original search-only branch; final Unix app Clippy, macOS packaged proofs and iOS compilation pass. Other native platform runtimes are not claimed. Added meaningful command/storage failure and late/invalid callback tests plus three supervisor ownership/cleanup tests; targeted instrumented tests and final checks pass. Combined simulator harness rerun limitation remains explicit above.

## Closeout

Only authorized source paths and this receipt/WIP were changed; source remains uncommitted. The historical restart WIP remains available and points here. The KB validator baseline was 146 errors / 22 warnings before receipt writing; final comparison and build-output cleanup are recorded below. These pre-existing KB issues are outside this lane's write set.

Final audit: no task app/build/display process, no booted or task-owned device, no options listener/descriptor, and no installer-attempt marker. Removed 7,916,984 KiB (7.55 GiB): native target 6,952,912 KiB; WASM target 779,040 KiB; generated Apple build 64,440 KiB; generated Externals 116,552 KiB; mobile bundle 4,040 KiB. Removed proof bytecode caches; generated project text and small frontend assets remain reproducible in the worktree. Logs, coverage and inspected image remain under /private/tmp for review. Final KB validation is unchanged at 146 errors / 22 warnings, with zero added issues. Authorized-path audit passes (34 source/document paths); rustfmt and diff checks pass. Coordinator review is the next gate; Android remains unstarted.
