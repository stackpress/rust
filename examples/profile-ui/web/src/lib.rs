//! Buffered native page adapter using the generated domain operation unchanged.

use stackpress_example_profile_ui_contracts::{SearchInput, SearchPage, SearchReply, snapshot};
use stackpress_example_profile_ui_shared::ProfileSearch;
use stackpress_framework_ui::{leptos::prelude::*, render::LeptosRenderer};
use stackpress_framework_view::PageLoad;
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, Server, ServerBuilder, ServerInvocationExt};

/// Build an ordinary immutable host with the generated plugin and one UI route.
///
/// # Errors
/// Preserves checked plugin/startup/registration failures.
pub async fn application() -> BuildResult<Server> {
    stackpress_framework_ui::render::initialize_ssr().map_err(Error::from_source)?;
    let mut app = ServerBuilder::new();
    stackpress_example_generated_profile::generated::register(&mut app)?;
    app.get("/ui/profiles", page)?;
    app.post("/ui-api/v1/profile-search", search_api)?;
    app.build().await
}

/// Fixed HTTP operation entry; caller fields never select the delegated event.
///
/// # Errors
/// Preserves validation/domain errors through the existing lifecycle.
pub async fn search_api(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let revision = req.data.path_as::<u32>("revision")?.unwrap_or_default();
    if revision != stackpress_example_profile_ui_contracts::REVISION {
        return Err(Error::invalid("Page version changed. Refresh to continue."));
    }
    match load_profiles(req, res, inv).await? {
        PageLoad::Abort => Ok(Flow::Abort),
        PageLoad::Ready(page) => {
            res.json(&page.reply)?;
            Ok(Flow::Continue)
        }
    }
}

/// Load owned public data by raw nested delegation inside the current lifecycle.
///
/// # Errors
/// Preserves input/domain failures; malformed public projection becomes an error.
pub async fn load_profiles(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<PageLoad<SearchPage>> {
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    let input = SearchInput::new(id).map_err(Error::invalid)?;
    // No call()/handle() here: the page already owns the one top-level lifecycle.
    if inv.delegate("profile-search", req, res).await? == Flow::Abort {
        return Ok(PageLoad::Abort);
    }
    let reply = SearchReply::from_envelope(&res.try_to_json()?).map_err(Error::invalid)?;
    Ok(PageLoad::Ready(SearchPage { input, reply }))
}

/// Named Send handler: complete the loader before constructing reactive state.
///
/// # Errors
/// Returns loader/render errors through the existing host error lifecycle.
pub async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let data = match load_profiles(req, res, inv).await? {
        PageLoad::Ready(data) => data,
        PageLoad::Abort => return Ok(Flow::Abort),
    };
    // Serialize two independent owned projections, without cloning the generated Profile type.
    let snapshot = snapshot(data).map_err(Error::from_source)?;
    let html = render_snapshot(snapshot).await?;
    res.html(html)?;
    Ok(Flow::Continue)
}

/// Render owned public data using the same component/seed path as an HTTP page.
///
/// # Errors
/// Returns bounded seed/HTML errors and missing native SSR initialization.
pub async fn render_snapshot(
    snapshot: stackpress_framework_view::PageSnapshot<SearchPage>,
) -> Result<String> {
    let seed =
        serde_json::from_value(serde_json::to_value(&snapshot.data).map_err(Error::from_source)?)
            .map_err(Error::from_source)?;
    let prepared = LeptosRenderer::prepare(&snapshot, || view! { <ProfileSearch seed/> })
        .map_err(Error::from_source)?;
    // Only owned Send HTML/Head buffers survive this await; the Owner is already disposed.
    let html = prepared
        .finish()
        .await
        .map_err(Error::from_source)?
        .into_html();
    Ok(html)
}
