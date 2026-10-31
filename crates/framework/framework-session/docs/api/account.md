# Account loaders, actions and enrollment

Select `auth-server` for the native `account` module. Its page handlers use the same ordinary async Request/Response/Invocation signature as auth; direct use does not add registrar/global checks. [Account guide](../account.md) lists all source routes, presentation behavior and sequential side effects.

## Load and mutate an account

[`account/profile.rs`](../../src/account/profile.rs), [account traces](../../tests/main/account_flows.rs) and [real SQL account tests](../../tests/main/account_flows/sql.rs) anchor these functions.

| Function | Arguments / result |
| --- | --- |
| `account` | Ordinary page handler; cookie identity → profile-detail → auth-search |
| `load_account_profile` | Async `(&Invocation, columns::Value) -> lib::Result<Response>`; auth projection includes only id/type/token/verified |
| `update` | Ordinary page handler; profile then username/email/phone writes, source partial effects |
| `update_auth` | Async `(&Invocation, profile: Value, kind: &str, id: Option<Value>, token: Option<Value>, secret: Option<Value>) -> Result<Option<Response>>`; blank token returns None, existing ID updates, otherwise creates |
| `password` | Ordinary page handler; matches current hash then sequential updates |
| `assert_password` | `(&str, current: &str, &PasswordConfig) -> Option<Record>` field errors |
| `remove` | Ordinary page handler; source confirmed/current-password branch, removes auth rows then emits profile-remove with the parent response |
| `export` | Ordinary page handler; warning unless download truthy, then profile-only CSV attachment |

**Returns:** process effects/errors remain on the supplied Response and normal paths continue; native failures use lib Result. `load_account_profile` returns the selected child response. [Event adapters](../../src/account/events.rs) register auth-search/update/create/remove and profile-detail/update/remove explicitly, without selecting access policy, installing tables or adding authorization. They skip existing truthy body/non-200 status/missing Engine, validate required IDs before I/O and return source first-row/null shapes. Remove selects by ID alone.

**Resolved under G46:** account deletion is intended soft delete via @active; the existing account flow retains Profile. Omitted phone on update follows the original source, unchanged; G44's optional phone ruling applies to signup. See [decision evidence](../source-behavior.md#owner-decisions-g46).

## CSV helper

`account::csv(&Record) -> lib::Result<String>` emits source two-column rows in insertion order, with nested values flattened and leading formula characters prefixed. No sorting step is inserted. [Export source](../../src/account/export.rs) and [byte-exact nested export test](../../tests/main/account_flows/sql.rs) pin it.

<!-- example: server -->
```rust
use stackpress_framework_session::{account, columns::Record};
use stackpress_lib::Result;

pub fn ordered_csv() -> Result<()> {
    let profile = Record::from_iter([
        ("name".into(), "Reader".into()),
        ("type".into(), "person".into()),
    ]);
    assert_eq!(account::csv(&profile)?, "name,\"Reader\"\ntype,\"person\"");
    Ok(())
}
```

The order in the constructor is observable in the download. Dates stay native until the CSV object branch; [Value/Date reference](storage.md) explains JSON conversion at explicit wire boundaries.

## Enrollment and removal

`enroll` and `remove_factor` are ordinary page handlers. `provisioning_uri(issuer,name,secret) -> String` retains source double-encoding of the issuer label; `qr_data_url(&str) -> lib::Result<String>` emits base64 PNG with M correction, margin 4 and scale 4. The approved Rust PNG/QR encoder internals can differ in compression bytes while URI/pixels/display contract follows source. [TOTP page source](../../src/account/totp.rs), [account source oracle](../../tests/account-flows.json) and [SQL enrollment test](../../tests/main/account_flows/sql.rs) are the anchors.

Enrollment deliberately renders secret/QR/profile/authId; POST validates source CSRF/code. Remove-factor's truthy confirmed branch can run on GET or POST, calls no csrf.valid and uses ID-only removal. There is no extra revocation, row-count or global policy check. [Account trace](../../tests/main/account_flows.rs) covers these cases; [shared UI](../guides/shared-ui.md) explains the deliberate browser values and copy action.
