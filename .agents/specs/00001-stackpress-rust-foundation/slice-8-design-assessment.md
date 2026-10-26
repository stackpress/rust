# Slice 8 library and tooling assessment

Evidence for [owner decisions](slice-8-design-decisions.md), checked 2026-10-26. Disposable sources, lockfiles, manifests and logs live only under `/private/tmp/stackpress-s8-probe/`. No production file, root manifest/lockfile, upstream source, developer tool installation, device setup, listener, signing identity or credential was changed. Library dependency sources were fetched with approved network/cache access; this was not `cargo install`, rustup target installation or system setup.

## Baseline and current tools

`tools.json` records exact finite read-only commands/output: Rust/Cargo 1.96.0, host `aarch64-apple-darwin`; installed targets are that host and `wasm32-unknown-unknown`; wasm-bindgen CLI 0.2.129; Tauri CLI 2.12.4. Keep existing locked Leptos 0.8.22/router 0.8.17/meta 0.8.7 and Tauri SDK 2.12.5; no framework/tool upgrade is proposed. Existing Cargo + matching bindgen scripts remain the build path.

Xcode 27.1 (27A9275) reports macOS SDK 27.0 and iOS/device/simulator SDK 27.1. SDK availability does not prove an installed/runnable matching simulator runtime. Directory metadata shows iOS 15.4 and 18.5 runtimes; no 27.1 runtime was observed. `xcodebuild` emitted sandbox cache/FSEvent warnings while successfully reporting its versions; no simulator or device was booted.

`mobile-inventory.json` records Android SDK at `/Users/cblanquera/Library/Android/sdk`: platforms through 35, build-tools through 35.0.0, NDK 27.1.12297006, system-image directories 30/33, no modern cmdline-tools directory. `sdkmanager`/`avdmanager` are not on PATH. The emulator executable exists but was not launched. `java_home -V` selects Zulu 11.0.18 ARM64; Android Studio's JBR is 11.0.15. JAVA_HOME/NDK_HOME are unset.

CocoaPods 1.17.0 is already installed. Ordinary `pod --version` fails because inherited GEM_PATH selects incompatible Ruby gems; **`env -u GEM_HOME -u GEM_PATH pod --version` succeeds with 1.17.0**. Reuse this scoped environment for the eventual iOS proof; no CocoaPods/Ruby install or shell-profile edit is currently requested.

## Selected pure-Rust candidates

Exact versions are proposed, not approved production dependency additions. The disposable Cargo resolver ran with declared Rust 1.96, a standalone workspace and its own lockfile. Declared MSRVs below came from fetched crate manifests; actual native tests and WASM compilation are stronger point-in-time evidence.

| Candidate / feature | Declared minimum | Assessment and recommendation |
| --- | --- | --- |
| [ammonia 4.2.4](https://docs.rs/ammonia/4.2.4/ammonia/) | 1.85 | Select optional sanitizer behind a closed policy and checked HTML type. Default sanitizer policy is not our application policy; explicitly replace tag/attribute/scheme/relative-URL rules. |
| [pulldown-cmark 0.13.5](https://docs.rs/pulldown-cmark/0.13.5/pulldown_cmark/), defaults off, html only | 1.71.1 | Select optional CommonMark parser. Parse to HTML, then sanitize; parsing Markdown alone is not an XSS guarantee. No automatic raw-HTML trust. |
| [icu_decimal 2.3.0](https://docs.rs/icu_decimal/2.3.0/icu_decimal/) + icu_locale 2.3.0 | 1.88 | Select optional deterministic decimal display with explicit locale and decimal-string input. Pinned data avoids browser Intl/OS locale drift. It is not a complete currency/monetary editing engine. |
| [icu_datetime 2.3.0](https://docs.rs/icu_datetime/2.3.0/icu_datetime/) | 1.88 | Static date fieldset compiled/tested; explicit ISO calendar date avoids ambient clocks. Full timezone/DST/relative-input integration remains M3 proof. Prefer static fieldsets and measure included data/bundle bytes. |
| [phonenumber 0.3.10+9.0.33](https://docs.rs/phonenumber/0.3.10+9.0.33/phonenumber/) | 1.76 | Optional explicit-region parsing/E.164 formatting candidate passes the probe. Its versioned metadata does not prove a phone is reachable; typed input/caret/partial-value behavior is still ours. |
| Existing native HTML + Leptos/web-sys | Existing pinned baseline | Prefer semantic table/details/dialog/select/range/date controls with explicit Rust controllers and narrow DOM adapters. They need target behavior tests; markup or native defaults do not establish all Frui behavior. |

No umbrella component library was selected on name similarity. Replacing all controls would change markup, focus, Head/Owner behavior and public customization; the current exact contracts are smaller and testable. Add rich/formatting features independently and compare enabled/disabled native/WASM dependency/bundle reports before adoption. Licenses and transitive versions must be recorded from the implementation lockfile, not presumed from a package name.

## Disposable probe outcomes

Fresh owned targets were used. `cargo clean` removed zero old files. Initial coverage-clean metadata resolution hit restricted DNS; a subsequent approved finite Cargo test fetched the pinned libraries. There was no pre-existing output to trust. These fixtures are not first-party production code and have no claimed coverage percentage.

| Probe and reproducible invocation | Actual result | What it does not establish |
| --- | --- | --- |
| `cargo test --manifest-path /private/tmp/stackpress-s8-probe/Cargo.toml --locked --offline` | **2 tests pass**, 0 fail: sanitizer corpus and deterministic number/date/phone outputs; final `date-native.log`. | Exhaustive sanitization, performance limits, an application component or every metadata/locale input. |
| `cargo check --manifest-path /private/tmp/stackpress-s8-probe/Cargo.toml --target wasm32-unknown-unknown --locked --offline` | **Pass**, including sanitizer/Markdown/ICU date/decimal/phone; `date-wasm.log`. | Actual WASM execution/string parity, final bundle size or the existing UI's complete client tree. |
| `cargo run --manifest-path /private/tmp/stackpress-s8-probe/Cargo.toml --locked --offline` | **Pass**, `results.json`: en `1,234.50`, fr `1 234,50`; dates `Oct 26, 2026` / `26 oct. 2026`; phone `+12025550123`; hostile HTML inert after sanitizer. | Formatted numeric editing/caret/IME, full money formatting, timezones or phone reachability. |
| `node /private/tmp/stackpress-s8-probe/browser.cjs` with a 120 s parent deadline | **Pass**, `browser.json`/`browser.log`: native modal focus containment, Escape and opener restoration; hostile sanitized HTML inert; acknowledged locale cookie survives reload with HttpOnly/Secure/mount Path; zero page errors; 375/1280 px at CSS zoom 1/2 have no page overflow. | Leptos component integration/hydration, production preference handler, full accessibility audit, WKWebView/mobile or OS zoom. This is an isolated platform prototype. |
| `cargo run --manifest-path /private/tmp/stackpress-s8-probe/async/Cargo.toml --locked --offline` | **Pass**, `async.log`: delayed Suspend resolves, reactive Title works while Owner remains live, collected future is Send, cleanup runs afterward under a 2 s async deadline. | Real Resource serialization/replay, hydration, cancellation/unwind per-poll Owner restoration, scheduling multiple root writers or network streaming. |

The sanitizer uses an explicit p/a/strong/em/list/code/pre/blockquote/br allowlist; a has only href/title; attributes/events/styles/SVG/MathML/images are removed. Only explicit http/https schemes are retained, relative/protocol-relative links denied, and link rel is fixed. Corpus includes script/event/encoded executable URLs, malformed SVG/MathML and style tags; repeat sanitation is equal on these samples. Recommend the same narrow initial policy; safely allowing app-relative links would require an additional typed resolver/normalization proof.

The async probe deliberately retains the Owner through complete body and Head assembly. This demonstrates the current cleanup-before-Head limitation is addressable in a distinct API, not that the current renderer should change. No Request/Response/Invocation is in a `'static` future. [Leptos rendering modes](https://book.leptos.dev/ssr/23_ssr_modes.html) distinguish buffered async and in-order/out-of-order output; collecting chunks proves buffering only.

Two probe-authoring issues were corrected outside the repository: a mismatched parenthesis in the async fixture, and the first Playwright cache lacked its browser executable. The final browser uses **installed Playwright 1.64.0 / Chromium 1248**, module `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright`. Browser launch required approved execution outside the sandbox; all HTTP is intercepted at a synthetic HTTPS origin over browser pipes and the browser closes in finally. No browser/package was downloaded. A user-local npx cache remains a reproducibility limitation.

## Editors, charts and media: evaluated, not selected

| Option | Fit and required custom boundary |
| --- | --- |
| Existing Frui CodeEditor with CodeMirror 6.0.2 core/language-data | Its DOM-oriented core is viable behind a thin adapter, but React hooks/props are not reusable Leptos code. Need mount/destroy, selection/IME/history, bounded language extensions, CSP and accessibility proofs before choosing a pinned core package set. Official guide fetch returned 403; source imports/options were inspected read-only. |
| Existing Frui TextEditor / alternative [Tiptap 3 vanilla](https://tiptap.dev/docs/editor/getting-started/install/vanilla-javascript) | Upstream TextEditor is its own contenteditable/toolbar/history code, not Tiptap. Tiptap provides a possible non-React DOM integration, but neither proves sanitized output, paste, selection/undo or exact toolbar behavior. Keep both in later Media and rich-editor integrations. |
| [Chart.js](https://www.chartjs.org/docs/latest/general/accessibility.html) versus [Plotters 0.3.7](https://docs.rs/plotters/0.3.7/plotters/) | Canvas Chart.js requires an explicit accessible text/table alternative; Rust Plotters offers multiple drawing backends but is not an interactive Leptos chart component. Neither was installed, compiled or selected. Choose only after an actual chart requirement defines data, keyboard, SSR fallback and interaction. |
| Native image/audio/video and existing Carousel/Film contracts | Prefer explicit native media elements with text/caption fallback if later needed. Upload/storage, playback policy, remote URL grants, gesture/autoplay and device codecs need separate proofs; no iframe/CDN/widget or media package is selected now. |

The small first rich editor is a bounded textarea plus sanitized preview, not a claim of rich-text editor parity. No JavaScript package/tool installation is necessary for the selected Rust/parser/formatter/semantic-component proofs.

## Owner installation request

**Approval requested, nothing executed.** Recommend the iOS simulator set first; Android is independently optional. Never run `tauri ... init` to discover missing tools: it can install targets/SDKs. Review install receipts before initialization; later init must use `--skip-targets-install` and already provisioned tools. No implicit license acceptance or host-wide environment change is requested.

| Target choice | Exact requested installation / configuration | Current status |
| --- | --- | --- |
| iOS ARM64 simulator | Rust **`aarch64-apple-ios-sim`** on the existing 1.96.0 toolchain; **iOS 27.1 ARM64 simulator runtime** for selected Xcode 27.1; a disposable named simulator device later | Rust target/runtime missing. Full Xcode already installed. A runtime download can be requested with `xcodebuild -downloadPlatform iOS -buildVersion 27.1 -architectureVariant arm64`; Apple availability/download is an install-stage check, not proved here. |
| iOS device, only if separately selected | Rust **`aarch64-apple-ios`**; device developer mode/pairing, an owner-selected team/profile for the app | Not needed for simulator proof. No signing identities/profiles were inspected or requested as secrets. |
| iOS Intel simulator, only if separately selected | Rust **`x86_64-apple-ios`** plus compatible Intel simulator host/runtime | Not needed on this ARM64 host; no Intel simulator support claim. |
| Android ARM64 device/emulator | Rust **`aarch64-linux-android`**; **Temurin JDK 17.0.21+9 macOS ARM64**; Google mac command-line tools archive **`commandlinetools-mac-13114758_latest.zip`** used by pinned CLI; SDK **`platforms;android-37`**, Build Tools **`build-tools;36.0.0`**, NDK **`ndk;29.0.13846066`**, Platform Tools **37.0.1** | All relevant Rust/JDK/platform/build-tools/NDK versions missing or older. Set task-scoped JAVA_HOME/ANDROID_HOME/NDK_HOME to exact approved paths; leave existing SDKs installed. |
| Android emulator run | SDK **`emulator`** package plus **`system-images;android-37;google_apis;arm64-v8a`**, then one disposable named ARM64 AVD | Emulator executable exists but version/compatibility and requested image availability are not proved. Installer must record the resolved emulator/image package revisions; if the named API-37 image is unavailable, stop for target choice rather than substitute another API silently. |
| Android additional ABIs, only if selected | Rust **`armv7-linux-androideabi`**, **`i686-linux-android`**, **`x86_64-linux-android`** and matching image/device | Not needed for first ARM64 proof; no install or ABI support claim. |

The Android versions above follow **installed/pinned Tauri CLI 2.12.4 source**, not generic “latest”: its Android module specifies SDK 37 and NDK 29.0.13846066; app template compile/target SDK is 37, Android Gradle plugin is 9.3.1, Kotlin plugin 2.2.10, Gradle wrapper 9.6.1. Those exact generated Gradle/Maven dependencies will require approved downloads at the later build stage, not global Gradle/Kotlin installs. [Pinned Android module](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-cli/src/mobile/android/mod.rs), [app template](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-cli/templates/mobile/android/app/build.gradle.kts), [wrapper](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-cli/templates/mobile/android/gradle/wrapper/gradle-wrapper.properties).

Android's AGP 9.3 compatibility table specifies Build Tools 36.0.0 and JDK 17; the proposed ARM64 JDK patch is the checked Temurin release, and Platform Tools revision is from its official release notes. [AGP compatibility](https://developer.android.com/build/releases/agp-9-3-0-release-notes), [Temurin 17.0.21+9](https://github.com/adoptium/temurin17-binaries/releases/tag/jdk-17.0.21+9), [Platform Tools 37.0.1](https://developer.android.com/tools/releases/platform-tools).

For iOS use the already working scoped CocoaPods 1.17.0 environment, not another install. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) list full Xcode/CocoaPods and platform Rust targets; [Apple components](https://developer.apple.com/documentation/xcode/downloading-and-installing-additional-xcode-components) describe runtime acquisition. Local xcodebuild help confirms the command options; the Apple page's Markdown endpoint could not be fetched, so it is not evidence that a specific runtime is downloadable.

The requested “emulator” SDK package is versioned by Google's repository at install time, unlike the pinned NDK/build-tools IDs. Approval of the Android set must include receipt/review of its exact resolved revision before a run; this packet does not invent an unverified emulator version. SDK/system-image/JDK archives must have their provenance/hashes captured by the installing operator. No license terms are accepted here.

## Other desktops, signing and updater

macOS Intel/universal would require `x86_64-apple-darwin` plus actual Intel/universal launch proofs; no cross-build claim follows from this ARM host. Windows needs a Windows runner, MSVC C++ tools and WebView2; Linux needs a Linux runner and its WebKitGTK/build dependencies per Tauri prerequisites. They are separate explicit host approvals, not packages to install on this Mac.

Keep the existing unsigned macOS local app and proposed simulator/emulator builds as development artifacts. Signed macOS/iOS/Android distribution and an updater need owner-selected app identifiers, teams, provisioning/keystore policy, update origins/channels, key storage/rotation and release approval. Never put private signing/update keys in schemas, public snapshots, frontend assets, tracked config or proof logs. No updater endpoint, key generation, account enrollment, notarization or publishing is requested.

## Checkpoint limitations

No production Rust tests/coverage, real generated CRUD, actual mobile compile/launch, resource hydration, actual streamed sink, packaged new components, axe scan, manual screen reader or owner visual acceptance is claimed. Those are the explicit milestone gates. Before installing the packet, the Agent Workspace baseline passed with warnings only; final validation must preserve existing diagnostics and name any new ones.
