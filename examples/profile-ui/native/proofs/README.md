# Native preferences proofs

Desktop and mobile proof modules, environment selectors and injected scripts are
compiled only with `debug_assertions`. Release apps always create the ordinary
host/window and cannot activate these fixtures. Desktop launch proofs use the
existing unsigned debug bundle.

`proof-isolation.py compile` checks exclusion of the actual proof source files on
the installed host/iOS/Android targets without launching an app. Set
`CARGO_TARGET_DIR` to the native subdirectory of the capped proof root. After
building the native crate in debug and release profiles with feature `app`, run:

```sh
python3 examples/profile-ui/native/proofs/proof-isolation.py strings \
  --debug <native-target>/debug/profile-ui \
  --release <native-target>/release/profile-ui
```

The strings check requires desktop debug markers to be present and every proof
selector/script marker to be absent from the actual release executable. Mobile
markers are target-gated and are not expected in a desktop debug executable.

Run from the repository root with already installed Rust/Tauri/Xcode tools. These
proofs never install tools or run a development server. Build output uses the two
task target trees and ignored `native/gen`; public logs/images use
`PROFILE_UI_IOS_LOGS` (default `/private/tmp/s8-lane-b-ios`).

```sh
python3 examples/profile-ui/native/proofs/test_launch_ios.py
python3 examples/profile-ui/native/proofs/build-ios.py
python3 examples/profile-ui/native/proofs/launch-ios.py
```

The build helper checks init help and uses `--skip-targets-install` when regenerating
an absent platform project. Tool installers are blocked. G32 permits the CLI's
token-authenticated ephemeral loopback options service during the build; the
helper verifies that it exits before any simulator launch. Builds are unsigned,
use bundled assets and have no devUrl.

The launch helper requires DeviceHub to be idle and owns one uniquely named iOS
27.1 iPhone Duo. It installs and runs functional checks headlessly, including
failed persistence, dirty input, command denial, search/navigation, restart,
corrupt-file recovery and background/resume. It then attaches its own DeviceHub
display. Xcode 27.1 uses DeviceHub in place of Simulator.app. Only the observed
`LCD` display (`1`, `primary`) is app evidence; inactive outputs can block capture.
Each subprocess/action and the full matrix have deadlines. A timeout is failure.
The helper terminates the app, shuts down/deletes its device, and exits only its
recorded display PIDs. It never reuses or quits a pre-existing DeviceHub.

`PROFILE_UI_DEVICEHUB` can supply an installed app path; the default resolves from
the selected Xcode. Never install it or change global preferences in this proof.
`ios-functional.json` is written after successful functional assertions, before
display work; `ios-launch.json` requires successful completion including cleanup.
Inspect `ios-app-lcd.png` before claiming visual evidence. A failed combined
wrapper must be reported separately from any successful in-app assertions.

The additional Chromium fixture needs installed Playwright supplied by path:

```sh
PROFILE_UI_PLAYWRIGHT=/path/to/installed/playwright \
PROFILE_UI_IOS_LOGS=/private/tmp/s8-lane-b-ios \
node examples/profile-ui/native/proofs/browser-preferences.cjs
```

It uses `.invalid` request interception and a closed fixture bridge. It proves
light/dark system-mode acquisition before mounting, persistence before publication,
saved system mode across locale changes, and failed-save draft/root preservation.
It proves the client adapter, not native permission or filesystem authority; those
are exercised by Rust and installed-app proofs. The original desktop/web browser
and packaged-app proofs remain separate and unchanged.

## Android debug build and bounded emulator attempt

```sh
python3 examples/profile-ui/native/proofs/test_android_proofs.py
python3 examples/profile-ui/native/proofs/build-android.py --init-only
python3 examples/profile-ui/native/proofs/build-android.py
```

Use the installed JDK/SDK/NDK through task-only `JAVA_HOME`, `ANDROID_HOME` and
`NDK_HOME`. The defaults resolve the approved macOS installations from HOME.
`PROFILE_UI_ANDROID_LOGS` defaults to `/private/tmp/s8-lane-b-android`. Use only
one platform's native/WASM target pair at a time; this build uses
`target/s8-android-native` and `target/s8-android-wasm`.

The helper guards target/tool installation, disables AGP SDK auto-download,
aligns the app and Tauri library to installed API 37, pins build tools 36.0.0 and
NDK 29.0.13846066, and creates only a disposable project-local debug keystore
through a Gradle task. Bundled native bootstrap assets load preferences before
mounting; `devUrl` is null. Gradle's private cache lives in `gen/android`, with
daemon persistence disabled and Kotlin compilation selected in-process for both
the app and buildSrc. The helper records sockets, invokes scoped `gradlew --stop`
and checks for surviving worktree Java/Gradle/Kotlin processes before success.
[Dependency inventory](android-dependency-inventory.json) records the exact 347
cached Gradle/Maven coordinate versions from the initially empty private cache.

**Current emulator limitation (2026-10-27):** the APK builds, but
`launch-android.py` has no passing installed-app result or screenshot. The private
adb server first blocked in macOS USB initialization (`IOServiceOpen`). Disabling
USB discovery with `ADB_USB=0` passed that point; the earlier attempt then
failed with `listening on specified hostname currently unsupported` for
`-L tcp:127.0.0.1:<private-port>`.

The coordinator released one more attempt using `adb -L tcp:<private-port> server
nodaemon`. Its actual listener passed the required loopback audit before AVD
creation (`127.0.0.1:53956`). The ARM64 emulator then stayed adb-offline and hit
the 180-second boot watchdog; no APK was installed or launched. Its log reported
`hvf is not enabled on this aarch64 host`, adb-key generation failure, repeated
`qemu_mprotect__osdep: mprotect failed: Permission denied`, and a warning about
transport port 5601. These observations do not establish a root cause. The
emulator's console/transport/helper listeners were all loopback-only. Owned adb,
emulator and helper processes exited; the disposable AVD was deleted; all observed
ports closed.

The subsequent released attempt ran with escalated execution permission. Its
acceleration preflight exited 0 and reported `Hypervisor.Framework OS X Version
26.6`; the actual headless qemu process received `-accel on`. Nevertheless, qemu
again logged that HVF was not enabled and reported mprotect denials. The device
remained offline, then a boot-status adb call hit its five-second action deadline,
before the configured 300-second overall boot deadline. Installation was never
reached. Cleanup's force-stop command also timed out, but the emulator/private adb
children were reaped, the AVD deleted, and an independent audit found no surviving
owned processes or listeners. Further attempts require a new coordinator release.
There is no passing Android functional or visual result.

The launcher scopes Android user/emulator/AVD homes to ignored `gen` folders,
uses a private foreground adb server and explicit emulator ports in 5600–5680,
and rejects non-loopback TCP listeners. The private adb listener is audited with
`lsof` before even creating the AVD; an unsafe binding is immediately reaped and
never retried with another syntax. USB and mDNS discovery are disabled.
Remote commands always name its own serial/server; the owner's port 5037 and AVD
definitions are never used for control. The full worker and each action have
deadlines, kill/reap ownership and cleanup. Successful reports require closure of
all observed ports and deletion of its AVD. App-private `mobile-proof-mode`
selects only three fixed fixture modes on debug Android builds; normal startup
and release Android applications do not activate this fixture.

A coordinator-released Android launch must run outside the Codex sandbox, with
escalated execution permission: Hypervisor.framework access is required. Before
adb or any AVD access, the launcher runs `emulator -accel-check` with a 30-second
watchdog, captures both streams in `android-acceleration.log`, and requires exit
0 with the Hypervisor.Framework version report. Failure stops before device
creation. The emulator is requested with `-accel on` and a 300-second cold-boot
deadline; the launcher never retries with software emulation or another setup.
The latest failed run shows that a usable preflight report does not establish
that this installed headless qemu binary actually enables HVF.
