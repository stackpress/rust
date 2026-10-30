//! Source view configuration, render skips, development middleware and finite build orchestration.

mod packaging;
pub use packaging::{Package, PackageOutcome, PackagePaths, Packaging};

use super::config::{Environment, ServerOptions, path, string};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result, Status};
use stackpress_server::{ConfigPresence, ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::Arc;

/// Optional source view section, distinct from preferences/flash admission configuration.
#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Section(pub Option<Value>);

/// Present native view configuration; not a renderer or a browser snapshot.
pub struct Settings {
    /// Original section, kept as configuration rather than implicitly projected public data.
    pub config: Value,
    /// Source server mode, default production; every other string selects development.
    pub mode: String,
    /// Source server.cwd, resolved against application-supplied process inputs.
    pub cwd: String,
}

/// Compiled view entries in the original application's registration order.
#[derive(Default)]
pub struct Catalog {
    /// Native module identities; never imported from caller-controlled data.
    pub entries: Vec<String>,
}

/// Trusted adapter marker for native HTTP middleware, absent on raw/library/IPC calls.
///
/// The execution adapter owns the native request/response; it supplies this marker only
/// when both native resources exist. Browser data cannot construct adapter context.
pub struct NativeHttp;

/// Owned native provider work; cancellation drops the selected operation's owned state.
pub type NativeFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;

/// Owned renderer call; a closure may capture this host's actual native renderer.
pub type Render = Arc<
    dyn Fn(String, Value) -> NativeFuture<Option<stackpress_framework_view::RenderedPage>>
        + Send
        + Sync,
>;

/// Explicit compiled native render provider; Leptos SSR implementations can use existing renderers.
pub struct Provider {
    /// No render is called while registering or building a host.
    pub render: Render,
}

/// One native compiler stage over owned options, with no retained request/Invocation borrow.
///
/// ```compile_fail
/// use std::{future::Future,pin::Pin,rc::Rc};
/// use serde_json::Value;
/// fn invalid(_:Value)->Pin<Box<dyn Future<Output=stackpress_lib::Result<Value>>+Send>> {
///     let native=Rc::new(1);
///     Box::pin(async move {drop(native);Ok(Value::Null)})
/// }
/// ```
pub type Build = Arc<dyn Fn() -> NativeFuture<Value> + Send + Sync>;

/// One command-owned compiler engine, never a process-global active build.
pub struct BuildEngine {
    /// Load each compiled view entry before any output-path check, retaining source errors.
    pub set: Arc<dyn Fn(String) -> NativeFuture<()> + Send + Sync>,
    /// Actual loaded engine size, including the provider's entry de-duplication behavior.
    pub size: Arc<dyn Fn() -> usize + Send + Sync>,
    /// Build all loaded browser clients.
    pub clients: Build,
    /// Build all loaded assets/styles.
    pub assets: Build,
    /// Build all loaded server pages.
    pub pages: Build,
}

/// Explicit native compiler factory, called per command with source cwd/engine options.
pub struct BuildProvider {
    /// Construct command-owned IO callbacks; registration never invokes this function.
    pub create: Arc<dyn Fn(Value) -> NativeFuture<BuildEngine> + Send + Sync>,
}

/// Queue only the present-view service and command; do not select a renderer or open assets.
/// # Errors
/// Retains typed-section and startup registration failures; configuration errors surface at build.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("view", Section::default(), |_| Ok(()))?;
    app.on_config(configure)?;
    app.on_listen(listen)?;
    app.on_route(routes)?;
    Ok(())
}

/// Publish real settings only when source configuration exists.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    if start.config::<ConfigPresence>()?.contains("view") {
        let config = start.config::<Section>()?.0.clone().unwrap_or(Value::Null);
        let server = &start.config::<ServerOptions>()?.0;
        let env = start.service::<Environment>().cloned().unwrap_or_default();
        let mode = string(path(server, "mode"), "production")?;
        let cwd = string(path(server, "cwd"), &env.cwd.to_string_lossy())?;
        start.register(Arc::new(Settings { config, mode, cwd }))?;
    }
    Ok(Flow::Continue)
}

/// Source build command is present even when the explicit view value is null or false.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    if start.config::<ConfigPresence>()?.contains("view") {
        super::commands::view_listen(start)?;
    }
    Ok(Flow::Continue)
}

/// Development hooks occur in Route, after earlier source Listen hooks.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    if let Ok(settings) = start.service::<Settings>()
        && settings.mode != "production"
    {
        start.on_request(development)?;
    }
    Ok(Flow::Continue)
}

/// Source middleware only handles native HTTP resources; an explicit provider owns asset/HMR IO.
/// # Errors
/// Propagates native middleware failures. A provider Abort preserves its headers-sent stop outcome.
pub async fn development(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if inv.adapter_context::<NativeHttp>().is_none()
        || inv.service::<Settings>()?.mode == "production"
    {
        return Ok(Flow::Continue);
    }
    inv.shared_service::<super::commands::Bindings>()?
        .invoke("view-http", req, res, inv)
        .await
}

/// Render the source page props after its exact redirect/flag/string-body skips.
/// # Errors
/// Retains me-event, serializable-data and native renderer failures without retrying.
pub async fn render(
    action: &str,
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let settings = inv.service::<Settings>()?;
    let status = Status::from_code(
        res.status()
            .map_or(200, |s| if s.code == 0 { 200 } else { s.code }),
    );
    res.status_code(status.code, Some(status.message.to_string()))?;
    let noview = string(path(&settings.config, "noview"), "json")?;
    if res.redirected() || req.data.has(&noview) || matches!(res.body(), Body::Text(_)) {
        return Ok(Flow::Continue);
    }
    let mut identity = Response::new();
    let _ = inv.emit("me", req, &mut identity).await?;
    let mut data = path(&settings.config, "props")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Some(response) = res.data().to_json()?.as_object() {
        data.extend(response.clone());
    }
    // These are source page values, not the native settings/catalog/provider graph.
    let view = req.view();
    let url = &view.url;
    let headers: serde_json::Map<String, Value> = view
        .headers
        .entries()
        .map(|(key, values)| (key.clone(), Value::String(values.join(", "))))
        .collect();
    let session: serde_json::Map<String, Value> = view
        .session
        .entries()
        .map(|(key, value)| {
            (
                key.clone(),
                match value {
                    stackpress_lib::SessionValue::Text(text) => Value::String(text.clone()),
                    stackpress_lib::SessionValue::List(values) => serde_json::json!(values),
                },
            )
        })
        .collect();
    // Keep source field/insertion order and conversions explicit; no native service enters props.
    let props = serde_json::json!({
        "data": data,
        "session": identity.result().map(|v| v.to_json()).transpose()?,
        "request": {
            "url": {
                "hash": url.fragment().map(|v| format!("#{v}")).unwrap_or_default(),
                "host": url.host_str().map(|host| url.port().map_or_else(
                    || host.to_owned(),
                    |port| format!("{host}:{port}")
                )).unwrap_or_default(),
                "hostname": url.host_str(),
                "href": url.as_str(),
                "origin": url.origin().ascii_serialization(),
                "pathname": url.path(),
                "port": url.port().map(|v| v.to_string()).unwrap_or_default(),
                "protocol": format!("{}:", url.scheme()),
                "search": url.query().map(|v| format!("?{v}")).unwrap_or_default()
            },
            "headers": headers,
            "session": session,
            "method": view.method.as_str(),
            "mime": view.mimetype,
            "data": view.data.to_json()?
        },
        "response": res.try_to_json()?
    });
    let provider = inv.shared_service::<Provider>()?;
    if let Some(html) = (provider.render)(action.into(), props).await? {
        res.html(html.into_html())?;
        res.status_code(status.code, Some(status.message.to_string()))?;
    }
    Ok(Flow::Continue)
}

/// Run selected build stages in source client→asset→page order, omitting empty catalogs.
/// # Errors
/// Propagates the first explicit build provider failure and does not start a compiler at startup.
pub async fn build(inv: &Invocation<'_>) -> Result<Vec<Value>> {
    let settings = inv.service::<Settings>()?;
    let provider = inv.service::<BuildProvider>()?;
    // The factory receives owned source options before catalog loading or output-path decisions.
    let engine = (provider.create)(serde_json::json!({
        "cwd": settings.cwd,
        "engine": path(&settings.config, "engine").cloned().unwrap_or(serde_json::json!({}))
    }))
    .await?;
    if let Ok(catalog) = inv.service::<Catalog>() {
        for entry in &catalog.entries {
            (engine.set)(entry.clone()).await?;
        }
    }
    if (engine.size)() == 0 {
        return Ok(Vec::new());
    }
    let mut results = Vec::new();
    // Pair each source output-path key with its actual compiler; ordering is part of the result.
    for (key, compiler) in [
        ("clientPath", &engine.clients),
        ("assetPath", &engine.assets),
        ("pagePath", &engine.pages),
    ] {
        // Source skips only an absent/falsy path, after every catalog entry was loaded above.
        if !path(&settings.config, &format!("engine.{key}")).is_some_and(super::config::truthy) {
            continue;
        }
        let mut result = compiler().await?;
        if let Some(contents) = result.get("contents").and_then(Value::as_str) {
            // Source substring counts UTF-16 code units. Rust replaces an isolated
            // cut surrogate with U+FFFD because Rust strings cannot contain it.
            let units: Vec<_> = contents.encode_utf16().take(100).collect();
            result["contents"] = format!("{} ...", String::from_utf16_lossy(&units)).into();
        }
        results.push(result);
    }
    Ok(results)
}

/// Source Build ignores an earlier error, then submits optional packaging after its build script.
/// # Errors
/// Propagates build errors. Packaging completion/failure belongs only to its supervision handle,
/// matching the source's unawaited build.ts:21 call. No packaging callback means no packaging IO.
pub async fn build_event(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    build(&inv).await?;
    if let Ok(packaging) = inv.service::<Packaging>() {
        let settings = inv.service::<Settings>()?;
        let server = &inv.config::<ServerOptions>()?.0;
        let default = std::path::Path::new(&settings.cwd).join(".build");
        let destination = path(server, "build")
            .cloned()
            .unwrap_or_else(|| default.to_string_lossy().into_owned().into());
        // No task borrows Invocation/response; path/callback errors stay in supervision.
        packaging.submit(settings.cwd.clone(), destination);
    }
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}
