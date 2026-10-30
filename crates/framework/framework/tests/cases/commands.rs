//! Original event traces and native command input/skip/error boundaries.

use super::*;
use stackpress_framework::{commands::schema, config::Environment};

/// A finite binding exposes computed options instead of listening or spawning children.
async fn options(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if let Some(result) = req
        .data
        .get("result")
        .and_then(|v| v.as_shared())
        .and_then(|v| v.downcast_ref::<stackpress_sql::QueryResult>())
    {
        // The host chooses its console projection; the aggregate retains native row types.
        assert!(req.data.to_json().is_err());
        res.results(&result.rows[0].try_get::<String>("answer").unwrap())?;
        return Ok(Flow::Continue);
    }
    res.results(&req.data.to_json()?)?;
    Ok(Flow::Continue)
}

/// Publish native bindings only; commands stay untouched until explicit invocation.
pub(super) fn bind(app: &mut ServerBuilder, environment: Environment) {
    app.on_config(task(move |start: &mut StartupContext| {
        let environment = environment.clone();
        Box::pin(async move {
            let mut bindings = Bindings::default();
            for name in ["serve", "develop", "generate", "output"] {
                bindings.bind(name, options);
            }
            start.register(Arc::new(bindings))?;
            start.register(Arc::new(environment))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
}

/// Source error skips differ from develop's intentionally unconditional execution.
#[tokio::test]
async fn source_prior_status_modes_and_public_errors() {
    let oracle: Value =
        serde_json::from_str(include_str!("../fixtures/source-commands.json")).unwrap();
    let mut app = application(json!({}));
    bind(&mut app, Environment::default());
    let host = app.build().await.unwrap();
    for case in oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["variant"] == "prior-error" && case["command"] != "build")
    {
        let command = case["command"].as_str().unwrap();
        let mut req = Request::new();
        let mut res = Response::new();
        res.status_code(409, None).unwrap();
        host.emit(command, &mut req, &mut res).await.unwrap();
        assert_eq!(
            res.status().unwrap().code,
            u16::try_from(case["code"].as_u64().unwrap()).unwrap(),
            "{command}"
        );
    }
    for command in [
        "install",
        "migrate",
        "purge",
        "push",
        "query",
        "uninstall",
        "upgrade",
    ] {
        let output = host.resolve(command, json!({})).await.unwrap();
        assert_eq!(output.response.error.as_deref(), Some("No database found"));
        assert_eq!(output.response.code, Some(500));
    }
    let missing = host.resolve("emit", json!({})).await.unwrap();
    assert_eq!(
        missing.response.error.as_deref(),
        Some("Missing event name")
    );
    assert_eq!(
        host.resolve("generate", json!({}))
            .await
            .unwrap()
            .response
            .code,
        None
    );
    for config in [
        json!({"client":{}}),
        json!({"client":null}),
        json!({"client":{"tsconfig":""}}),
    ] {
        let output = application(config)
            .build()
            .await
            .unwrap()
            .resolve("generate", json!({}))
            .await
            .unwrap();
        assert_eq!(
            output.response.error.as_deref(),
            Some("Missing tsconfig path")
        );
    }
}

/// Source defaults, supplied override paths and nullish develop defaults reach finite providers.
#[tokio::test]
async fn native_options_and_generation_precedence() {
    let mut app =
        application(json!({"client":{"tsconfig":"project.json"},"cli":{"idea":"configured.idea"}}));
    bind(
        &mut app,
        Environment {
            cwd: "/app".into(),
            loader_cwd: "/loader".into(),
            ..Default::default()
        },
    );
    let host = app.build().await.unwrap();
    assert_eq!(
        host.host().service::<Client>().unwrap().module(),
        "stackpress-client"
    );
    assert_eq!(
        host.resolve("serve", json!({}))
            .await
            .unwrap()
            .response
            .results
            .unwrap()
            .to_json()
            .unwrap(),
        json!({"host":"127.0.0.1","port":3000})
    );
    let generated = host
        .resolve("generate", json!({"input":"long.idea","i":"short.idea"}))
        .await
        .unwrap()
        .response
        .results
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(
        generated,
        json!({"idea":"short.idea","build":"/app/node_modules/stackpress-client","tsconfig":"project.json","lang":"js","prettier":{}})
    );
    assert_eq!(
        host.resolve("generate", json!({}))
            .await
            .unwrap()
            .response
            .results
            .unwrap()
            .to_json()
            .unwrap()["idea"],
        "configured.idea"
    );
    let developed = host
        .resolve("develop", json!({}))
        .await
        .unwrap()
        .response
        .results
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(developed["debounce"], 100);
    assert_eq!(developed["extensions"], json!([".ts"]));
    assert_eq!(developed["timeout"], 10000);
    let mut app = application(
        json!({"server":{"host":"source.test","port":4100,"develop":{"debounce":0,"extensions":[],"ignore":null,"timeout":null,"watch":[]}},"client":{"module":"static-module","tsconfig":"config","build":"out","lang":"ts","prettier":{"semi":false}}}),
    );
    bind(&mut app, Environment::default());
    let host = app.build().await.unwrap();
    assert_eq!(
        host.host().service::<Client>().unwrap().module(),
        "static-module"
    );
    assert_eq!(
        host.resolve("serve", json!({}))
            .await
            .unwrap()
            .response
            .results
            .unwrap()
            .to_json()
            .unwrap()["host"],
        "source.test"
    );
    let developed = host
        .resolve("develop", json!({}))
        .await
        .unwrap()
        .response
        .results
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(developed["debounce"], 0);
    assert_eq!(developed["ignore"], json!([]));
    assert_eq!(developed["timeout"], 10000);
    let generated = host
        .resolve("generate", json!({}))
        .await
        .unwrap()
        .response
        .results
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(generated["idea"], "schema.idea");
    assert_eq!(generated["lang"], "ts");
    assert_eq!(generated["build"], "out");
}

/// Emit uses the original argv tail and raw nested event chain, then explicit output ownership.
#[tokio::test]
async fn emit_reuses_direct_events_and_surfaces_provider_errors() {
    let mut app = application(json!({}));
    bind(
        &mut app,
        Environment {
            argv: ["node", "cli", "emit", "echo", "--name", "Ada"]
                .map(String::from)
                .into(),
            ..Default::default()
        },
    );
    app.on("echo", options).unwrap();
    let host = app.build().await.unwrap();
    let output = host
        .resolve("emit", json!({}))
        .await
        .unwrap()
        .response
        .results
        .unwrap()
        .to_json()
        .unwrap();
    let nested: Value = serde_json::from_str(output["text"].as_str().unwrap()).unwrap();
    assert_eq!(nested["results"], json!({"name":"Ada"}));
    let mut direct = ServerBuilder::new();
    stackpress_framework::config::register(&mut direct).unwrap();
    stackpress_framework::commands::register_server(&mut direct).unwrap();
    let direct = direct.build().await.unwrap();
    assert!(direct.resolve("serve", json!({})).await.is_err());
    let mut req = Request::new();
    let mut res = Response::new();
    res.status_code(409, None).unwrap();
    assert_eq!(
        direct.emit("serve", &mut req, &mut res).await.unwrap(),
        stackpress_lib::DispatchOutcome::Completed
    );
}

/// Typed native string errors occur at use, not at registration or an unrelated skipped command.
#[test]
fn generation_direct_options_report_native_string_boundary() {
    use stackpress_framework::config::{CliOptions, ClientOptions, ServerOptions};
    let req = Request::new();
    let env = Environment::default();
    assert!(
        schema::options(
            &req,
            &ClientOptions(json!({"build":3})),
            &ServerOptions::default(),
            &CliOptions::default(),
            &env
        )
        .is_err()
    );
}
