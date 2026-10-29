//! Compose only example routes into the existing native HTTP host; construction binds nothing.
use crate::{
    options::Options,
    panel::{self, Panel},
    setup,
};
use leptos::prelude::*;
use stackpress_framework_view::BasePath;
use stackpress_leptos::components::{ADVANCED_STYLES, BREADTH_STYLES, CORE_STYLES};
use stackpress_lib::{Error, Flow, Result, handler};
use stackpress_server::{Server, ServerBuilder};
use stackpress_server_http::{AssetBundle, AssetLimits, HttpServer, NetworkLimits, NetworkOptions};
use std::{
    fs,
    io::{Read, Write},
    sync::Arc,
    time::Duration,
};

/// Immutable composed host and its explicitly closable SQLite resource.
pub struct Gateway {
    /// Existing deployment host used by both injected IO and real devmetrics accepts.
    pub http: Arc<HttpServer>,
    /// Built application for bounded buffered handler tests.
    pub app: Arc<Server>,
    /// Review-only controls, deliberately absent from other example graphs.
    pub panel: Arc<Panel>,
    /// The exact setup fixture's pool; close after host drain.
    pub engine: Arc<stackpress_sql::Engine>,
}
impl Gateway {
    /// Attach existing fixtures/assets and prepare the entire host without opening network IO.
    /// # Errors
    /// Refuses missing setup/assets, invalid hashes, routes and startup policy.
    pub async fn new(options: &Options) -> Result<Self> {
        if options.variant != env!("STACKPRESS_REVIEW_VARIANT") {
            return Err(Error::invalid(
                "review executable and bundle variant disagree",
            ));
        }
        stackpress_framework_view::leptos::render::initialize_ssr().map_err(Error::from_source)?;
        let path = setup::attach(&options.directory)?;
        let (engine, acknowledgement) = generated_admin_web::database::open_controlled(&path)
            .await
            .map_err(Error::from_source)?;
        let panel = Arc::new(Panel::new(acknowledgement));
        let built = application(options, engine.clone(), panel.clone()).await;
        let app = match built {
            Ok(app) => Arc::new(app),
            Err(error) => {
                let _ = engine.close().await;
                return Err(error);
            }
        };
        let mut network = NetworkOptions::http(
            ([127, 0, 0, 1], options.port).into(),
            format!("{}/", options.origin()),
        );
        network.http.response_limits.max_bytes = 64 * 1024 * 1024;
        network.limits = NetworkLimits {
            max_connections: 8,
            max_handshakes: 4,
            lifecycle_timeout: Duration::from_secs(8),
            response_timeout: Duration::from_secs(10),
            shutdown_grace: Duration::from_secs(2),
            ..Default::default()
        };
        let http = Arc::new(HttpServer::from_options(app.clone(), network)?);
        Ok(Self {
            http,
            app,
            panel,
            engine,
        })
    }
}
/// Register scoped example mounts, then the review-only policy and assets.
async fn application(
    options: &Options,
    engine: Arc<stackpress_sql::Engine>,
    panel: Arc<Panel>,
) -> Result<Server> {
    let mut builder = ServerBuilder::new();
    stackpress_example_ui_gallery_web::configure(
        &mut builder,
        BasePath::new(options.path("/ui-gallery")).map_err(Error::from_source)?,
        options.csr,
        options.origin(),
    )?;
    stackpress_example_ui_preferences_web::configure(
        &mut builder,
        BasePath::new(options.path("/ui-preferences")).map_err(Error::from_source)?,
        options.csr,
        false,
        options.origin(),
    )?;
    generated_admin_web::host::configure(
        &mut builder,
        engine,
        generated_admin_web::host::Config {
            mount: options.path("/generated-admin/"),
            csrf: "review-local-csrf".into(),
            csr: options.csr,
            transfer_limits: Default::default(),
        },
        Arc::new(generated_admin_web::host::FixturePolicy),
        options.origin(),
    )?;
    stackpress_example_ui_stream_web::configure(
        &mut builder,
        &options.path("/ui-stream"),
        panel.probe.clone(),
    )?;
    panel::configure(&mut builder, options, panel)?;
    pages(&mut builder, options)?;
    assets(&mut builder, options)?;
    builder.build().await.map_err(|error| error.error)
}
/// Verify existing manifest identities after prefixing only routes with the trusted mount.
fn assets(builder: &mut ServerBuilder, options: &Options) -> Result<()> {
    let mode = if options.csr { "csr" } else { "hydrate" };
    for name in ["gallery", "preferences", "admin", "format", "stream"] {
        let selected = if name == "stream" { "hydrate" } else { mode };
        let root = options
            .directory
            .join("bundle")
            .join(&options.variant)
            .join(name)
            .join(selected);
        let manifest_path = root.join("manifest.json");
        let metadata = fs::symlink_metadata(&manifest_path).map_err(Error::from_source)?;
        if !metadata.is_file() {
            return Err(Error::invalid("review manifest must be regular"));
        }
        let mut bytes = Vec::new();
        fs::File::open(&manifest_path)
            .map_err(Error::from_source)?
            .take(64 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(Error::from_source)?;
        if bytes.len() > 64 * 1024 {
            return Err(Error::limit("review manifest too large"));
        }
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(Error::from_source)?;
        let entries = manifest
            .get_mut("entries")
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| Error::invalid("review manifest entries unavailable"))?;
        for entry in entries {
            let route = entry
                .get("route")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::invalid("review manifest route unavailable"))?;
            entry["route"] = options.path(route).into();
        }
        // The library validates every descriptor/hash/file. Only this temporary descriptor
        // changes; no asset or retained manifest is modified by the runtime.
        let mut temporary = tempfile::NamedTempFile::new_in(
            std::env::temp_dir()
                .canonicalize()
                .map_err(Error::from_source)?,
        )
        .map_err(Error::from_source)?;
        temporary
            .write_all(&serde_json::to_vec(&manifest).map_err(Error::from_source)?)
            .map_err(Error::from_source)?;
        AssetBundle::load(&root, temporary.path(), AssetLimits::default())?.register(builder)?;
    }
    Ok(())
}
/// Additional example documents use the existing component sources and matching client modules.
fn pages(builder: &mut ServerBuilder, options: &Options) -> Result<()> {
    let home = options.path("/review");
    builder.get(
        &options.path("/"),
        handler(move |_, res, _| {
            let home = home.clone();
            Box::pin(async move {
                res.redirect(&home)?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    let base = options.path("/generated-admin");
    let csr = options.csr;
    builder.get(
        &format!("{base}/roles"),
        handler(move |_, res, _| {
            let base = base.clone();
            Box::pin(async move {
                let body = if csr {
                    String::new()
                } else {
                    let owner = Owner::new();
                    let body = owner.with(|| view! {<generated_admin_shared::Admin/>}.to_html());
                    owner.cleanup();
                    body
                };
                res.html(generated_admin_web::render::roles_document(
                    &body,
                    &format!("{base}/assets/start.js"),
                ))?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    for (suffix, locale) in [
        ("/", None),
        ("/edit/en", Some("en")),
        ("/edit/fr", Some("fr")),
    ] {
        let base = options.path("/ui-format");
        let csr = options.csr;
        builder.get(
            &format!("{base}{suffix}"),
            handler(move |_, res, _| {
                let base = base.clone();
                Box::pin(async move {
                    let body = if csr {
                        String::new()
                    } else if let Some(locale) = locale {
                        stackpress_example_ui_format::editing::render(if locale == "fr" {
                            stackpress_leptos::format::DisplayLocale::French
                        } else {
                            stackpress_leptos::format::DisplayLocale::English
                        })
                    } else {
                        stackpress_example_ui_format::render_html()
                            .map_err(|e| Error::invalid(e.to_string()))?
                    };
                    let attr = locale
                        .map(|locale| format!(" data-editor-locale=\"{locale}\""))
                        .unwrap_or_default();
                    res.html(document(
                        "Formatting",
                        &body,
                        &format!("{base}/assets/start.js"),
                        &attr,
                    ))?;
                    Ok(Flow::Continue)
                })
            }),
        )?;
    }
    Ok(())
}
/// Fixed review shell; only compiled text/component output is inserted here.
fn document(title: &str, body: &str, module: &str, attr: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title}</title><style>{CORE_STYLES}{ADVANCED_STYLES}{BREADTH_STYLES}*{{box-sizing:border-box}}body{{font:16px system-ui;margin:1rem auto;padding:1rem;max-width:70rem}}input,select,textarea{{max-width:100%}}section,fieldset{{min-width:0}}.admin-table-scroll{{overflow:auto}}pre,output{{overflow-wrap:anywhere;white-space:pre-wrap}}</style></head><body><div id=\"app\"{attr}>{body}</div><script type=\"module\" src=\"{module}\"></script></body></html>"
    )
}
