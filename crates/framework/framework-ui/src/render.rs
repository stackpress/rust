//! Synchronous component evaluation followed by owned metadata assembly.
//!
//! SSR Head values must be eager owned strings, including Title text/formatters.
//! Resolve signals while evaluating the component. Reactive SSR Head belongs to
//! [`crate::async_render::AsyncLeptosRenderer`]. Head evaluation panic returns
//! [`stackpress_framework_view::ViewError::Failure`].

use crate::owner::RestoreOwner;
use futures::{FutureExt, StreamExt, stream};
use leptos::prelude::*;
use leptos_meta::{ServerMetaContext, ServerMetaContextOutput};
use serde::Serialize;
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageLimits, PageRenderer, PageSnapshot, PublicLocation,
    RenderFuture, RenderedPage, ViewError,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Only this documented helper marks the native render boundary ready.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize native SSR's process-global Leptos executor, idempotently.
///
/// Call once during host startup inside an active Tokio runtime. Repeated calls
/// and an already-initialized Leptos executor succeed. This enables buffered Head
/// assembly only; component-owned async work uses the explicit async renderer.
/// Synchronous SSR Head props must be eager owned values.
///
/// # Errors
/// Returns [`ViewError::ExecutorUnavailable`] outside an active Tokio runtime.
pub fn initialize_ssr() -> Result<(), ViewError> {
    tokio::runtime::Handle::try_current().map_err(|_| ViewError::ExecutorUnavailable)?;
    // AlreadySet is the executor's only error: another initialized caller is valid.
    match any_spawner::Executor::init_tokio() {
        Ok(()) | Err(any_spawner::ExecutorError::AlreadySet) => {
            INITIALIZED.store(true, Ordering::Release);
            Ok(())
        }
    }
}

/// Reject missing startup or runtime context before a dependency can panic.
pub(super) fn require_executor() -> Result<(), ViewError> {
    if !INITIALIZED.load(Ordering::Acquire) || tokio::runtime::Handle::try_current().is_err() {
        return Err(ViewError::ExecutorUnavailable);
    }
    Ok(())
}

/// Staged output without the render Owner; Head must obey the eager-data contract.
pub struct PreparedPage {
    /// Complete body/seed shell awaiting leptos_meta's buffered Head injection.
    html: String,
    /// Channel output and owned title state, independent of reactive Owner lifetime.
    head: ServerMetaContextOutput,
    /// The same lowerable policy guards the final fully assembled document.
    limits: PageLimits,
}

/// Synchronous render entry; each call receives a fresh, unshared Owner.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_ui::render::{LeptosRenderer};
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

impl PreparedPage {
    /// Finish leptos_meta's owned buffered Head injection, never component async SSR.
    /// Head values must be eager owned text; reactive Head needs the async renderer.
    ///
    /// # Errors
    /// Rejects missing SSR startup/runtime or the final complete HTML ceiling before
    /// the host can publish bytes. Consuming this staged page never retries Head work.
    /// A panicking Head getter/formatter returns [`ViewError::Failure`]; its native
    /// cause names the eager Head requirement, while public Display stays safe.
    pub async fn finish(self) -> Result<RenderedPage, ViewError> {
        require_executor()?;
        // No Owner or borrow guard survives this metadata tick. Catch only this
        // consumed assembly future: an unwind drops it; it is never resumed/retried.
        let limits = self.limits;
        let assembly = async move {
            let chunks = self
                .head
                .inject_meta_context(stream::iter([self.html]))
                .await;
            chunks.collect::<Vec<_>>().await.concat()
        };
        let html = std::panic::AssertUnwindSafe(assembly)
            .catch_unwind()
            .await
            .map_err(|payload| ViewError::from_source(HeadPanic::new(payload.as_ref())))?;
        RenderedPage::with_limits(html, limits)
    }
}

/// Native-only diagnostics for a failed getter; ViewError never displays this publicly.
#[derive(Debug)]
struct HeadPanic(String);

impl HeadPanic {
    /// Retain a textual panic cause without exposing arbitrary payloads in a response.
    fn new(payload: &(dyn std::any::Any + Send)) -> Self {
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("non-text panic payload");
        Self(format!(
            "SSR Head requires eager owned values (reactive SSR Head is S5-F6): {message}"
        ))
    }
}

impl std::fmt::Display for HeadPanic {
    /// Only explicit native cause traversal receives the getter's failure details.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl std::error::Error for HeadPanic {}

/// Scope guard owns cleanup and preserves the caller's thread-local reactive context.
struct RenderOwner {
    /// Fresh state belongs only to this synchronous component evaluation.
    owner: Owner,
    /// Strongly retain the prior Owner until restoration, including an unwind.
    previous: Option<Owner>,
}

impl RenderOwner {
    /// Capture context before creating the request-local child Owner.
    fn new() -> Self {
        Self {
            previous: Owner::current(),
            owner: Owner::new(),
        }
    }
}

impl Drop for RenderOwner {
    /// Clean signals/effects before the render exits or unwinds.
    fn drop(&mut self) {
        // Leptos 0.8's Owner::with restores only on normal return. Keep restoration
        // in its own guard so even a cleanup callback panic restores caller context.
        let _restore = RestoreOwner(self.previous.take());
        self.owner.cleanup();
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

/// Evaluate the component under a fresh Owner, then return only owned Send buffers.
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
        stackpress_framework_view::theme::bootstrap_tag(
            &assets.base,
            assets.theme_bootstrap.as_ref(),
            None,
        )?
    } else {
        String::new()
    };
    // The unset marker permits one initial sample while the no-JS document remains readable Light.
    let theme_marker = if snapshot.theme.is_none() {
        " data-theme-unset"
    } else {
        ""
    };
    let owner = RenderOwner::new();
    let (meta, head) = ServerMetaContext::new();
    let body = owner.owner.with(|| {
        provide_context(meta);
        leptos_meta::provide_meta_context();
        crate::routing::provide_route_context(assets.base.clone());
        // Router SSR sees only the canonical public path, never an origin/cookie/header dump.
        provide_context(leptos_router::location::RequestUrl::new(&snapshot.location));
        let body = component()?.to_html();
        // leptos_meta 0.8.7 injects title text as raw text. Reserve its final formatter
        // for escaping so an untrusted plain-text title cannot terminate the title tag.
        // The thin adapter retains Leptos's required owned-String callback contract.
        let _ =
            view! { <leptos_meta::Title formatter=|value: String| escape_html(&value)/> }.to_html();
        let _ =
            view! { <leptos_meta::Link rel="canonical" href=snapshot.location.clone()/> }.to_html();
        if let Some(style) = &assets.stylesheet {
            let href = assets.base.join(style.as_str())?.as_str().to_owned();
            let _ = view! { <leptos_meta::Stylesheet id="stackpress-style" href/> }.to_html();
        }
        Ok::<_, ViewError>(body)
    })?;
    drop(owner);
    // Every interpolated attribute is checked and escaped independently of JSON script text.
    let html = format!(
        "<!DOCTYPE html><html lang=\"{}\" data-theme=\"{}\"{theme_marker}><head>{theme_head}<meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><div id=\"app\">{body}</div><script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"application/json\" id=\"stackpress-assets\">{bootstrap}</script><script type=\"module\" src=\"{}\"></script></body></html>",
        snapshot.locale,
        snapshot.theme.as_deref().unwrap_or("light"),
        escape_html(module.as_str())
    );
    Ok(PreparedPage {
        html,
        head,
        limits: contract.limits,
    })
}

/// Encode plain text for HTML text/attribute contexts; browser Title uses its safe DOM API.
pub(super) fn escape_html(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => encoded.push_str("&amp;"),
            '<' => encoded.push_str("&lt;"),
            '>' => encoded.push_str("&gt;"),
            '"' => encoded.push_str("&quot;"),
            '\'' => encoded.push_str("&#39;"),
            other => encoded.push(other),
        }
    }
    encoded
}

#[cfg(test)]
mod escaping_tests {
    use super::escape_html;

    /// Escaping borrows reusable input and preserves Unicode and all HTML delimiters.
    #[test]
    fn escaping_borrows_text() {
        let input = String::from("雪 & < > \" '");
        assert_eq!(escape_html(&input), "雪 &amp; &lt; &gt; &quot; &#39;");
        assert_eq!(input, "雪 & < > \" '");
        assert_eq!(escape_html(""), "");
    }
}
