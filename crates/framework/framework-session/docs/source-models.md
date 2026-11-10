# Generated Profile and Auth families

The shipped `schema.idea` is the source for application-generated Profile/Auth descriptors. Import
`stackpress-session/schema.idea` through PackageLoader, redeclare Profile with additional
columns, then run the public source emitters. For a session-only graph declare Application
and Session external: the links remain in metadata, with no hidden API models or tables.
Each application owns its generated family. The [local fixture generator](../tests/fixtures/generation.rs) demonstrates the public emitters and explicit external links.

## Select a family

`auth::register_with<M: SessionModels>(&mut ServerBuilder) -> Result<()>` selects the
handwritten auth processes over M's actual associated actions. `composite::register_with`
keeps auth → email → identity. `account::events::{register_with,register_listen_with}`
select the same seven account data events, immediately or in Listen. The generic functions remain independently usable; these functions create no
Engine, schema, guard or route ownership policy.

`storage::AccountActions<M>` is the direct-library collaborator. Construct it with the
application-owned Arc<Engine>, seed, Date offset and PasswordConfig. Profile and Auth
come from the same family; custom SourceActions implementations are retained. Signup
passes only name/type/roles to Profile.create (Q50-SIGNUP), while generated defaults may
populate additional fields. Signin joins the same extended Profile.

## Existing paths and boundaries

Import ProfileSchema/AuthSchema and the model/store/action types from the application's own
generated module. The framework supplies no stock family or model aliases. Import
`columns::SourceSchema` for `new(String, offset_minutes)` and static `describe()`/`catalog()`;
schemas expose columns()/column(). Construct CRUD actions with `new(engine, Store::new(seed, offset))`,
or use the generated SourceModel's actions factory. Supply that family's `Models` to
`storage::AccountActions<Models>`. RecordSchema aliases the lower metadata-backed schema.
StoreQuery uses native Value q; native family transfer preserves Dates, absent/null and insertion order.
Browser renderers contain portable UI, while native schemas, codec seeds, Engine and factories
remain in the native application module.

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
