//! Native page-policy adapter around independent synchronous Leptos execution.
use super::safety::escape_html;
use crate::{
    BasePath, PageAssets, PageContract, PageLimits, PageRenderer, PageSnapshot, PublicLocation,
    RenderFuture, RenderedPage, ViewError,
};
use leptos::prelude::*;
use serde::Serialize;
use stackpress_leptos::render::{PreparedRender, RenderError, RenderLimits, Renderer};
/// Initialize native SSR, preserving the page adapter's established error type.
/// # Errors
/// Requires an active Tokio runtime.
pub fn initialize_ssr() -> Result<(), ViewError> {
    stackpress_leptos::render::initialize_ssr().map_err(|error| match error {
        RenderError::ExecutorUnavailable => ViewError::ExecutorUnavailable,
        RenderError::InvalidPolicy => ViewError::InvalidPolicy,
        RenderError::HtmlLimit => ViewError::HtmlLimit,
        RenderError::Replay(e) => super::replay::map_error(e),
        RenderError::Failure(e) => ViewError::Failure(e),
        RenderError::Component(never) => match never {},
    })
}
/// Admit the executor before page validation or component side effects.
pub(super) fn require_executor() -> Result<(), ViewError> {
    stackpress_leptos::render::require_executor().map_err(map_error)
}
/// Translate generic execution failures while preserving original caller errors and native causes.
pub(super) fn map_error(error: RenderError<ViewError>) -> ViewError {
    match error {
        RenderError::ExecutorUnavailable => ViewError::ExecutorUnavailable,
        RenderError::InvalidPolicy => ViewError::InvalidPolicy,
        RenderError::HtmlLimit => ViewError::HtmlLimit,
        RenderError::Replay(e) => super::replay::map_error(e),
        RenderError::Component(e) => e,
        RenderError::Failure(e) => ViewError::Failure(e),
    }
}
/// Adapt page limits without widening either output policy.
pub(super) fn limits(value: PageLimits) -> RenderLimits {
    RenderLimits {
        html_bytes: value.html_bytes,
        replay_bytes: value.snapshot_bytes,
    }
}
/// Staged page shell; generic execution has already disposed its synchronous Owner.
pub struct PreparedPage {
    inner: PreparedRender<ViewError>,
    limits: PageLimits,
}
impl PreparedPage {
    /// Finish eager owned Head and validate the fully framed page.
    /// # Errors
    /// Returns existing executor, Head failure or page-output limit errors.
    pub async fn finish(self) -> Result<RenderedPage, ViewError> {
        let html = self.inner.finish().await.map_err(map_error)?;
        RenderedPage::with_limits(html, self.limits)
    }
}
/// Synchronous render entry; each call receives a fresh, unshared Owner.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_view::leptos::render::{LeptosRenderer};
/// use stackpress_framework_view::{BasePath, PageAssets, PageContract, PageSnapshot, PublicLocation};
/// let contract = PageContract::new(1, "app-build")?;
/// let assets = PageAssets {
///     theme_bootstrap: None,
///     build: "app-build".into(), base: BasePath::default(),
///     module: PublicLocation::new("/start.js")?, stylesheet: None,
/// };
/// let _renderer = LeptosRenderer::new(contract, assets, |page: &PageSnapshot<String>| {
///     Ok(view! { <p>{page.data.clone()}</p> })
/// })?;
/// # Ok::<(), stackpress_framework_view::ViewError>(())
/// ```
pub struct LeptosRenderer<F = ()> {
    /// Immutable explicit version/settings/limit contract.
    contract: PageContract,
    /// Checked assets and route mount agree before registration.
    assets: PageAssets,
    /// Component factory creates only request-local reactive state.
    component: F,
}

impl LeptosRenderer<()> {
    /// Render hydration-marked body and safely stage the public seed.
    ///
    /// Component evaluation is synchronous. The Owner is cleaned on success or
    /// unwind before returning; no global render lock is required.
    /// Call [`initialize_ssr`] during startup and render inside a Tokio runtime.
    /// Head values must be eager owned text: read signals before constructing
    /// Title props. Use the explicit async renderer for reactive Head.
    ///
    /// # Errors
    /// Rejects missing SSR startup/runtime and oversized/malformed public seed
    /// before component evaluation, without an uninitialized-executor panic.
    pub fn prepare<T: Serialize, V: IntoView>(
        snapshot: &PageSnapshot<T>,
        component: impl FnOnce() -> V,
    ) -> Result<PreparedPage, ViewError> {
        let mut contract = PageContract::new(snapshot.revision, snapshot.build.clone())?;
        contract.locale = snapshot.locale.clone();
        contract.theme = snapshot.theme.clone();
        let assets = PageAssets {
            theme_bootstrap: None,
            build: contract.build.clone(),
            base: BasePath::default(),
            module: PublicLocation::new("/assets/start.js")?,
            stylesheet: None,
        };
        prepare(snapshot, &contract, &assets, || Ok(component()))
    }

    /// Construct a reusable native renderer from checked immutable metadata and a component.
    ///
    /// The factory runs synchronously in a fresh Owner and may only return owned
    /// view state. It receives the allowlisted snapshot, never a live invocation.
    ///
    /// Call [`initialize_ssr`] during startup inside Tokio before invoking the
    /// [`PageRenderer::render`] entry. Missing initialization is a typed error.
    /// Factories are trusted synchronous code: byte ceilings bound staged output,
    /// not CPU time or allocations performed inside an arbitrary component.
    /// Head values/formatters must eagerly own their data, never read disposed
    /// signals during assembly. The explicit async renderer supports reactive Head.
    ///
    /// # Panics
    /// A component panic propagates after request-local Owner cleanup. The host
    /// chooses unwind/abort policy; this boundary never retries a failed render.
    ///
    /// # Errors
    /// Rejects unsafe policy or an asset/data build mismatch before registration.
    pub fn new<T, F, V>(
        contract: PageContract,
        assets: PageAssets,
        component: F,
    ) -> Result<LeptosRenderer<F>, ViewError>
    where
        F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
        V: IntoView,
    {
        assets.validate(&contract)?;
        Ok(LeptosRenderer {
            contract,
            assets,
            component,
        })
    }
}

impl<T: Serialize, F, V> PageRenderer<T> for LeptosRenderer<F>
where
    F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
    V: IntoView,
{
    /// Expose only immutable public policy to the native page registration adapter.
    fn contract(&self) -> &PageContract {
        &self.contract
    }
    /// Evaluate local reactive state before returning the Send owned-buffer future.
    /// Head props must be eager owned data; reactive Head uses the async renderer.
    fn render(&self, snapshot: PageSnapshot<T>) -> RenderFuture<'_> {
        let staged = prepare(&snapshot, &self.contract, &self.assets, || {
            (self.component)(&snapshot)
        });
        Box::pin(async move { staged?.finish().await })
    }
}

/// Admit public policy before evaluating a root-local component.
fn prepare<T: Serialize, V: IntoView>(
    snapshot: &PageSnapshot<T>,
    contract: &PageContract,
    assets: &PageAssets,
    component: impl FnOnce() -> Result<V, ViewError>,
) -> Result<PreparedPage, ViewError> {
    require_executor()?;
    assets.validate(contract)?;
    snapshot.validate(contract.revision, &contract.build)?;
    if snapshot.locale != contract.locale || snapshot.theme != contract.theme {
        return Err(ViewError::ContractMismatch);
    }
    let seed = snapshot.script_json_with(contract.limits)?;
    let bootstrap = assets.script_json()?;
    let module = assets.base.join(assets.module.as_str())?;
    // Saved explicit choices are complete SSR output; only unset admits the external sampler.
    let theme_head = if snapshot.theme.is_none() {
        crate::theme::bootstrap_tag(&assets.base, assets.theme_bootstrap.as_ref(), None)?
    } else {
        String::new()
    };
    // The unset marker permits one initial sample while the no-JS document remains readable Light.
    let theme_marker = if snapshot.theme.is_none() {
        " data-theme-unset"
    } else {
        ""
    };
    let inner=Renderer::try_prepare(limits(contract.limits),||{
        super::routing::provide_route_context(assets.base.clone());
        // Router SSR sees only the canonical public path, never an origin/cookie/header dump.
        provide_context(leptos_router::location::RequestUrl::new(&snapshot.location));
        component()
    },|body|{
        let _ =
            view! { <leptos_meta::Link rel="canonical" href=snapshot.location.clone()/> }.to_html();
        if let Some(style) = &assets.stylesheet {
            let href = assets.base.join(style.as_str())?.as_str().to_owned();
            let _ = view! { <leptos_meta::Stylesheet id="stackpress-style" href/> }.to_html();
        }
    let html = format!(
        "<!DOCTYPE html><html lang=\"{}\" data-theme=\"{}\"{theme_marker}><head>{theme_head}<meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><div id=\"app\">{body}</div><script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"application/json\" id=\"stackpress-assets\">{bootstrap}</script><script type=\"module\" src=\"{}\"></script></body></html>",
        snapshot.locale,
        snapshot.theme.as_deref().unwrap_or("light"),
        escape_html(module.as_str())
    );
        Ok(html)
    }).map_err(map_error)?;
    Ok(PreparedPage {
        inner,
        limits: contract.limits,
    })
}
