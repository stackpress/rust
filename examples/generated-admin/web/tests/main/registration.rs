//! Real generated registration, explicit dependencies, mounts and direct-library alternatives.
use generated_admin_web::{generated::pages_host, host};
use stackpress_framework_sql::{
    operations::{Policy, Registration},
    plugin::{self, Contributions},
};
use stackpress_lib::{Flow, queue::task};
use stackpress_server::{PluginSpec, ServerBuilder, StartupContext};
use stackpress_sql::{DialectId, Engine, Sql};
use std::sync::Arc;

/// Supply an already opened empty database; only the application owns opening/closing it.
async fn fixture(mount: bool) -> (ServerBuilder, Arc<Engine>, tempfile::TempDir) {
    let directory = tempfile::tempdir_in("/private/tmp").unwrap();
    let engine = generated_admin_web::database::open(&directory.path().join("empty.sqlite"), false)
        .await
        .unwrap();
    let retained = engine.clone();
    let mut app = ServerBuilder::new();
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = retained.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(Arc::new(Policy(Arc::new(host::FixturePolicy))))?;
            if mount {
                start.register(Arc::new(pages_host::Mount::new("/admin/")?))?;
            }
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    (app, engine, directory)
}
/// Empty SQLite catalog is observable proof that build performed no schema setup or seeding.
async fn assert_empty(engine: &Engine) {
    let rows = engine
        .fetch(&Sql::literal(
            DialectId::Sqlite,
            "SELECT name FROM sqlite_master WHERE type = 'table'",
        ))
        .await
        .unwrap();
    assert!(rows.is_empty(), "startup must not create tables");
}
/// Generated plugin and direct helpers contribute the same finite operation/route inventories.
#[tokio::test(flavor = "current_thread")]
async fn selected_and_direct_hosts_match_without_implicit_database_setup() {
    let (mut selected, engine, _root) = fixture(true).await;
    selected.plugin("sql", plugin::register).unwrap();
    selected
        .plugin_with(
            PluginSpec::new("pages").requires(&["sql"]),
            pages_host::register,
        )
        .unwrap();
    let selected = selected.build().await.unwrap();
    assert!(selected.host().service::<plugin::Service>().is_ok());
    assert!(selected.host().service::<Contributions>().is_ok());
    let (mut direct, other, _other_root) = fixture(false).await;
    direct
        .on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                pages_host::operations(start, &mut Registration::default())?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    pages_host::routes(&mut direct, "/admin/").unwrap();
    let direct = direct.build().await.unwrap();
    let inventory = |server: &stackpress_server::Server| {
        let mut entries = server
            .listeners()
            .map(|listener| (listener.pattern.clone(), listener.priority))
            .collect::<Vec<_>>();
        entries.sort();
        entries
    };
    assert_eq!(inventory(&selected), inventory(&direct));
    // Operations come from SQL Listen; routes come from the generated host's Route task.
    for listener in selected.listeners() {
        let plugin = if listener.pattern.contains("/admin/") {
            "pages"
        } else {
            "sql"
        };
        assert_eq!(listener.plugin.as_deref(), Some(plugin));
    }
    assert!(direct.listeners().all(|listener| listener.plugin.is_none()));
    assert!(
        selected.listeners().count() > 40,
        "real selected mappings and operations"
    );
    assert!(direct.host().service::<plugin::Service>().is_err());
    assert_empty(&engine).await;
    assert_empty(&other).await;
    engine.close().await.unwrap();
    other.close().await.unwrap();
}
/// Underlying Engine/Policy cannot satisfy explicit SQL integration; wrong order fails too.
#[tokio::test(flavor = "current_thread")]
async fn omitted_wrong_order_and_missing_mount_fail_before_route_or_database_effects() {
    for variant in ["omitted", "wrong-order", "mount"] {
        let (mut app, engine, _root) = fixture(variant != "mount").await;
        if variant == "mount" {
            app.plugin("sql", plugin::register).unwrap();
        }
        app.plugin("pages", pages_host::register).unwrap();
        if variant == "wrong-order" {
            app.plugin("sql", plugin::register).unwrap();
        }
        let error = app
            .build()
            .await
            .err()
            .expect("invalid dependencies must fail");
        assert_eq!(error.stage, stackpress_server::StartupStage::Config);
        assert_empty(&engine).await;
        engine.close().await.unwrap();
    }
}
/// Generated singleton and SQL contribution ownership reject aliases and competing providers.
#[tokio::test(flavor = "current_thread")]
async fn duplicate_generated_selection_and_contribution_claims_fail() {
    for duplicate in [true, false] {
        let (mut app, engine, _root) = fixture(true).await;
        if !duplicate {
            app.on_config(task(|start: &mut StartupContext| {
                Box::pin(async move {
                    start.register(Arc::new(Contributions {
                        operations: &[pages_host::operations],
                    }))?;
                    Ok(Flow::Continue)
                })
            }))
            .unwrap();
        }
        app.plugin("sql", plugin::register).unwrap();
        app.plugin("pages", pages_host::register).unwrap();
        if duplicate {
            app.plugin("pages-alias", pages_host::register).unwrap();
        }
        assert!(app.build().await.is_err());
        assert_empty(&engine).await;
        engine.close().await.unwrap();
    }
}
/// Compiling generated host modules alone contributes no operations or routes.
#[tokio::test(flavor = "current_thread")]
async fn unselected_integration_registers_nothing() {
    let (app, engine, _root) = fixture(true).await;
    let server = app.build().await.unwrap();
    assert_eq!(server.listeners().count(), 0);
    assert!(server.host().service::<plugin::Service>().is_err());
    assert!(server.host().service::<Contributions>().is_err());
    assert_empty(&engine).await;
    engine.close().await.unwrap();
}
/// Route/asset config shares a validated mount and rejects invalid input before build.
#[test]
fn invalid_mount_and_host_configuration_are_rejected() {
    for value in ["", "admin", "/../", "/admin/?q=x", "//"] {
        assert!(pages_host::Mount::new(value).is_err(), "{value}");
    }
    for value in ["/", "/admin/", "/demo/v1/"] {
        assert_eq!(pages_host::Mount::new(value).unwrap().as_str(), value);
    }
}

/// Invalid application configuration fails before any plugin callbacks or native effects.
#[tokio::test(flavor = "current_thread")]
async fn invalid_host_config_leaves_an_empty_builder_and_database() {
    for invalid in ["origin", "csrf", "limits", "mount"] {
        let (_, engine, _root) = fixture(false).await;
        let mut app = ServerBuilder::new();
        let config = host::Config {
            mount: if invalid == "mount" {
                "/../"
            } else {
                "/admin/"
            }
            .into(),
            csrf: if invalid == "csrf" { "" } else { "fixture" }.into(),
            csr: false,
            transfer_limits: stackpress_framework_admin::transfer::Limits {
                rows: if invalid == "limits" { 0 } else { 100 },
                ..Default::default()
            },
        };
        assert!(
            host::configure(
                &mut app,
                engine.clone(),
                config,
                Arc::new(host::FixturePolicy),
                if invalid == "origin" {
                    "bad"
                } else {
                    "https://admin.invalid"
                }
                .into()
            )
            .is_err()
        );
        assert_eq!(app.build().await.unwrap().listeners().count(), 0);
        assert_empty(&engine).await;
        engine.close().await.unwrap();
    }
}

/// Actual generated operation families share SQL's one marker, even without the admin composite.
#[tokio::test(flavor = "current_thread")]
async fn duplicate_actual_generated_operations_fail_during_listen() {
    let (mut app, engine, _root) = fixture(false).await;
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.register(Arc::new(Contributions {
                operations: &[pages_host::operations, pages_host::operations],
            }))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.plugin("sql", plugin::register).unwrap();
    let error = app
        .build()
        .await
        .err()
        .expect("duplicate generated operations");
    assert_eq!(error.stage, stackpress_server::StartupStage::Listen);
    assert_empty(&engine).await;
    engine.close().await.unwrap();
}
