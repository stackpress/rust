# Feature and facade lookup

Features select code before compilation. Disabling a plugin requires rebuilding/redeploying. Configuration can deactivate selected behaviour, but cannot remove its code from the dependency graph. Both aggregate and root facade keep `default = []`.

## Aggregate crate features

These are features of `stackpress-framework`. Native exports are target-gated away on wasm32.

| Feature | Selection |
| --- | --- |
| native | Shared native catalog/config/command binding types; no stage registrar by itself. |
| server-commands | develop/emit/serve event registration. |
| schema-client | Compiled Client and generate bridge. |
| language | Source locale service/request hook. |
| csrf | Existing CSRF registrar and its optional server feature. |
| sql | SQL commands and compiled model listen callbacks; no backend connector chosen. |
| view | Source view config/render/build bridges and supervision types; no renderer chosen. |
| session | Existing auth → email → identity composite via session/auth-server. |
| api | Existing API registrar with api/sql; no API UI or codegen selected. |
| admin | Application-generated admin callbacks, last; no universal admin route. |
| full | All nine runtime stages; excludes codegen, renderer and database connector selection. |
| codegen | schema-client plus existing Idea/schema/SQL generation APIs; separate build-host capability. |

The no-feature normal graph is the crate alone. Real registrar reuse and native-only dependency edges are pinned by the [manifest](../../Cargo.toml), [leaf test](../../tests/leaves.rs) and [graph proof](../../consumers/graphs.py).

## Root facade features

`stackpress` feature `framework` exports `stackpress::framework` with no stage selection. `framework-aggregate-native`, `-server-commands`, `-schema-client`, `-language`, `-csrf`, `-sql`, `-view`, `-session`, `-api`, `-admin`, `-full` and `-codegen` forward their corresponding aggregate features. Existing facade feature meanings are unchanged.

| Independent feature family | Export / forwarding |
| --- | --- |
| framework-api | framework_api; -native, -sql, -codegen, -ui, -ui-ssr, -ui-hydrate, -ui-csr forward API leaves. |
| framework-csrf | framework_csrf; -server forwards its native registration feature. |
| framework-email | framework_email; -server forwards its native registration feature. |
| Existing framework-session* | Existing framework_session paths/selection retained; independent of aggregate registration. |

These leaf exports do not invoke any registrar. The default facade normal graph remains only stackpress and stackpress-lib among first-party packages. [Root manifest](../../../../../Cargo.toml) and [exports](../../../../../src/lib.rs) are the source of truth for exact spellings.

## Renderer separation and guards

Select UI families with `framework-aggregate-ui-session`, `-ui-api`, `-ui-admin` and `-ui-view`. Then choose exactly one aggregate renderer suffix: `-ssr`, `-hydrate` or `-csr`. Their weak forwarding only affects selected UI crates; a renderer alone activates no UI family or native stage.

The facade rejects two classes of build:

 - Multiple aggregate renderers in one artifact: `select exactly one aggregate renderer per artifact`.
 - On wasm32, a hydrate/CSR artifact combined with native aggregate leaves/codegen/full: `build native aggregate and browser UI as separate artifacts`.

Use the [website graph](../../consumers/website/Cargo.toml): SSR combines native stages and UI; hydrate/CSR contain only UI/runtime dependencies. API's UI/codegen leaves stay separate from aggregate api/sql selection. Native view rendering uses explicit providers; no React/Vite implementation is selected. Evidence: [graph checker](../../consumers/graphs.py), [facade guards](../../../../../src/lib.rs), [website recipe](../guides/website.md).
