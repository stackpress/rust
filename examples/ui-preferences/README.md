# Preferences proof

This finite example exercises Lane B M1. It starts no listener and installs nothing.
Run `python3 examples/ui-preferences/build.py`, then supply the installed Playwright
package with `PREFERENCES_PLAYWRIGHT=<package-directory> python3 examples/ui-preferences/browser.py`.
Logs, generated browser assets and screenshots go to `/private/tmp/s8-lane-b`.
The build uses only `target/s8-native` and `target/s8-wasm`.

The native process handles one pipe request through the real HttpAdapter and exits.
The browser intercepts only `https://preferences.invalid`, blocks other origins,
and closes its contexts and browser. The parent kills/reaps the group after 120
seconds; actions and native requests have 10-second deadlines.

The host selects locale and theme, consumes/clears flash, and emits no-store HTML
with the theme attribute and CSS in the head before rendering. System mode uses a
CSS media query. Hydration receives the same token and performs no initial storage
read or preference fetch. The example uses its own finite shell because the current
framework-view PageContract admits only light/dark. It does not modify that contract
or establish a framework-view system-theme projector.

The fixed POST action writes both scoped HttpOnly cookies. A second fixed POST
confirms the actual returned cookies before publishing locale/theme/history in one
root action. A blocked cookie or unconfirmed outcome preserves content and drafts
and announces failure. An unconfirmed write might nevertheless be saved: refresh
can reveal it. No rollback, exactly-once write or automatic retry is promised.

Language rendering uses the existing Language/LanguageContext facility. The fixed
sample date/number are translated phrase examples; full formatters and specialized
editing remain Lane A's named milestone. These examples do not claim ICU parity.

Flash text is public and bounded. The host clears the same cookie identity and
scope before committing HTML. Root ID dedupe survives dismissal. Concurrent tabs
can read the same cookie before either clear reaches the jar, so delivery is best
effort. This is not a durable notification system.

Protected routes are always denied unless the host environment explicitly selects
`PREFERENCES_TEST_POLICY=explicit-fixture`. That opt-in policy uses test metadata
and is not authentication. No policy/role/token snapshot authorizes work; the host
checks fresh metadata before invoking the protected closure. A deployed application
must supply real identity, resource authorization and mutation CSRF policy.

Installed-app files/restart, generated CRUD, packaged targets, streaming, native
screen readers and owner visual acceptance remain later gates.
