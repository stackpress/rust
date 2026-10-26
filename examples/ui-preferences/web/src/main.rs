//! Finite real HttpAdapter host. Reads one pipe request, writes one response, exits.
//! The opt-in fixture policy is deliberately not authentication or a deployable host.
use serde::Deserialize;
use stackpress_example_ui_preferences_shared::{Seed, action, registry};
use stackpress_framework_language::{MAX_PREFERENCE_BYTES, PreferenceError};
use stackpress_framework_preferences::{
    ActionId, CookieScope, FlashCookie, HostAccess, HostPolicy,
};
use stackpress_framework_view::BasePath;
use stackpress_lib::{Body, Error, ErrorKind, Flow, Request, Response, Result, handler};
use stackpress_server::{
    ServerBuilder,
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
) -> Result<Flow> {
    let location = req.url.query().map_or_else(
        || req.url.path().to_owned(),
        |query| format!("{}?{query}", req.url.path()),
    );
    let cookie = req.headers.get("cookie").unwrap_or_default();
    let preferences = action(base.clone())
        .select(&location, cookie)
        .map_err(|cause| failure("select preferences", cause))?;
    let scope = CookieScope::new("https://preferences.invalid".into(), base.clone())
        .map_err(|cause| failure("scope flash", cause))?;
    let consumed = FlashCookie::new(scope)
        .consume(cookie)
        .map_err(|cause| failure("consume public flash", cause))?;
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
            flash: consumed.notice,
            allowed,
        },
        csr,
    )?;
    // Clear with exactly the issuing scope before this complete response is committed.
    if let Some(clear) = consumed.clear_cookie {
        res.headers_mut()?.append("set-cookie", &clear)?;
    }
    res.html(html)?;
    Ok(Flow::Continue)
}
/// Fixed validated action; failed actions never stage preference cookies.
async fn preferences(req: &mut Request, res: &mut Response, base: BasePath) -> Result<Flow> {
    let bytes = match req.body() {
        Body::Bytes(value) => value.as_slice(),
        Body::Text(value) => value.as_bytes(),
        _ => &[],
    };
    if bytes.len() > MAX_PREFERENCE_BYTES {
        return denied(res, 413);
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return denied(res, 400);
    };
    let action = action(base);
    let outcome = if req.url.path().ends_with("/confirm") {
        action.confirm(
            "POST",
            req.headers.get("origin"),
            text,
            req.headers.get("cookie").unwrap_or_default(),
        )
    } else {
        action.apply("POST", req.headers.get("origin"), text)
    };
    match outcome {
        Ok(reply) => {
            for cookie in &reply.cookies {
                res.headers_mut()?.append("set-cookie", cookie)?;
            }
            res.json(&reply)?;
            Ok(Flow::Continue)
        }
        Err(cause) => denied(
            res,
            match cause {
                PreferenceError::Origin => 403,
                PreferenceError::Limit => 413,
                _ => 400,
            },
        ),
    }
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
    app.on("response", handler(move |_, res, _| {
        let request = requests.load(Ordering::SeqCst); let response = responses.fetch_add(1, Ordering::SeqCst) + 1; let calls = count.load(Ordering::SeqCst);
        Box::pin(async move {
            res.headers_mut()?.set("x-preferences-lifecycle", &format!("{request}/{response}"))?;
            res.headers_mut()?.set("x-protected-calls", &calls.to_string())?;
            res.headers_mut()?.set("cache-control", "no-store")?;
            res.headers_mut()?.set("content-security-policy", "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'unsafe-inline'; connect-src 'self'; base-uri 'none'; form-action 'self'")?;
            Ok(Flow::Continue)
        })
    }))?;
    for prefix in registry().prefixes() {
        let path = format!("{}{prefix}/preferences", base.as_str());
        let base = base.clone();
        app.get(
            &path,
            handler(move |req, res, _| {
                let base = base.clone();
                Box::pin(page(req, res, base, csr, configured))
            }),
        )?;
    }
    for suffix in ["/ui-preferences", "/ui-preferences/confirm"] {
        let action_path = format!("{}{suffix}", base.as_str());
        let base = base.clone();
        app.post(
            &action_path,
            handler(move |req, res, _| {
                let base = base.clone();
                Box::pin(preferences(req, res, base))
            }),
        )?;
    }
    for (method, mutation) in [
        (stackpress_lib::MethodPattern::GET, false),
        (stackpress_lib::MethodPattern::POST, true),
    ] {
        let count = protected_calls.clone();
        app.route(
            method,
            "/private",
            handler(move |req, res, _| {
                Box::pin(protected(req, res, configured, mutation, count.clone()))
            }),
        )?;
    }
    Ok(app.build().await?)
}
/// Normalize and handle once; no listening socket, daemon or implicit retry exists.
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
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
