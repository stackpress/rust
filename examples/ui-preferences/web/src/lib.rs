//! Example-only preference registration and the unchanged one-request HttpAdapter proof.
//! Callers own HTTP hosting; this library never binds a listener.
//! The opt-in fixture policy is deliberately not authentication or a deployable host.
use serde::Deserialize;
use stackpress_example_ui_preferences_shared::{Seed, registry};
use stackpress_framework_view::access::{ActionId, HostAccess, HostPolicy};
use stackpress_framework_view::preferences::LocaleRouting;
use stackpress_framework_view::{BasePath, PublicLocation, flash, preferences};
use stackpress_lib::{Error, ErrorKind, Flow, Invocation, Request, Response, Result, handler};
use stackpress_server::{
    ConfigLayers, ServerBuilder, ServerInvocationExt, StartupContext,
    http::{HttpAdapter, HttpOutput},
};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

mod policy;
mod render;
use policy::{Facts, FixturePolicy, facts};
use render::render;

/// Narrow bridge DTO. Policy selection is host environment, never this request.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    method: String,
    path: String,
    #[serde(default)]
    cookie: String,
    origin: Option<String>,
    #[serde(default)]
    body: String,
    #[serde(default)]
    caller: String,
    #[serde(default)]
    csrf: String,
    #[serde(default)]
    csr: bool,
}
/// Retain a concrete native error cause once without reflecting its Display.
fn failure(context: &'static str, cause: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::new(context)
        .with_kind(ErrorKind::External)
        .with_cause(cause)
}
/// Uniform public error; stage status after Response::json's success default.
fn denied(res: &mut Response, code: u16) -> Result<Flow> {
    res.json(&serde_json::json!({"error":"Action unavailable"}))?;
    res.status_code(code, None)?;
    Ok(Flow::Continue)
}
/// Select, consume and project before HTML; the host alone applies cookie headers.
async fn page(
    req: &mut Request,
    res: &mut Response,
    base: BasePath,
    csr: bool,
    configured: bool,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let location = req.url.query().map_or_else(
        || req.url.path().to_owned(),
        |query| format!("{}?{query}", req.url.path()),
    );
    let preferences = invocation
        .service::<preferences::Service>()?
        .action()
        .select(&location, req.headers.get("cookie").unwrap_or_default())
        .map_err(|cause| failure("select preferences", cause))?;
    let notice = invocation.service::<flash::Service>()?.notice(req)?;
    let facts = facts(req);
    let policy: Option<&dyn HostPolicy<Facts>> = configured.then_some(&FixturePolicy);
    let allowed = HostAccess::new(policy, &facts)
        .project(vec![
            ActionId::new("private.page").map_err(|cause| failure("page action", cause))?,
            ActionId::new("private.mutate").map_err(|cause| failure("mutation action", cause))?,
        ])
        .map_err(|cause| failure("project allowed actions", cause))?;
    let html = render(
        Seed {
            build: "s8-b-m1".into(),
            preferences,
            base,
            location,
            flash: notice,
            allowed,
        },
        csr,
    )?;
    // The independently selected flash response hook clears before adapter commitment.
    res.html(html)?;
    Ok(Flow::Continue)
}
/// Denied deep links and forged mutations never invoke the protected closure.
async fn protected(
    req: &mut Request,
    res: &mut Response,
    configured: bool,
    mutation: bool,
    count: Arc<AtomicUsize>,
) -> Result<Flow> {
    let facts = facts(req);
    let policy: Option<&dyn HostPolicy<Facts>> = configured.then_some(&FixturePolicy);
    let action = ActionId::new(if mutation {
        "private.mutate"
    } else {
        "private.page"
    })
    .map_err(|cause| failure("identify protected action", cause))?;
    if HostAccess::new(policy, &facts)
        .run(&action, || {
            count.fetch_add(1, Ordering::SeqCst);
        })
        .is_err()
    {
        return denied(res, 403);
    }
    res.json(&serde_json::json!({"public":"Explicit fixture admission"}))?;
    Ok(Flow::Continue)
}
/// Compile routes once, with one top-level request and response lifecycle per call.
async fn application(
    base: BasePath,
    csr: bool,
    configured: bool,
) -> std::result::Result<stackpress_server::Server, Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    register(
        &mut app,
        base,
        csr,
        configured,
        "https://preferences.invalid".into(),
        false,
    )?;
    Ok(app.build().await?)
}
/// Register existing preference pages/actions with a trusted canonical origin.
/// # Errors
/// Returns route registration failures; caller owns subsequent startup.
pub fn configure(
    app: &mut ServerBuilder,
    base: BasePath,
    csr: bool,
    configured: bool,
    origin: String,
) -> Result<()> {
    register(app, base, csr, configured, origin, true)
}
/// Preserve finite-proof metadata on rejected paths while isolating composed example headers.
fn register(
    app: &mut ServerBuilder,
    base: BasePath,
    csr: bool,
    configured: bool,
    origin: String,
    scoped_lifecycle: bool,
) -> Result<()> {
    // This example explicitly composes ordinary registration functions; no nested plugin API.
    app.on_config(sessionless)?;
    preferences::register(app)?;
    flash::register(app)?;
    let locales = registry();
    let pages = LocaleRouting::new(&locales, &base)
        .prefixes()
        .into_iter()
        .map(|prefix| PublicLocation::new(format!("{prefix}/preferences")))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|cause| failure("flash pages", cause))?;
    let preference_config = preferences::Config {
        origin: origin.clone(),
        base: base.clone(),
        locales: locales.locales().map(str::to_owned).collect(),
        default_locale: locales.default_locale().into(),
        aliases: locales
            .aliases()
            .map(|(alias, locale)| (alias.into(), locale.into()))
            .collect(),
        settings_path: PublicLocation::new("/ui-preferences")
            .map_err(|cause| failure("settings path", cause))?,
        confirmation_path: PublicLocation::new("/ui-preferences/confirm")
            .map_err(|cause| failure("confirmation path", cause))?,
        // The composed review host already owns its checked generated AssetBundle.
        serve_theme_bootstrap: !scoped_lifecycle,
    };
    let flash_config = flash::Config {
        origin,
        base: base.clone(),
        pages,
    };
    let mut layers = ConfigLayers::new();
    layers.value(
        "preferences-example",
        serde_json::json!({
            "view_preferences": preference_config, "view_flash": flash_config,
        }),
    )?;
    app.config_layers(layers)?;
    let requests = Arc::new(AtomicUsize::new(0));
    let responses = Arc::new(AtomicUsize::new(0));
    let protected_calls = Arc::new(AtomicUsize::new(0));
    let count = requests.clone();
    app.on(
        "request",
        handler(move |_, _, _| {
            let count = count.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(Flow::Continue)
            })
        }),
    )?;
    let count = protected_calls.clone();
    let response_scope = if scoped_lifecycle {
        base.as_str().to_owned()
    } else {
        String::new()
    };
    app.on("response", handler(move |req, res, _| {
        let scoped = response_scope.is_empty() || req.url.path().starts_with(&format!("{response_scope}/"));
        let request = requests.load(Ordering::SeqCst); let response = responses.fetch_add(1, Ordering::SeqCst) + 1; let calls = count.load(Ordering::SeqCst);
        Box::pin(async move {
            if !scoped {return Ok(Flow::Continue);}
            res.headers_mut()?.set("x-preferences-lifecycle", &format!("{request}/{response}"))?;
            res.headers_mut()?.set("x-protected-calls", &calls.to_string())?;
            // The immutable pre-paint resource retains its cache policy in the finite host.
            if !req.url.path().ends_with(stackpress_framework_view::theme::BOOTSTRAP_FILE) {
                res.headers_mut()?.set("cache-control", "no-store")?;
            }
            res.headers_mut()?.set("content-security-policy", "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'unsafe-inline'; connect-src 'self'; base-uri 'none'; form-action 'self'")?;
            Ok(Flow::Continue)
        })
    }))?;
    for prefix in LocaleRouting::new(&registry(), &base).prefixes() {
        let path = format!("{}{prefix}/preferences", base.as_str());
        let base = base.clone();
        app.get(
            &path,
            handler(move |req, res, invocation| {
                let base = base.clone();
                Box::pin(page(req, res, base, csr, configured, invocation))
            }),
        )?;
    }
    // Match the layout link's mount for both protected reads and mutations.
    let protected_path = format!("{}/private", base.as_str());
    for (method, mutation) in [
        (stackpress_lib::MethodPattern::GET, false),
        (stackpress_lib::MethodPattern::POST, true),
    ] {
        let count = protected_calls.clone();
        app.route(
            method,
            &protected_path,
            handler(move |req, res, _| {
                Box::pin(protected(req, res, configured, mutation, count.clone()))
            }),
        )?;
    }
    Ok(())
}
/// Explicit fixture admission is sessionless; protected-operation policy remains separate.
async fn sessionless(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new(preferences::Admission::sessionless()))?;
    Ok(Flow::Continue)
}
/// Normalize and handle once; no listening socket, daemon or implicit retry exists.
pub async fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut text = String::new();
    std::io::stdin()
        .take(64 * 1024 + 1)
        .read_to_string(&mut text)?;
    if text.len() > 64 * 1024 {
        return Err("bridge input exceeds limit".into());
    }
    let input: Input = serde_json::from_str(&text)?;
    let base = BasePath::new(std::env::var("PREFERENCES_BASE").unwrap_or_default())?;
    let configured = std::env::var("PREFERENCES_TEST_POLICY").as_deref() == Ok("explicit-fixture");
    let app = application(base, input.csr, configured).await?;
    let mut request = http::Request::builder()
        .method(input.method.as_str())
        .uri(&input.path)
        .header("host", "preferences.invalid")
        .header("cookie", input.cookie)
        .header("x-fixture-caller", input.caller)
        .header("x-fixture-csrf", input.csrf);
    if input.method == "POST" && !input.body.is_empty() {
        request = request.header("content-type", "application/json");
    }
    if let Some(origin) = input.origin {
        request = request.header("origin", origin);
    }
    let outcome = HttpAdapter::new(Default::default())?
        .handle(&app, request.body(input.body.into_bytes())?)
        .await?;
    let HttpOutput::Response(response) = outcome.output else {
        return Err("fixture output unavailable".into());
    };
    let mut headers = BTreeMap::<String, Vec<String>>::new();
    for (key, value) in response.headers() {
        headers
            .entry(key.as_str().into())
            .or_default()
            .push(value.to_str()?.to_owned());
    }
    println!(
        "{}",
        serde_json::json!({"status":response.status().as_u16(),"headers":headers})
    );
    std::io::stdout().write_all(response.body())?;
    Ok(())
}
