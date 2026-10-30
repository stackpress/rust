# Preserved CSRF source behavior

The contract follows original `packages/stackpress-csrf/src/plugin.ts` under the accepted G43 direction. Each process chooses its protection; these notes describe current behavior without adding application policy.

## No global guard

The CSRF registrar adds no global guard; routes call the service themselves. Missing CSRF config still supplies the default mechanism. Missing or nonobject sections have no path overrides; Rust object overrides use typed optional strings and preserve their startup errors.

## Comparison and failure

Generation uses 32 random bytes encoded as lowercase hex. Validation accepts arbitrary equal nonempty strings, checks UTF-8 byte lengths, then uses timing-safe byte equality. It adds no hex, request-method, Origin or expiry requirement. Matching input returns true without clearing it.

Mismatch stages 419 / Page Expired with the configured explanation, falling back only when `error` is omitted. The default text and mutation signatures are in [Service reference](api/service.md).

## State and rendering

Generate stages the outgoing cookie before a `{name, token}` data object. Clear removes aggregate request data and outgoing cookie/data, retaining the incoming cookie and query/post snapshots. The [concept table](concepts.md#the-three-operations) separates these carriers.

The form token is intended public form data. This crate chooses neither a view nor a renderer. Email's finite SSR integration correlates its hidden input with the outgoing cookie while retaining the same request/response behavior.

## Rust mechanics and evidence

Existing typed public errors, cookie-name validation, response mutability and carrier errors remain in force. Generation/clear can leave earlier mutations after a later mechanical error. Native timing-safe comparison and OS entropy are library mechanisms, not a timing certification.

The [CSRF tests](../tests/main.rs) cover arbitrary strings, UTF-8 lengths, missing/wrong shapes, exact defaults/overrides, query aggregate input, clear retention, randomness and registrar behavior. Email's [integration suite](../../framework-email/tests/cases/integration/mod.rs) and [source oracle](../../framework-email/tests/source-page-boundaries.cjs) exercise source CSRF/email/auth ordering with no listener or live email delivery. Browser and installed-host acceptance are separate.
