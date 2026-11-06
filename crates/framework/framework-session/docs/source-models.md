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
their public paths as aliases to generated defaults. Import `columns::SourceSchema`
for `new(String, offset_minutes)` and static `describe()`/`catalog()`; schemas expose
columns()/column(). Construct a CRUD action with `new(engine, Store::new(seed, offset))`,
or use a generated SourceModel's actions factory. `storage::DefaultModels` names the
stock family for `AccountActions<DefaultModels>`; custom apps supply their own family.
RecordSchema aliases the lower metadata-backed schema. StoreQuery uses native `Value` q; native family transfer preserves Dates, absent/null and insertion order.
The SQL default package feature preserves its existing surface; schema-only consumers
use default-features=false. Browser renderers contain portable UI, never native schemas,
codec seeds, Engine or action factories.

Under G53, `eq` accepts one value per field; arrays are misuse and retain source indexed-path
behavior. `has: {tags: ["red", "blue"]}` requires both values in the list column on every
dialect. A scalar `has` keeps its existing SQL. Under G54, `hasnt` excludes each requested
value; several values require none of them. SQL NULL lists and missing JSON paths match.
A physically nonexistent SQL column remains a backend error. The shared
[membership cases](../../framework-sql/tests/support/g54.rs) pin those boundaries.

Source event registration defaults to JSON results. An application's generated client may
select native Profile/Auth event responses for nested account/passwordless processes and
perform the explicit JSON conversion at its native page boundary. CSV uses native records
and their insertion order. This introduces no new protection policy under G42/G43.

Evidence: [registered extension test](../tests/main/generated_family.rs) checks actual
signup inputs, signin, arbitrary CRUD, native CSV and nested JSON order. [Default traces](../tests/main/auth_flows/events.rs)
retain the source flags/errors/skips. The [aggregate source consumer](../../framework/consumers/support/source_family.rs)
uses one generated extended family for auth, API joins, admin and export.
