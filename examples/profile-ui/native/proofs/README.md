# Native preferences proofs

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
