//! Finite SQLite-backed account/auth host. Writes review artifacts and exits; opens no port.
use serde_json::json;
use stackpress::framework_view as stackpress_framework_view;
use stackpress_framework_session::{
    Service, account,
    auth::{Environment, Hooks},
    columns::Record,
    presentation::Presentation,
};
use stackpress_lib::{Body, Flow, Request, Response, Result, queue::task};
use stackpress_server::{ConfigLayers, ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{path::Path, sync::Arc};

/// Replay the existing 16-page SQL-backed matrix through the full aggregate registrar.
pub async fn run(out: &Path) -> Result<()> {
    std::fs::create_dir_all(out)?;
    let temp = tempfile::tempdir()?;
    let engine = Arc::new(Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(temp.path().join("account.sqlite")).create_if_missing(true),
        )
        .await
        .map_err(stackpress_lib::Error::from_source)?,
    ));
    let actions = stackpress_framework_session::storage::AuthActions::new(
        engine.clone(),
        stackpress_framework_session::storage::AuthStore::new(
            "DATABASE-SEED-MUST-NOT-REACH-BROWSER".into(),
            0,
        ),
    );
    let account = stackpress_framework_session::storage::AccountActions::<
        stackpress_framework_session::storage::DefaultModels,
    >::new(
        engine.clone(),
        "DATABASE-SEED-MUST-NOT-REACH-BROWSER".into(),
        0,
        Default::default(),
    );
    account
        .profile()
        .install()
        .await
        .map_err(stackpress_lib::Error::from_source)?;
    actions
        .install()
        .await
        .map_err(stackpress_lib::Error::from_source)?;
    let input: Record = serde_json::from_value(
        json!({"name":"Ada Lovelace","username":"ada-user","email":"ada@example.test","phone":"+639171234567","secret":"correct","roles":["USER"]}),
    )?;
    let profile = account
        .signup(&input)
        .await
        .map_err(stackpress_lib::Error::from_source)?;
    actions.create(&serde_json::from_value(json!({"profileId":profile["id"],"type":"2fa","token":"JBSWY3DPEHPK3PXP","secret":"JBSWY3DPEHPK3PXP"}))?).await.map_err(stackpress_lib::Error::from_source)?;
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers.value("finite",json!({"session":{"seed":"SIGNING-SEED-MUST-NOT-REACH-BROWSER"},"auth":{"base":"/auth","password":{"min":8,"upper":true,"number":true},"2fa":{"issuer":"Stackpress"},"email":{"name":"Stackpress","address":"sender@example.test"},"menu":[{"name":"Email","icon":"envelope","path":"/auth/signin/email"},{"name":"Username","icon":"user","path":"/auth/signin/username"},{"name":"Phone","icon":"phone","path":"/auth/signin/phone"},{"type":"footer","name":"Create an account","path":"/auth/signup"}]},"email":{"jsonTransport":true,"auth":{"user":"fixture","pass":"SMTP-PASSWORD-MUST-NOT-REACH-BROWSER"}}}))?;
    app.config_layers(layers)?;
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(Arc::new(Environment {
                database_seed: Some("DATABASE-SEED-MUST-NOT-REACH-BROWSER".into()),
                email_configured: true,
                host: Some("https://session.invalid".into()),
                ..Default::default()
            }))?;
            let mut presentation = Presentation::default();
            presentation.config.logo = None;
            presentation.assets.theme_bootstrap = Some(
                stackpress_framework_view::theme::bootstrap_location()
                    .map_err(stackpress_lib::Error::from_source)?,
            );
            start.register(Arc::new(presentation))?;
            Ok(Flow::Continue)
        })
    }))?;
    // The aggregate owns source auth/email/identity order and the default CSRF service.
    app.plugin("framework", register)?;
    let server = Arc::new(app.build().await.map_err(|error| error.error)?);
    server
        .host()
        .service::<Hooks>()?
        .bind(&server, tokio::runtime::Handle::current());
    let token = server
        .host()
        .service::<Service>()?
        .create(
            json!({"id":profile["id"],"name":"Ada Lovelace","roles":["USER"]})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
        .map_err(stackpress_lib::Error::from_source)?;
    let mut artifacts = Vec::new();
    let pages = [
        ("signup", "/auth/signup"),
        ("signin", "/auth/signin"),
        ("username", "/auth/signin/username"),
        ("email", "/auth/signin/email"),
        ("phone", "/auth/signin/phone"),
        ("otp", "/auth/signin/otp/missing/wrong"),
        ("link", "/auth/signin/link/missing/wrong"),
        ("two-factor", "/auth/signin/2fa/missing/missing/wrong"),
        ("account", "/auth/account"),
        ("update", "/auth/account/update"),
        ("security", "/auth/account/security"),
        ("password", "/auth/account/security/password"),
        ("export", "/auth/account/security/export"),
        ("remove", "/auth/account/security/remove"),
        ("enroll", "/auth/account/security/2fa"),
        ("remove-factor", "/auth/account/security/2fa/remove"),
    ];
    for (name, path) in pages {
        let mut req = Request::builder()
            .url(format!("https://session.invalid{path}"))
            .build()?;
        if path.contains("/account") {
            req.session.set("session", token.clone(), None)?;
        }
        let mut res = Response::new();
        server.handle_route(&mut req, &mut res).await?;
        let Body::Text(html) = res.body() else {
            return Err(stackpress_lib::Error::new(format!("{name} did not render")));
        };
        assert!(html.contains("<html"));
        assert!(!html.contains("SIGNING-SEED-MUST-NOT-REACH-BROWSER"));
        assert!(!html.contains("DATABASE-SEED-MUST-NOT-REACH-BROWSER"));
        assert!(!html.contains("SMTP-PASSWORD-MUST-NOT-REACH-BROWSER"));
        std::fs::write(out.join(format!("{name}.html")), html)?;
        // Review both explicit choices from the same SQL-loaded public snapshot.
        let seed_text = html
            .split(r#"id="stackpress-page">"#)
            .nth(1)
            .and_then(|text| text.split("</script>").next())
            .ok_or_else(|| stackpress_lib::Error::invalid("missing seed"))?;
        let snapshot: stackpress_framework_view::PageSnapshot<(
            stackpress_framework_session::views::Page,
            stackpress_framework_session::views::PageProps,
        )> = stackpress_framework_view::PageSnapshot::decode(seed_text, 1, "session-v1")
            .map_err(stackpress_lib::Error::from_source)?;
        for theme in ["light", "dark"] {
            let mut props = snapshot.data.1.clone();
            props.theme = Some(theme.into());
            let themed = stackpress_framework_session::presentation::render(
                snapshot.data.0,
                props,
                server.host().service::<Presentation>()?,
            )
            .await?;
            std::fs::write(out.join(format!("{name}-{theme}.html")), themed)?;
            // Exercise long source-shaped values through the same SSR and browser DTO.
            let mut long = snapshot.data.1.clone();
            long.theme = Some(theme.into());
            let text = "長いアカウント名<&>".repeat(24);
            long.config.brand = Some(text.clone());
            if let Some(menu) = long.config.menu.first_mut() {
                menu.name = text.clone();
            }
            long.error = format!("Source-shaped error: {text}");
            long.code = 400; // The checked view contract pairs an error message with an error status.
            long.results["name"] = json!(text);
            long.results["token"] = json!(format!("{}@example.test", "long".repeat(40)));
            if long.results.get("auth").is_some() {
                long.results["auth"]["email"]["token"] = json!("long".repeat(64));
            }
            long.input["name"] = json!(text);
            let document = stackpress_framework_session::presentation::render(
                snapshot.data.0,
                long,
                server.host().service::<Presentation>()?,
            )
            .await?;
            std::fs::write(out.join(format!("{name}-long-{theme}.html")), document)?;
        }
        artifacts.push(json!({"page":name,"path":path,"code":res.status().map_or(0,|s|s.code)}));
    }
    // Exercise the real Lane B event through the frozen registrar, entirely in JSON transport.
    let email=server.resolve("email-send",json!({"from":"sender@example.test","to":"ada@example.test","subject":"Finite account host","text":"Local JSON transport proof"})).await?;
    assert_eq!(email.response.code, Some(200));
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({"pages":artifacts,"email":email.response.code}))?,
    )?;
    server.host().service::<Hooks>()?.shutdown().await;
    actions
        .engine
        .close()
        .await
        .map_err(stackpress_lib::Error::from_source)?;
    println!(
        "Finite SQLite host rendered {} pages and real email-send JSON transport",
        pages.len()
    );
    Ok(())
}

/// Explicit source metadata/catalog; no dynamic module loading or registration-time IO.
fn register(app: &mut ServerBuilder) -> Result<()> {
    static CLIENT: stackpress::framework::CompiledClient = stackpress::framework::CompiledClient {
        bindings: stackpress::framework::FrameworkBindings {
            session: None,
            api: None,
        },
        build: "slice11-session-matrix",
        describe: || vec![],
        models: &[stackpress::framework::CompiledModel {
            name: "Account",
            listen: Some(account::events::register_listen),
            admin: None,
        }],
    };
    stackpress::framework::register_with(app, &CLIENT)
}
