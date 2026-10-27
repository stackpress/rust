> Historical restart checkpoint. Current evidence and limitations are in the [iOS implementation receipt](00013-receipt-lane-b-ios.md).

# Lane B iOS WIP — restart checkpoint, 2026-10-27

**Incomplete. Stop requested by the coordinator for the owner's computer restart. Do not treat this as the iOS M6 receipt or acceptance. Android has not started. Source is uncommitted.**

Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s8-lane-b`, branch `trial/3-s8-lane-b`, HEAD `1d1cbee8214390fd8ff491e63816f3f02d5e7e48` after the authorized fast-forward from current `trial/3`. Never inspect trial/1 or trial/2. Earlier accepted M1 is already merged; this work is the native iOS part of M6 only.

## Post-restart discovery — stopped as instructed, 2026-10-27

The coordinator authorized resumption subject to locating the installed Simulator.app read-only and explicitly required stopping if none is installed. No installed Simulator.app was found in the discovery scope below, so work stopped before any build or device creation. This is an environment blocker; the prior incomplete implementation and validation status below is unchanged. No final iOS receipt or acceptance is claimed.

Read-only checks performed:

- `mdfind "kMDItemCFBundleIdentifier == 'com.apple.iphonesimulator'"` returned no match.
- `mdfind -onlyin <root> "kMDItemFSName == 'Simulator.app'"` returned no match under each of `/Applications`, `~/Applications`, `/Library/Developer`, and `~/Library/Developer`.
- `xcode-select -p` still returned `/Applications/Xcode.app/Contents/Developer`. Its `Applications/` directory is absent. Xcode's `Contents/Applications/` exists but contains Accessibility Inspector, Create ML, DeviceHub, FileMerge, Icon Composer and Instruments, with no Simulator.
- Filesystem enumeration with `rg --files --hidden --no-ignore` found **zero** `/Simulator.app/` paths inside either `/Applications/Xcode.app` or `/Applications/Developer.app` (case-insensitive comparison, captured output and printed counts only).
- `/Library/Developer/PrivateFrameworks/CoreSimulator.framework/Versions/A/Applications`, `/Library/Developer/CoreSimulator/Applications`, and `~/Library/Developer/Applications` are absent. `/Applications/Simulator.app` and `~/Applications/Simulator.app` are absent.

No new build, options service, simulator device, installed app, Simulator UI process, installer, commit or push was started on this resumption. Android remains untouched; the source and HEAD are intact. The earlier archive/native checks and partial headless simulator proof do not satisfy the required inspected screenshot gate.

**Next prerequisite:** coordinator to identify an existing installed Simulator.app path, or resolve its absence through the owner. Installing anything is not authorized for this lane. Once an actual app path is available, fix discovery/owned-process cleanup in launch-ios.py and follow the remaining resume plan below; do not substitute DeviceHub or change global Xcode settings without a decision.

## Restart safety (verified before stopping)

- No task-owned build, compiler, proof, installed app, or Simulator UI process remains. The verification supervisor was interrupted at the workspace-test stage; its `finally` killed the active child process group and reaped it. The tool session exited 130, intentionally, not a passing workspace result.
- Both disposable devices created on separate proof attempts were shut down/deleted by the runner: `9331A7EA-7F0E-4CAD-AD7F-6E300821BEC6` (`Stackpress-LaneB-iOS-4d1f407c30`) and `CC18E399-B642-4959-AE48-C08FA03719D7` (`Stackpress-LaneB-iOS-248011385c`). Final `simctl list devices --json` had no `Stackpress-LaneB-iOS-` devices. The owner's other devices were not operated on.
- No Simulator.app process was successfully started: the second attempt failed locating its executable, before Popen succeeded. See the unresolved path below.
- The last G32 options service was `127.0.0.1:55007`, authenticated by the CLI's private random token. The CLI exited; a final connection was refused and `native/gen/apple/.tauri/cli-options-server.json` was absent. Tokens were never printed or retained in this note.
- Deleted task output: `target/s8-ios-native` (2,942,636 KiB), `target/s8-ios-wasm` (524,176 KiB), `native/gen/apple/build` (66,056 KiB), `native/gen/apple/Externals` (116,552 KiB), `native/gen/csr-bundle` (4,040 KiB), and proof `__pycache__`. About 3.48 GiB of measured build output removed. Generated Xcode project/configuration remains ignored under `native/gen/apple`; build wrapper recreates compiled output and bundled assets.
- No installs, commits, pushes, signing identity/team/provisioning, dev commands, devmetrics, or Android work were performed. Only the explicitly accepted G32 build-time listener was used.
- Logs/screenshots were under `/private/tmp/s8-lane-b-ios/` and **will be lost on restart**. Their essential conclusions and exact passing commands are recorded here.

## Implemented source (still needs full closeout)

### Native storage

`crates/framework/framework-preferences` now has an opt-in `native-file` feature and pure DTO exports `LoadedPreferences { revision, preferences, recovered }` and `NativePreferenceUpdate { revision, preferences }`. DTOs reject unknown fields. The native adapter is Unix-only; the approved direct `libc = 0.2.191` is under `[target.'cfg(unix)'.dependencies]`, optional, with only constants used through safe `OpenOptionsExt::custom_flags`. No new registry package/version was added to Cargo.lock.

`native_file::PreferenceFile` uses a trusted, exclusively host-owned and serialized app-config directory, a fixed `ui-preferences-v1.json` basename, revision 1, English/French canonical locale admission, and a 1,024-byte ceiling. Reads use `O_NOFOLLOW | O_NONBLOCK`, metadata on the actual opened handle (regular file and size checked before reading), and a second bounded-read limit. Missing files select English/light normally; invalid, truncated, oversized, symlinked and FIFO files recover to defaults with an explicit recovery flag, without rewriting their contents or following their targets.

Writes validate before file mutation, reject a nonregular/symlink destination, use create-new mode-0600 `.ui-preferences-v1.pending`, sync, atomic rename, and directory sync. Startup removes only that fixed abandoned staging name, so crashes cannot accumulate arbitrary temporary names. The host must own trusted ancestors and serialize the store; this is not a cooperative external-writer store. Failure after rename/fsync is unconfirmed persistence, not rollback or exactly-once storage.

Native `src/preferences.rs` supplies lazy serialized `PreferencesHost` fixed load/update methods. Commands validate authoritative Tauri WebView identity using existing `trusted`, run blocking file work off the event loop, and expose only a fixed safe failure code. Unknown DTO fields/revisions fail before storage creation. Errors retain concrete causes via `Error::new(context).with_kind(..).with_cause(cause)`.

### Exact optional hook API (approved client/shared exception)

`examples/profile-ui/shared` exports:

```rust
pub enum PreferenceChange { Locale(String), Theme(framework_ui::state::Theme) }
pub struct DisplayPreferences { pub locale: String, pub theme: framework_ui::state::Theme }
pub type PreferenceCompletion = Arc<dyn Fn(Result<DisplayPreferences, ()>) + Send + Sync>;
pub type PreferenceTransport = UnsyncCallback<(PreferenceChange, PreferenceCompletion)>;
```

`ProfilePage` and `ProfileSearch` accept optional `preferences: Option<PreferenceTransport>` (default None) and `preference_recovered: bool` (default false). Omission keeps existing immediate button behavior and adds no markup. A selected adapter persists before publishing acknowledged locale/theme together. A pending flag prevents competing saves; a generation ticket rejects duplicate callbacks from an older save during a later save. Failures retain the current locale/theme and independent dirty input, and notify `Preferences could not be saved. Your draft is preserved.` Completion must execute on the originating UI context; late completions after disposal are ignored.

Client synchronous WASM `start()` remains the default web/desktop entry. New CSR-only async `start_native()` loads and validates preferences before the single root mounts. `native-bootstrap.js` selects it only in the mobile bundle. It resolves saved system theme via matchMedia before mount and preserves the saved system mode across locale-only writes. Fixed IPC helpers have 10-second bridge deadlines; only successful matching acknowledgements publish settings. Invalid-file recovery shows `Saved preferences could not be loaded. Defaults are in use.` after initial search admission, so bootstrap cannot erase it.

No contracts/, web/, gateway/, framework-ui/, framework-view/, ui-gallery/, server/, or SQL source was edited. Existing labels/layout/default English/light and default bootstrap are intended unchanged; full byte/regression proofs remain pending.

### Native build and proof scaffolding

- `native/src/main.rs` delegates to the library's public `run`; the former app body is in `src/app.rs`, with `#[cfg_attr(mobile, tauri::mobile_entry_point)]`. Native library crate types include lib/staticlib/cdylib. The existing desktop proof module and `launch-proof.js` remain behaviorally unchanged; `proof.rs` only changes its self-crate import to `crate::NativeHost` after extraction.
- `native/tauri.ios.conf.json` uses `gen/csr-bundle`, `devUrl: null`, minimum iOS 17.0. Desktop config remains unchanged and still uses normal `../dist/csr` assets.
- `native/proofs/build-ios.py` builds bundled CSR, replaces only the mobile copy's start.js, invokes the finite unsigned simulator archive command below, enforces a 1,200-second parent watchdog with process-group kill/reap, blocks installers, and records the ephemeral options address and verified shutdown. Its local xcodebuild wrapper strips cargo-mobile2's unconditional `-allowProvisioningUpdates` and forces signing off; derived data stays under ignored native/gen. Its local security wrapper prevents credential lookup. GEM_HOME/GEM_PATH are unset in task environment because inherited Ruby variables otherwise break the already-installed CocoaPods.
- `native/src/mobile_proof.rs` + `mobile-proof.js` are explicitly environment-selected simulator fixtures (`first`, `restart`, `recovery`), absent in ordinary startup. Closed phase-control files and diagnostic navigation produce receipts in the app's own config directory; they never select arbitrary operations, paths, or JavaScript. Real Suspended/Resumed events are counted.
- `native/proofs/launch-ios.py` owns one unique device per run, installed-app launch --console processes, finite per-command/action deadlines, a 420-second outer watchdog, and shutdown/delete cleanup. It checks options-service closure before creating a device. It never operates on `booted` or a pre-existing device. This runner is **not yet passing end-to-end**, and its Simulator UI path/cleanup deserves final safety review before rerun.

## Evidence already obtained

Installed versions used: Rust/Cargo 1.96.0, Tauri CLI 2.12.4, tauri 2.12.5, tauri-build 2.7.4, Xcode 27.1 (27A9275), iOS 27.1 ARM64 simulator runtime 24A94232, xcodegen 2.46.0, libimobiledevice 1.4.0_2, CocoaPods 1.17.0, wasm-bindgen 0.2.129, cargo-llvm-cov 0.8.6. Runtime ID `com.apple.CoreSimulator.SimRuntime.iOS-27-1`; selected device type `com.apple.CoreSimulator.SimDeviceType.iPhone-Duo`. These are task-specific verified/installed versions, not a claim about every supported platform.

1. Guarded `cargo tauri ios init --ci --skip-targets-install` succeeded. No installer was attempted. It generated `native/gen/apple`. CocoaPods worked with GEM_HOME/GEM_PATH unset. No team was selected.
2. **Unsigned simulator archives built successfully twice**, including the later source fixes. Final underlying command:
   `cargo tauri ios build --ci --debug --target aarch64-sim --features app --no-sign --archive-only -- --locked`
   The latest wrapper report: exit 0, 12.85 seconds for the mobile build phase, service ran on `127.0.0.1:55007`, descriptor removed and service closed before proof, unsigned true, installer_attempted false. Bundled CSR build also passed. Archive had been `native/gen/apple/build/stackpress-example-profile-ui-native_iOS.xcarchive/Products/Applications/Stackpress Profile M0.app`; it has now been removed with build output.
3. First real installed-app run (headless CoreSimulator) reached and passed JS assertions for packaged `tauri://localhost`, denied generic commands, forged search/preference DTO rejection, symlink-induced failed persistence retaining English/light plus dirty input, later French/dark persistence retaining dirty input, permitted Grace Hopper search, detail/back navigation, actual background/resume with the same input node/draft/settings, and restart first-render French/dark. Native file contents were checked directly. It then **failed** its corrupt-file recovery check because initial search cleared the warning.
4. Fixed that recovery ordering and added `recovery_notice_survives_initial_search`. Rebuilt the iOS app. The next simulator attempt stopped before app launch because the assumed Simulator UI executable was absent. Its device was deleted. The fixed recovery behavior has native SSR test evidence but **has not yet been rerun in the installed app**.
5. Screenshot conclusion: `simctl io screenshot` in the headless attempt produced black images (all ~104 KiB). They are **not usable visual evidence**. Coordinator explicitly approved launching an owned Simulator.app bound to the disposable device and quitting only that process. No valid final images yet. The next proof must produce and visually inspect nonblack screenshots.
6. Passing targeted tests after the recovery fix: framework-preferences **12** (including five native file tests, FIFO child under 5-second kill/reap watchdog); profile-ui-shared **14** with SSR; profile-ui-client **6**; profile-ui-native **2** parent tests (existing native_cases supervises its own child); preferences doctest **1**. No failures. The printed controlled registry poison panic is intentional existing test evidence, not a failed suite.
7. Passing: workspace rustfmt check and `git diff --check`; targeted Clippy -D warnings; native app-feature Clippy -D warnings; Rustdoc -D warnings; `cargo +1.96.0 check` for the targeted scope. Exact scope below.
8. Workspace tests were started but **intentionally interrupted**, so no workspace pass. Required check.py, --sqlite, browser matrices, macOS packaged proof, byte-equivalence comparison, dependency graphs, and coverage have **not yet run for this change**.

## Review rounds and remaining findings

- Complexity pass: added generation-qualified acknowledgements so an older duplicate completion cannot publish during a newer save; targeted tests passed. Recovery ordering bug found by actual simulator proof and test-verified on SSR after fix.
- Responsibility/storage pass: retained pure DTOs in the shared default graph and filesystem behind the Unix/native-file gate; changed staging to one fixed bounded name and added abandoned staging/symlink/collision tests; targeted tests passed.
- Consistency/Rustdoc/comments pass: reviewed file/command/optional hook contracts, safe error wrappers and default props; targeted tests, doctests, Clippy, Rustdoc and MSRV passed. Final documentation/consistency review is still needed for the proof wrappers.
- Safety/soundness round and measured coverage remain **unfinished**. Review UI-process cleanup if device shutdown/delete itself fails, parent-watchdog coverage of every owned UI child, special-file controls in test scaffolding, source/platform cfg boundaries, and no native dependency leaks in browser graphs. Do not claim four completed rounds or >=70% coverage yet.

## Exact resume plan and commands

1. Read this note, git status, task 00013, decisions G29/G30/G32 and the project's Rust/coordinated workflows. Stay in this worktree and preserve the dirty source. Keep two target directories only. No new installs or Android. If current trial/3 must be merged again, coordinate that action rather than consulting other trial branches.
2. Fix Simulator UI discovery **before** creating another device. Last runner assumed:
   `/Applications/Xcode.app/Contents/Developer/Applications/Simulator.app/Contents/MacOS/Simulator`
   `xcode-select -p` returned `/Applications/Xcode.app/Contents/Developer`, but that Applications directory/executable was absent in the attempted launch. Locate the installed Simulator.app (e.g. query its bundle ID with mdfind and inspect CFBundleExecutable), resolve the actual app executable, and use a finite owned process with `-CurrentDeviceUDID <new-owned-uuid>`. If absent or tooling tries any installer, stop/report to coordinator. Do not install or change global Xcode settings. Keep its PID and ensure terminate/kill/wait even if device cleanup fails; never kill an owner's Simulator process.
3. Rebuild, then rerun the repaired proof (both scripts require the existing approved outside-sandbox CoreSimulator/Xcode access):

   ```sh
   python3 examples/profile-ui/native/proofs/build-ios.py
   python3 examples/profile-ui/native/proofs/launch-ios.py
   ```

   build-ios.py creates its own no-install guards and task env. Generated project files remain; if regeneration becomes necessary, use only a guarded `ios init --ci --skip-targets-install` after checking current --help. Do not run ios dev. Archive path configuration previously failed when forcing custom OBJROOT/SYMROOT (`BuildProductsPath` missing); this was fixed by removing those overrides and keeping only the in-worktree derivedDataPath. Do not reintroduce them.
4. Complete regression and quality commands with an outer child process-group watchdog/kill/reap (do not rely on an abandoned terminal), task environment:

   ```sh
   export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
   export CARGO_TARGET_DIR="$PWD/target/s8-ios-native"
   export PROFILE_UI_NATIVE_TARGET="$PWD/target/s8-ios-native"
   export PROFILE_UI_TARGET="$PWD/target/s8-ios-wasm"
   export PROFILE_UI_PLAYWRIGHT=/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright
   export PROFILE_UI_CHECK_LOGS=/private/tmp/s8-lane-b-ios/check-memory
   python3 examples/profile-ui/check.py
   PROFILE_UI_CHECK_LOGS=/private/tmp/s8-lane-b-ios/check-sqlite python3 examples/profile-ui/check.py --sqlite
   PROFILE_UI_BROWSER_LOGS=/private/tmp/s8-lane-b-ios/browser-memory python3 examples/profile-ui/proofs/browser.py
   PROFILE_UI_BROWSER_LOGS=/private/tmp/s8-lane-b-ios/browser-sqlite python3 examples/profile-ui/proofs/browser.py --sqlite
   ```

   Browser proofs use .invalid request interception only, existing scripts unchanged: 8 runs (four scripts × two mounts) for each backend, parent 120-second watchdog and action deadlines. Use the installed Playwright via environment, no install/listener. Existing desktop bundle uses ordinary CSR assets, not native-bootstrap.js. Finite packaged build command from native cwd: `cargo tauri build --ci --debug --no-sign --features app --bundles app -- --locked`; then run unchanged `examples/profile-ui/proofs/launch.py` against `target/s8-ios-native/debug/bundle/macos/Stackpress Profile M0.app`, setting `PROFILE_UI_LAUNCH_REPORT` under /private/tmp, for normal navigation plus existing window-close/app-quit modes. Keep bundle build/launch watchdogs. No signing credentials.

   Last passing native quality commands (run from root with the two-target env):

   ```sh
   cargo test --locked -p stackpress-framework-preferences -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native --features stackpress-framework-preferences/native-file,stackpress-example-profile-ui-shared/ssr
   cargo clippy --locked -p stackpress-framework-preferences -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native --features stackpress-framework-preferences/native-file,stackpress-example-profile-ui-shared/ssr --all-targets -- -D warnings
   cargo clippy --locked -p stackpress-example-profile-ui-native --features app --all-targets -- -D warnings
   RUSTDOCFLAGS='-D warnings' cargo doc --locked -p stackpress-framework-preferences -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native --features stackpress-framework-preferences/native-file,stackpress-example-profile-ui-shared/ssr --no-deps
   cargo +1.96.0 check --locked -p stackpress-framework-preferences -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native --features stackpress-framework-preferences/native-file,stackpress-example-profile-ui-shared/ssr
   cargo test --workspace --locked
   ```

5. Prove default SSR markup byte-equivalence against **current HEAD** shared source in an isolated temporary verification crate (not another trial branch), plus existing hydrate/CSR browser/default desktop proofs. Finish WASM normal/no-proc-macro graph checks for both client modes: no server/DB/CLI/Idea/Tauri/Tokio or libc/native-file leakage. Measure each changed crate's declared native scope using fresh cargo-llvm-cov instrumentation; report native/WASM/iOS adapter exclusions honestly and use real-app proofs for those boundaries. Development floor is 70%, not unmeasured 100%. Finish the four test-verified review rounds, final fmt/diff checks and KB baseline comparison.
6. Shut down/delete only the newly owned simulator, reap owned UI/app/build children, confirm options service closure, delete large task build output and record sizes. Write the final pre-approved `.agents/specs/00001-stackpress-rust-foundation/tasks/00013-receipt-lane-b-ios.md` with exact hook API, versions, real simulator screenshots, tests/coverage/limitations and G32 service use. Stop for coordinator review before Android. Do not commit/push.
