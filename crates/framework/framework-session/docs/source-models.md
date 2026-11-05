# Generated Profile and Auth families

The shipped `schema.idea` is the source for the default Profile/Auth descriptors. Import
`stackpress-session/schema.idea` through PackageLoader, redeclare Profile with additional
columns, then run the public source emitters. For a session-only graph declare Application
and Session external: the links remain in metadata, with no hidden API models or tables.
The [default model package](../../framework-models/README.md) supplies the closed stock family.

## Select a family

`auth::register_with<M: SessionModels>(&mut ServerBuilder) -> Result<()>` selects the
handwritten auth processes over M's actual associated actions. `composite::register_with`
keeps auth → email → identity. `account::events::{register_with,register_listen_with}`
select the same seven account data events, immediately or in Listen. Ordinary functions
and default-family `register` remain independently usable; these functions create no
Engine, schema, guard or route ownership policy.

`storage::AccountActions<M>` is the direct-library collaborator. Construct it with the
application-owned Arc<Engine>, seed, Date offset and PasswordConfig. Profile and Auth
come from the same family; custom SourceActions implementations are retained. Signup
passes only name/type/roles to Profile.create (Q50-SIGNUP), while generated defaults may
populate additional fields. Signin joins the same extended Profile.

## Existing paths and boundaries

ProfileSchema/AuthSchema and ProfileStore/AuthStore/ProfileActions/AuthActions retain
old paths and constructor arities as adapters over generated defaults. RecordSchema and
SourceColumn use columns(), column() and describe() accessors. StoreQuery retains its
String q decoder; native family transfer preserves Dates, absent/null and insertion order.
The SQL default package feature preserves its existing surface; schema-only consumers
use default-features=false. Browser renderers contain portable UI, never native schemas,
codec seeds, Engine or action factories.

Under G53, `eq` accepts one value per field; arrays are misuse and retain source indexed-path
behavior. `has: {tags: ["red", "blue"]}` requires both values in the list column on every
dialect. A scalar `has` keeps its existing SQL. `hasnt` is unchanged: scalar values perform
positive membership, while arrays retain the source's invalid indexed-column behavior.

Source event registration defaults to JSON results. An application's generated client may
select native Profile/Auth event responses for nested account/passwordless processes and
perform the explicit JSON conversion at its native page boundary. CSV uses native records
and their insertion order. This introduces no new protection policy under G42/G43.

Evidence: [registered extension test](../tests/main/generated_family.rs) checks actual
signup inputs, signin, arbitrary CRUD, native CSV and nested JSON order. [Default traces](../tests/main/auth_flows/events.rs)
retain the source flags/errors/skips. The [aggregate source consumer](../../framework/consumers/support/source_family.rs)
uses one generated extended family for auth, API joins, admin and export.
