# Configuration and presence

Configuration supplies source values; executable selection and native resources remain compiled/application-owned. The aggregate calls `config::register` once before its stage callbacks. It claims only the typed roots needed by the compiled leaves.

## Source roots and process inputs

Transparent ServerOptions, ClientOptions, DatabaseOptions and CliOptions wrap serde_json::Value and default to an empty object. Environment captures application process inputs explicitly; registration does not read argv, cwd or terminal state.

| Surface | Defaults / use |
| --- | --- |
| Environment.argv | Empty; caller supplies executable and entrypoint at indices 0/1 before source command arguments. |
| Environment.cwd / loader_cwd | Empty PathBuf defaults; explicit native working directories. |
| Environment.force | false; source force flag supplied by caller. |
| server | host 127.0.0.1, port 3000 when used by serve; develop nullish defaults; cwd/build for generation/view. |
| client | module stackpress-client; revisions/build/package/tsconfig/lang/prettier used lazily by relevant operations. |
| database | migrations output and populate event/data pairs; does not construct an Engine. |
| cli | idea input path; overridden by request input then i. |
| language | Source key locale, fallback en_US, exact insertion-ordered language keys. |
| view | Explicit key presence activates; mode defaults production, noview flag defaults json, props/engine supplied as source values. |

Selected server-commands/schema-client/view claim server; schema-client/sql claim client; schema-client claims cli; sql claims database. Unclaimed input sections retain the server's existing validation rules. Explicitly register only the corresponding roots when directly composing adapters. [Config source](../../src/native/config.rs) and [input tests](../../tests/cases/commands.rs) pin these rules.

## Presence differs from value

The server's ConfigPresence records explicit merged section keys, not whether a value is truthy. For view, missing is inactive while null/false/empty object are present and activate the source plugin. This follows source `config.has('view')`; it does not alter layering/default decoding. Server defaults are not explicit presence. Layers add/replace keys; replacing a key with null still leaves it present.

Language instead tests source truthiness: missing/null/false/zero/empty string deactivate; arrays and objects are truthy. Session/access/auth missing-config behaviour is owned by those registrars, with no aggregate replacement gate. [Presence tests](../../../../server/server/tests/config_presence.rs), [view tests](../../tests/cases/view.rs) and [language tests](../../tests/cases/language.rs) distinguish these contracts.

## Resolve generation input

This pure full-graph example computes options without loading a file or generating output.

<!-- example: full -->
```rust
use serde_json::json;
use stackpress_framework::{commands::schema, config::*};
use stackpress_lib::{Request, Result};

pub fn generation_options() -> Result<()> {
    let req = Request::builder()
        .data(json!({"input":"long.idea","i":"short.idea"}))
        .build()?;
    let options = schema::options(
        &req,
        &ClientOptions(json!({"tsconfig":"project.json"})),
        &ServerOptions(json!({"cwd":"/app"})),
        &CliOptions(json!({"idea":"configured.idea"})),
        &Environment {
            loader_cwd: "/loader".into(),
            ..Default::default()
        },
    )?;
    assert_eq!(options.idea, "short.idea");
    assert_eq!(
        options.build,
        std::path::PathBuf::from("/app/node_modules/stackpress-client")
    );
    Ok(())
}
```

The short request option wins over input, then cli.idea, then loader_cwd/schema.idea. This pure resolver does not enforce the event wrapper's missing-client/tsconfig response checks. String-required values fail at their typed native boundary when used. See [generation reference](codegen.md) and [command errors](commands.md).
