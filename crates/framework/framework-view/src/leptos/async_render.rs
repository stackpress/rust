//! Public-page adapter for complete buffered asynchronous Leptos rendering.
use super::{
    render::{limits as render_limits, map_error, require_executor},
    safety::escape_html,
};
use crate::{
    AsyncPageRenderer, AsyncRenderFuture, PageAssets, PageContract, PageSnapshot, RenderedPage,
    ViewError,
};
use leptos::prelude::*;
use serde::Serialize;
use stackpress_leptos::async_render::AsyncRenderer;
/// Explicit complete-page renderer; the existing synchronous renderer is unchanged.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_view::leptos::async_render::{AsyncLeptosRenderer};
/// use stackpress_framework_view::{BasePath, PageAssets, PageContract, PageSnapshot, PublicLocation};
/// let contract = PageContract::new(1, "app-build")?;
/// let assets = PageAssets {
///     theme_bootstrap: None,
///     build: "app-build".into(), base: BasePath::default(),
///     module: PublicLocation::new("/start.js")?, stylesheet: None,
/// };
/// let _renderer = AsyncLeptosRenderer::new(contract, assets, |page: &PageSnapshot<String>| {
///     Ok(view! { <p>{page.data.clone()}</p> })
/// })?;
/// # Ok::<(), stackpress_framework_view::ViewError>(())
/// ```
pub struct AsyncLeptosRenderer<F> {
    /// Immutable contract and lowerable byte budgets for this page.
    contract: PageContract,
    /// Checked app mount, build and external bootstrap module.
    assets: PageAssets,
    /// Synchronous component factory; async work belongs to guarded Resources.
    component: F,
}

impl<F> AsyncLeptosRenderer<F> {
    /// Validate an explicit buffered renderer without changing executor startup.
    /// Native factories must retain only Send/Sync reactive state, because this
    /// future may migrate threads. LocalStorage is client-only at this boundary.
    ///
    /// # Errors
    /// Rejects invalid page/assets policy or incompatible build identities.
    pub fn new<T, V>(
        contract: PageContract,
        assets: PageAssets,
        component: F,
    ) -> Result<Self, ViewError>
    where
        F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
        V: IntoView,
    {
        assets.validate(&contract)?;
        Ok(Self {
            contract,
            assets,
            component,
        })
    }
}

impl<T: Serialize, F, V> AsyncPageRenderer<T> for AsyncLeptosRenderer<F>
where
    F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
    V: IntoView,
{
    /// The page adapter retains its existing typed loader and lifecycle contract.
    fn contract(&self) -> &PageContract {
        &self.contract
    }

    /// Stage synchronously, then return a Send future owning all render state.
    fn render_async(&self, snapshot: PageSnapshot<T>) -> AsyncRenderFuture<'_> {
        let staged = prepare(&snapshot, &self.contract, &self.assets, || {
            (self.component)(&snapshot)
        });
        Box::pin(async move { staged?.await })
    }
}

/// Stage a Send generic assembly future after page and asset admission.
fn prepare<T: Serialize, V: IntoView>(
    snapshot: &PageSnapshot<T>,
    contract: &PageContract,
    assets: &PageAssets,
    component: impl FnOnce() -> Result<V, ViewError>,
) -> Result<AsyncRenderFuture<'static>, ViewError> {
    require_executor()?;
    assets.validate(contract)?;
    snapshot.validate(contract.revision, &contract.build)?;
    if snapshot.locale != contract.locale || snapshot.theme != contract.theme {
        return Err(ViewError::ContractMismatch);
    }
    let seed = snapshot.script_json_with(contract.limits)?;
    let bootstrap = assets.script_json()?;
    let module = assets.base.join(assets.module.as_str())?;
    let locale = snapshot.locale.clone();
    let theme = snapshot.theme.as_deref().unwrap_or("light").to_owned();
    // The unset marker permits one initial sample while the no-JS document remains readable Light.
    let theme_marker = if snapshot.theme.is_none() {
        " data-theme-unset"
    } else {
        ""
    };
    // Saved explicit choices are complete SSR output; only unset admits the external sampler.
    let theme_head = if snapshot.theme.is_none() {
        crate::theme::bootstrap_tag(&assets.base, assets.theme_bootstrap.as_ref(), None)?
    } else {
        String::new()
    };
    let limits = contract.limits;
    let render=AsyncRenderer::try_prepare(render_limits(limits),||{
        super::routing::provide_route_context(assets.base.clone());
        provide_context(leptos_router::location::RequestUrl::new(&snapshot.location));
        component()
    },||{
        // Resolve the checked stylesheet under the same mount as the bootstrap.
        if let Some(style) = &assets.stylesheet {
            let href = assets.base.join(style.as_str())?.as_str().to_owned();
            let _ = view! { <leptos_meta::Stylesheet id="stackpress-style" href/> }.to_html();
        }
        Ok(())
    },move|content,replay|{
        let html = format!(
            "<!DOCTYPE html><html lang=\"{locale}\" data-theme=\"{theme}\"{theme_marker}><head>{theme_head}<meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><div id=\"app\">{content}</div><script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"application/json\" id=\"stackpress-resources\">{replay}</script><script type=\"application/json\" id=\"stackpress-assets\">{bootstrap}</script><script type=\"module\" src=\"{}\"></script></body></html>",
            escape_html(module.as_str())
        );
        Ok(html)
    }).map_err(map_error)?;
    Ok(Box::pin(async move {
        RenderedPage::with_limits(render.await.map_err(map_error)?, limits)
    }))
}
