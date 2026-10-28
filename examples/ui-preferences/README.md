# Preferences proof

This finite example exercises Lane B M1. It starts no listener and installs nothing.
Run `python3 examples/ui-preferences/build.py`, then supply the installed Playwright
package with `PREFERENCES_PLAYWRIGHT=<package-directory> python3 examples/ui-preferences/browser.py`.
Logs, generated browser assets and screenshots go to `/private/tmp/s8-lane-b`.
The build defaults to `target/s8-native` and `target/s8-wasm`. Override them with
`PROFILE_UI_NATIVE_TARGET` and `PROFILE_UI_TARGET` to use two subdirectories of one
capped root. `PREFERENCES_LOGS` selects the logs/assets/screenshot directory for
both the build and browser proof; the defaults remain unchanged.

The native process handles one pipe request through the real HttpAdapter and exits.
The browser intercepts only `https://preferences.invalid`, blocks other origins,
and closes its contexts and browser. The parent kills/reaps the group after 120
seconds; actions and native requests have 10-second deadlines.

The host selects locale and any saved Light/Dark choice, consumes flash, and emits
no-store HTML. Unset uses readable Light SSR and a tiny immutable same-origin
classic head asset that samples the OS once before themed content. No-JS, blocked
or missing bootstrap stays Light; an explicit saved Dark stays Dark. Acquisition
never writes a cookie and later OS changes do not track. Hydration keeps the same
seed and DOM; only the Light/Dark controls save. Legacy saved `system` reads as unset.
The maintained `proofs/first-paint.cjs` matrix runs against this real example and its
WASM, including delayed hydration, no-JS, strict CSP, missing assets and early drafts.

The fixed POST action writes scoped HttpOnly locale and explicit-theme cookies. A second fixed POST
confirms the actual returned cookies before publishing locale/theme/history in one
root action. A blocked cookie or unconfirmed outcome preserves content and drafts
and announces failure. An unconfirmed write might nevertheless be saved: refresh
can reveal it. No rollback, exactly-once write or automatic retry is promised.

Language rendering uses the existing Language/LanguageContext facility. The fixed
sample date/number are translated phrase examples. The shared formatters and
specialized inputs are separate framework-ui facilities; this proof does not
exercise them or claim ICU parity.

Flash text is public and bounded. The host clears the same cookie identity and
scope before committing HTML. Root ID dedupe survives dismissal. Concurrent tabs
can read the same cookie before either clear reaches the jar, so delivery is best
effort. This is not a durable notification system.

Protected routes are always denied unless the host environment explicitly selects
`PREFERENCES_TEST_POLICY=explicit-fixture`. That opt-in policy uses test metadata
and is not authentication. No policy/role/token snapshot authorizes work; the host
checks fresh metadata before invoking the protected closure. A deployed application
must supply real identity, resource authorization and mutation CSRF policy.
The protected fixture route follows `PREFERENCES_BASE`: `/private` at the root,
or `/demo/private` with base `/demo`. An unprefixed route outside that configured
mount cannot invoke protected work. The browser proof clicks the allowed link at
both mounts after separately proving the default and forged-request denials.

This finite example does not supply installed-app storage, generated CRUD,
packaging, streaming, native screen-reader evidence or owner visual acceptance.
