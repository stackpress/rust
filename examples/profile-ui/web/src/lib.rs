//! Buffered typed pages and a fixed HTTP operation over the existing lifecycle.
//!
//! The generated in-memory capability is the default. The native-only `sqlite`
//! feature selects sql-profile's existing read-only plugins before compilation;
//! explicit configuration names an already installed file. Presentation is shared.

use stackpress_example_profile_ui_contracts::{
    BUILD, DetailContext, MAX_REQUEST_BYTES, REVISION, SearchInput, SearchPage, SearchReply,
};
use stackpress_example_profile_ui_shared::ProfilePage;
use stackpress_framework_view::leptos::render::LeptosRenderer;
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageLoad, PageRenderer, PageSnapshot, PageSpec,
    PublicLocation, ServerBuilderViewExt, ViewError,
};
use stackpress_leptos::leptos::prelude::*;
use stackpress_lib::{Body, Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder, ServerInvocationExt};

pub mod backend;

#[path = "../../route-contract.rs"]
pub mod routes;

#[cfg(feature = "demo-customization")]
#[path = "../../customization.rs"]
pub mod customization;

/// Initialize the native render requirement at an explicit execution-host startup boundary.
///
/// # Errors
/// Returns a typed missing-runtime/SSR-executor error.
pub fn initialize_renderer() -> Result<()> {
    stackpress_framework_view::leptos::render::initialize_ssr().map_err(Error::from_source)
}

/// Build the compiled read-only backend at the root mount with registered defaults.
///
/// # Errors
/// Preserves checked plugin/startup/page-registration failures.
pub async fn application() -> BuildResult<Server> {
    application_with_base(BasePath::default()).await
}

/// Build one explicit mount with registered defaults; caller headers never select the base.
///
/// # Errors
/// Returns checked startup, route, asset or generated plugin failures.
pub async fn application_with_base(base: BasePath) -> BuildResult<Server> {
    application_with_config(base, ConfigLayers::new()).await
}

/// Build the compiled backend with explicit application-owned configuration layers.
///
/// With `sqlite`, startup opens an installed file read-only and verifies its schema.
/// Use sql-profile's separate setup command first; no startup path installs or seeds.
///
/// # Errors
/// Returns configuration, database-open/schema, plugin and page registration failures.
pub async fn application_with_config(base: BasePath, layers: ConfigLayers) -> BuildResult<Server> {
    initialize_renderer()?;
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    configure_application(&mut app, base.clone())?;
    app.build().await.map_err(backend::startup_error)
}

/// Register the selected domain, optional compiled plugin and shared page/API routes.
///
/// # Errors
/// Returns ordinary plugin, route and renderer registration errors.
pub fn configure_application(app: &mut ServerBuilder, base: BasePath) -> Result<()> {
    configure_application_with_module(app, base, "/assets/start.js")
}

/// Select a checked content-addressed bootstrap module for the example gateway.
///
/// # Errors
/// Returns invalid module, plugin and page registration errors.
#[expect(
    clippy::needless_pass_by_value,
    reason = "retain the public owned mount API while sharing it across page and operation registration"
)]
pub fn configure_application_with_module(
    app: &mut ServerBuilder,
    base: BasePath,
    module: &str,
) -> Result<()> {
    backend::register(app)?;
    #[cfg(feature = "demo-customization")]
    app.plugin("profile-ui-labels", customization::register)?;
    register_pages_with_module(app, base.clone(), module)?;
    app.post(
        routes::search_api(&base)
            .map_err(Error::from_source)?
            .as_str(),
        search_api,
    )?;
    Ok(())
}

/// Register native pages from the same compiled catalog used by client path descriptors.
///
/// # Errors
/// Returns checked page policy and ordinary server registration errors.
pub fn register_pages(app: &mut ServerBuilder, base: BasePath) -> Result<()> {
    register_pages_with_module(app, base, "/assets/start.js")
}

/// Register the same catalog with an explicit checked application-relative asset module.
fn register_pages_with_module(app: &mut ServerBuilder, base: BasePath, module: &str) -> Result<()> {
    let catalog = routes::catalog(base.clone()).map_err(Error::from_source)?;
    let search_base = base.clone();
    let search = PageSpec::get(
        catalog
            .pattern("profiles")
            .map_err(Error::from_source)?
            .as_str(),
        "profiles",
        load_profiles,
    )?
    .location(move |page: &SearchPage| routes::search_location(&search_base, &page.input.id));
    app.page(
        search,
        renderer_with_settings(base.clone(), "en", Some("light"), module)
            .map_err(Error::from_source)?,
    )?;
    let detail_base = base.clone();
    let detail = PageSpec::get(
        catalog
            .pattern("profile-detail")
            .map_err(Error::from_source)?
            .as_str(),
        "profile-detail",
        load_detail,
    )?
    .location(move |page: &SearchPage| {
        let detail = page.detail.as_ref().ok_or(ViewError::InvalidSnapshot)?;
        routes::detail_location(&detail_base, &detail.id, &detail.search_prefix)
    });
    app.page(
        detail,
        renderer_with_settings(base, "en", Some("light"), module).map_err(Error::from_source)?,
    )?;
    Ok(())
}

/// Decode only the fixed JSON body, independently of merged query/path data.
fn decode_search(req: &Request) -> Result<SearchInput> {
    if req.mimetype().split(';').next().map(str::trim) != Some("application/json") {
        return Err(Error::invalid("search requires JSON")
            .with_public_message("Send a JSON search request."));
    }
    let bytes = match req.body() {
        Body::Bytes(bytes) => bytes.as_slice(),
        Body::Text(text) => text.as_bytes(),
        _ => return Err(Error::invalid("search requires a buffered JSON body")),
    };
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(Error::limit("search request exceeds finite limit"));
    }
    let input: SearchInput = serde_json::from_slice(bytes).map_err(|error| {
        Error::from_source(error)
            .with_code(400)
            .with_public_message("Invalid search request.")
    })?;
    input.validate().map_err(|message| {
        Error::invalid("invalid public search input").with_public_message(message)
    })?;
    Ok(input)
}

/// Fixed JSON operation entry; no DTO field can choose an event or native service.
///
/// # Errors
/// Preserves malformed input/domain causes through the existing error lifecycle;
/// public responses use its sanitized error envelope, never a HandleReport.
pub async fn search_api(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let input = decode_search(req)?;
    // Replace the one allowed domain field after strict body validation. Query fields
    // cannot override it, and unknown body fields were rejected by the DTO decoder.
    req.data.set("id", input.id)?;
    match load_profiles(req, res, inv).await? {
        PageLoad::Abort => Ok(Flow::Abort),
        PageLoad::Ready(page) => {
            res.json(&page.reply)?;
            Ok(Flow::Continue)
        }
    }
}

/// Load owned public data by raw delegation inside the current top-level lifecycle.
///
/// # Errors
/// Preserves input/domain failures; invalid public projection becomes an error.
pub async fn load_profiles(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<PageLoad<SearchPage>> {
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    let input = SearchInput::new(id).map_err(Error::invalid)?;
    // No call()/handle() here: the current page owns exactly one lifecycle.
    if inv.delegate("profile-search", req, res).await? == Flow::Abort {
        return Ok(PageLoad::Abort);
    }
    let reply = SearchReply::from_envelope(&res.try_to_json()?).map_err(Error::invalid)?;
    Ok(PageLoad::Ready(SearchPage {
        input,
        reply,
        detail: None,
    }))
}

/// Project an exact detail from the selected prefix-search capability.
///
/// # Errors
/// Preserves domain errors and checked response-status changes.
pub async fn load_detail(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<PageLoad<SearchPage>> {
    // Native route captures are exact encoded pathname text. Decode once before
    // the shared input boundary; caller query fields cannot replace this capture.
    let encoded = inv
        .captures()
        .params
        .get("id")
        .ok_or_else(|| Error::invalid("missing detail capture"))?;
    let id = routes::decode_parameter(encoded).map_err(Error::from_source)?;
    let query = req.url.query().unwrap_or_default();
    let search_prefix = routes::decode_return_prefix(query)
        .map_err(Error::from_source)?
        .unwrap_or_else(|| "abc".into());
    req.data.set("id", id)?;
    let mut page = match load_profiles(req, res, inv).await? {
        PageLoad::Abort => return Ok(PageLoad::Abort),
        PageLoad::Ready(page) => page,
    };
    // The selected read capability stays unchanged; only the page projection is narrowed.
    page.reply.rows.retain(|row| row.id == page.input.id);
    page.reply.total = page.reply.rows.len() as u64;
    page.detail = Some(DetailContext {
        id: page.input.id.clone(),
        search_prefix,
    });
    if page.reply.rows.is_empty() {
        res.status_code(404, None)?;
    }
    Ok(PageLoad::Ready(page))
}

/// Explicit trusted display settings, shared by fixtures and normal native registration.
fn renderer_with_settings(
    base: BasePath,
    locale: &str,
    theme: Option<&str>,
    module: &str,
) -> std::result::Result<impl PageRenderer<SearchPage>, ViewError> {
    let mut contract = PageContract::new(REVISION, BUILD)?;
    contract.locale = locale.into();
    contract.theme = theme.map(str::to_owned);
    let assets = PageAssets {
        theme_bootstrap: Some(stackpress_framework_view::theme::bootstrap_location()?),
        build: BUILD.into(),
        base,
        module: PublicLocation::new(module)?,
        stylesheet: None,
    };
    LeptosRenderer::new(contract, assets, |snapshot: &PageSnapshot<SearchPage>| {
        // Generated Profile is intentionally non-Clone: independent wire projections
        // avoid adding a hidden sharing contract to the generated domain type.
        let seed = serde_json::from_value(
            serde_json::to_value(&snapshot.data).map_err(ViewError::from_source)?,
        )
        .map_err(ViewError::from_source)?;
        Ok(
            view! { <ProfilePage seed page=snapshot.page.as_str().to_owned() status=snapshot.status.clone()
            initial_language=stackpress_example_profile_ui_shared::profile_language(&snapshot.locale).map_err(|_| ViewError::InvalidSnapshot)?
            initial_theme=if snapshot.theme.as_deref() == Some("dark") { stackpress_leptos::state::Theme::Dark } else { stackpress_leptos::state::Theme::Light } initial_theme_pending=snapshot.theme.is_none()/> },
        )
    })
}

/// Compatibility named handler for applications selecting a single root page.
///
/// # Errors
/// Returns loader/render errors through the existing lifecycle.
pub async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let data = match load_profiles(req, res, inv).await? {
        PageLoad::Ready(data) => data,
        PageLoad::Abort => return Ok(Flow::Abort),
    };
    let snapshot =
        stackpress_example_profile_ui_contracts::snapshot(data).map_err(Error::from_source)?;
    res.html(render_snapshot(snapshot).await?)?;
    Ok(Flow::Continue)
}

/// Render an explicit public snapshot through the same bounded component factory.
///
/// # Errors
/// Returns bounded seed/HTML failures and missing native SSR initialization.
pub async fn render_snapshot(snapshot: PageSnapshot<SearchPage>) -> Result<String> {
    render_snapshot_with_base(snapshot, BasePath::default()).await
}

/// Render explicit public settings under the same trusted mount as assets and links.
///
/// # Errors
/// Preserves checked snapshot, display policy and render-boundary failures.
pub async fn render_snapshot_with_base(
    snapshot: PageSnapshot<SearchPage>,
    base: BasePath,
) -> Result<String> {
    let renderer = renderer_with_settings(
        base,
        &snapshot.locale,
        snapshot.theme.as_deref(),
        "/assets/start.js",
    )
    .map_err(Error::from_source)?;
    Ok(renderer
        .render(snapshot)
        .await
        .map_err(Error::from_source)?
        .into_html())
}
