# Rich content and formatting proof

This finite example never listens on a port. It compiles one component source for
native SSR and actual browser hydrate/CSR. Outputs and logs live under
`/private/tmp/s8-format`; build trees are exclusively `target/s8-native` and
`target/s8-wasm`. No tool installation is performed.

`python3 examples/ui-format/check.py build` builds the native fixture and eight
optimized WASM variants (hydrate/CSR × base/rich/format/both), retaining dependency
trees, exact artifact sizes, deterministic gzip sizes and SHA-256 hashes. It uses
the existing matching wasm-bindgen 0.2.129 executable. Sizes belong to this
fixture, toolchain and optimization configuration; they are not universal app
overheads. The base variant has neither optional feature enabled.

Set `UI_FORMAT_PLAYWRIGHT` to an already installed Playwright package and run
`python3 examples/ui-format/browser.py`. The parent kills/reaps at 120 seconds;
each browser action has a ten-second deadline. Only explicit local fixture assets
under `https://format.invalid/` and `/demo/v1/` are intercepted. Any other request,
console error/warning, popup or policy violation fails the proof. The corpus is
tested without relying on CSP. Original parsed SSR and hydrated DOM compare
exactly; cross-mode DOM comparison removes only empty Leptos comment anchors.
Raw native/WASM sanitizer/formatter strings are compared without normalization.

`python3 examples/ui-format/check.py verify` runs selected tests, baseline SSR
tests, Clippy, Rustdoc, the pinned compiler and the locked workspace suites. Its
last KB validation retains the repository's existing missing-source baseline as
a failing result. `python3 examples/ui-format/check.py clean` removes only the two
owned target trees; it preserves installed tooling and the evidence packet.

After other Cargo jobs finish, `python3 examples/ui-format/coverage.py --run`
cleans the owned native tree, instruments the selected tests and finite native
binary, then exports LLVM line counts including first-party example sources.
Reports exclude inline test modules and dedicated test files using recorded
cutoffs. A clean run avoids mixing instrumented objects with normal rebuilds.

Display is explicitly bounded: decimal input is at most 128 ASCII bytes with no
exponent/grouping/whitespace coercion; precision is 0..=18, half-even rounding,
signed zero preserved. ISO dates use Gregorian 0001..9999; local times admit at
most nine fractional digits, reject leap seconds and 24:00, and truncate display
precision. Datetimes require a numeric UTC offset within ±14:00 (`Z` is zero;
`-00:00` is unknown and rejected). Output always shows that offset. The host must
supply the correct offset; this module does not resolve IANA zones, detect DST
gaps or choose a side of a fold. DST fixtures test supplied values immediately
before/after changes and both repeated wall times. No OS/browser locale, clock,
Intl formatter or network data affects output. International phone presentation
is the same for en/fr, using the pinned metadata pattern and canonical E.164
input; validation does not prove assignment or reachability.

## Specialized editing deferral for coordinator review

No formatted number/date/phone input is shipped or claimed by this milestone.
The display types intentionally reject partial drafts. Reusing these parsers on
every keystroke would discard valid in-progress work and cannot establish the
selected Frui helper behavior. The proposed follow-up owns a pure edit controller
and a thin Leptos DOM adapter; it must integrate Lane A's basic input conventions
through the coordinator. No editor, chart, mask or date library is selected here.

| Area | Required contract and proving cases before delivery |
| --- | --- |
| Value versus draft | Separate committed typed value, raw draft, selection, composition and validation states. Empty clears only under explicit nullable policy. Partial `-`, `1.`, incomplete ISO fields and `+33` survive typing without becoming zero or a committed invalid value. |
| Decimal grammar | Decide localized draft grammar separately from canonical storage: en/fr decimal/grouping separators, U+202F/U+00A0 paste, sign, leading/trailing zeros, explicit scale and half-even commit behavior. Reject ambiguous mixed separators; no float/exponent coercion. |
| Date/time and timezone | Decide segmented versus text editing, null/partial calendar fields, min/max inclusivity and precision. Reject impossible leap days; explicit offset is required for instants. Named-zone conversion and DST gap/fold resolution need a separate reviewed conversion/data capability; ICU presentation alone does not provide it. |
| Phone grammar | Decide explicit-region national drafts versus international-only drafts, region switching, extension exclusion, `+` partials and pasted international prefixes. Commit canonical E.164 only after complete validation; display metadata is not reachability. |
| Caret and selection | Map semantic digit positions through inserted/removed grouping separators, sign and localized decimal marks; preserve forward/backward selection, middle insertion/deletion and select-all replacement. Test UTF-16 DOM offsets against Unicode separators. |
| Paste and undo | Paste at collapsed/ranged selection, invalid/oversized clipboard data, localized separators, leading plus/sign and line breaks. Preserve native undo/redo behavior; do not fabricate trusted paste events. Real browser keyboard/clipboard proofs need separately scoped clipboard permissions if used. |
| IME and event order | Keep composition text untouched through compositionstart/update/end; commit only after the documented final input ordering. Prove cancellation, blur during composition, duplicate final events and selection stability. Synthetic event coverage must be labelled separately from real OS IME evidence. |
| Bounds and errors | Draft byte and digit limits before allocation; inclusive/exclusive typed min/max and scale checks at commit. Rejected paste/commit preserves draft and prior value with a named field error. Test zero, negative zero, exact limits and one-past limits. |
| Browser lifecycle | SSR parse/hydrate keeps node/focus/selection and pre-JS draft; CSR starts deterministically. Change locale while dirty without reinterpreting the draft. Dispose listeners, avoid RefCell guards across callbacks, and prove no duplicate change/commit emission. |
| Completion gate | Pure state transition tests; real Chromium caret, selection, paste and composition sequences at en/fr, both mounts/modes and narrow/zoomed layout; ≥70% production controller coverage; bounded browser watchdog. Packaged WebView and real OS IME remain explicit separate evidence until exercised. |

This is a proposed deferral, pending coordinator review, rather than acceptance or
removal of the specialized editing requirement. Rich-editor and media integrations
remain the separately approved add-ons identified by G29.
