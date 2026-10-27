//! Finite real HttpAdapter fixture; every invocation handles once and exits without listening.
use leptos::prelude::*;
use serde::Deserialize;
use stackpress_example_ui_gallery_shared::{Gallery, GallerySeed, RESOURCE_TEXT};
use stackpress_framework_language::{
    LocaleRegistry, MAX_PREFERENCE_BYTES, PreferenceAction, PreferenceUpdate,
};
use stackpress_framework_ui::{async_render::AsyncLeptosRenderer, render::initialize_ssr};
use stackpress_framework_view::{
    AsyncPageRenderer, BasePath, PageAssets, PageContract, PageId, PageSnapshot, PublicLocation,
    PublicStatus,
};
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

/// Explicit request DTO supplied only by the finite browser pipe bridge.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    /// Real HTTP method.
    method: String,
    /// Local path/query at the selected mount.
    path: String,
    /// Incoming Cookie header from the real browser cookie jar.
    #[serde(default)]
    cookie: String,
    /// Browser Origin metadata; missing remains missing.
    origin: Option<String>,
    /// Fixed preference JSON, not merged query input.
    #[serde(default)]
    body: String,
    /// Select an empty CSR shell seeded by the same host policy.
    #[serde(default)]
    csr: bool,
}

/// Preserve a concrete native cause while adding operation context exactly once.
fn failure(context: &'static str, cause: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::new(context)
        .with_kind(ErrorKind::External)
        .with_cause(cause)
}

/// Explicit canonical languages and one declared alias, with no browser guessing.
fn registry() -> LocaleRegistry {
    LocaleRegistry::new(
        vec!["en".into(), "fr".into()],
        "en".into(),
        BTreeMap::from([("fr-FR".into(), "fr".into())]),
    )
    .expect("fixed valid registry")
}

/// Send page loader projects only checked public settings and app data.
async fn load_gallery(req: &mut Request, base: &BasePath) -> Result<GallerySeed> {
    let location = match req.url.query() {
        Some(query) => format!("{}?{query}", req.url.path()),
        None => req.url.path().to_owned(),
    };
    let selected = registry()
        .select(
            base,
            &location,
            req.headers.get("cookie").unwrap_or_default(),
        )
        .map_err(|cause| failure("select gallery locale", cause))?;
    let theme = if req
        .url
        .query_pairs()
        .any(|(key, value)| key == "theme" && value == "dark")
    {
        "dark"
    } else {
        "light"
    };
    let seed = GallerySeed {
        locale: selected.locale,
        base: base.as_str().to_owned(),
        location,
        name: "Ada".into(),
        theme: theme.into(),
    };
    // The owned projection survives a real top-level async boundary without Request borrows.
    tokio::task::yield_now().await;
    Ok(seed)
}

/// Render a complete typed page; no nested handle or second lifecycle is invoked.
async fn page(req: &mut Request, res: &mut Response, base: BasePath, csr: bool) -> Result<Flow> {
    let seed = load_gallery(req, &base).await?;
    let mut contract =
        PageContract::new(1, "s8-m1").map_err(|cause| failure("configure gallery page", cause))?;
    contract.locale = seed.locale.clone();
    contract.theme = seed.theme.clone();
    let snapshot = PageSnapshot {
        revision: 1,
        build: "s8-m1".into(),
        page: PageId::try_from("gallery".to_owned())
            .map_err(|cause| failure("identify gallery page", cause))?,
        location: seed.location.clone(),
        locale: seed.locale.clone(),
        theme: seed.theme.clone(),
        status: PublicStatus::ok(),
        data: seed,
    };
    let html = if csr {
        // CSR acquires the same seed before mounting; only its Resource loads initially.
        let json = snapshot
            .script_json()
            .map_err(|cause| failure("serialize CSR seed", cause))?;
        format!(
            "<!DOCTYPE html><html lang=\"{}\" data-theme=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"></head><body><div id=\"app\"></div><script type=\"application/json\" id=\"stackpress-page\">{json}</script><script type=\"application/json\" id=\"stackpress-resources\">[]</script><script type=\"module\" src=\"{}/assets/start.js\"></script></body></html>",
            snapshot.locale,
            snapshot.theme,
            base.as_str()
        )
    } else {
        let assets = PageAssets {
            build: "s8-m1".into(),
            base,
            module: PublicLocation::new("/assets/start.js")
                .map_err(|cause| failure("select gallery module", cause))?,
            stylesheet: None,
        };
        let renderer =
            AsyncLeptosRenderer::new(contract, assets, |snapshot: &PageSnapshot<GallerySeed>| {
                Ok(view! {<Gallery seed=snapshot.data.clone()/>})
            })
            .map_err(|cause| failure("configure async gallery", cause))?;
        renderer
            .render_async(snapshot)
            .await
            .map_err(|cause| failure("render async gallery", cause))?
            .into_html()
    };
    res.headers_mut()?.set("cache-control", "no-store")?;
    res.headers_mut()?.set("content-security-policy","default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'unsafe-inline'; connect-src 'self'; img-src 'self'; base-uri 'none'; form-action 'self'")?;
    res.html(html)?;
    Ok(Flow::Continue)
}

/// Apply the real fixed preference policy and keep Set-Cookie outside the public JSON.
async fn preferences(req: &mut Request, res: &mut Response, base: BasePath) -> Result<Flow> {
    let body = match req.body() {
        Body::Bytes(bytes) => bytes.as_slice(),
        Body::Text(text) => text.as_bytes(),
        _ => &[],
    };
    if body.len() > MAX_PREFERENCE_BYTES {
        return public_failure(res, 413, "Preference input too large");
    }
    let update: PreferenceUpdate = match serde_json::from_slice(body) {
        Ok(update) => update,
        Err(_) => {
            return public_failure(res, 400, "Invalid preference request");
        }
    };
    let action = PreferenceAction::new(registry(), "https://gallery.invalid".into(), base)
        .map_err(|cause| failure("configure preference action", cause))?;
    match action.apply("POST", req.headers.get("origin"), update) {
        Ok(reply) => {
            res.headers_mut()?.set("set-cookie", &reply.cookie)?;
            res.headers_mut()?.set("cache-control", "no-store")?;
            res.json(&reply)?;
        }
        Err(error) => {
            return public_failure(res, 403, &error.to_string());
        }
    }
    Ok(Flow::Continue)
}

/// JSON staging selects success first; publish the explicit denial status afterward.
fn public_failure(res: &mut Response, code: u16, message: &str) -> Result<Flow> {
    res.json(&serde_json::json!({"error": message}))?;
    res.status_code(code, None)?;
    res.headers_mut()?.set("cache-control", "no-store")?;
    Ok(Flow::Continue)
}

/// Build only fixed routes and lifecycle counters; no listener or native credential exists.
async fn application(
    base: BasePath,
    csr: bool,
) -> std::result::Result<stackpress_server::Server, Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
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
    app.on(
        "response",
        handler(move |_, res, _| {
            let count = calls.load(Ordering::SeqCst);
            Box::pin(async move {
                res.headers_mut()?
                    .set("x-gallery-lifecycle", &count.to_string())?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    for prefix in ["", "/en", "/fr", "/fr-FR"] {
        let path = format!("{}{prefix}/gallery", base.as_str());
        let base = base.clone();
        app.get(
            &path,
            handler(move |req, res, _| {
                let base = base.clone();
                Box::pin(page(req, res, base, csr))
            }),
        )?;
    }
    let action_base = base.clone();
    app.post(
        &format!("{}/ui-preferences", base.as_str()),
        handler(move |req, res, _| {
            let base = action_base.clone();
            Box::pin(preferences(req, res, base))
        }),
    )?;
    app.get(
        &format!("{}/resource", base.as_str()),
        handler(|_, res, _| {
            Box::pin(async move {
                res.json(&RESOURCE_TEXT)?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    Ok(app.build().await?)
}

/// Accept one bounded pipe request, execute the real HTTP adapter once, and exit.
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    initialize_ssr()?;
    if std::env::args().nth(1).as_deref() == Some("corpus") {
        let safe = stackpress_example_ui_gallery_shared::CORPUS
            .iter()
            .map(|input| {
                stackpress_framework_ui::rich::SafeHtml::html(input)
                    .map(|value| value.as_str().to_owned())
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        println!("{}", serde_json::to_string(&safe)?);
        return Ok(());
    }
    let mut input = String::new();
    std::io::stdin()
        .take(64 * 1024 + 1)
        .read_to_string(&mut input)?;
    if input.len() > 64 * 1024 {
        return Err("fixture input too large".into());
    }
    let input: Input = serde_json::from_str(&input)?;
    let base = BasePath::new(std::env::var("GALLERY_BASE").unwrap_or_default())?;
    let app = application(base, input.csr).await?;
    let mut request = http::Request::builder()
        .method(input.method.as_str())
        .uri(input.path);
    if input.method == "POST" {
        request = request.header("content-type", "application/json");
    }
    if !input.cookie.is_empty() {
        request = request.header("cookie", input.cookie);
    }
    if let Some(origin) = input.origin {
        request = request.header("origin", origin);
    }
    let output = HttpAdapter::new(Default::default())?
        .handle(&app, request.body(input.body.into_bytes())?)
        .await?
        .output;
    match output {
        HttpOutput::Response(response) => {
            let headers: BTreeMap<_, _> = response
                .headers()
                .iter()
                .map(|(name, value)| {
                    (
                        name.to_string(),
                        value.to_str().unwrap_or_default().to_owned(),
                    )
                })
                .collect();
            println!(
                "{}",
                serde_json::json!({"status":response.status().as_u16(),"headers":headers})
            );
            std::io::stdout().write_all(response.body())?;
        }
        _ => return Err("fixture output unavailable".into()),
    }
    Ok(())
}
