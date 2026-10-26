//! Synchronous component evaluation followed by owned metadata assembly.

use futures::{StreamExt, stream};
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
/// assembly only; it does not enable general asynchronous component SSR.
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
fn require_executor() -> Result<(), ViewError> {
    if !INITIALIZED.load(Ordering::Acquire) || tokio::runtime::Handle::try_current().is_err() {
        return Err(ViewError::ExecutorUnavailable);
    }
    Ok(())
}

/// An owned staging buffer; no Owner, request or host service crosses its await.
pub struct PreparedPage {
    /// Complete body/seed shell awaiting leptos_meta's buffered Head injection.
    html: String,
    /// Channel output and owned title state, independent of reactive Owner lifetime.
    head: ServerMetaContextOutput,
    /// The same lowerable policy guards the final fully assembled document.
    limits: PageLimits,
}

/// Synchronous render entry; each call receives a fresh, unshared Owner.
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
    ///
    /// # Errors
    /// Rejects the final complete HTML ceiling before the host can publish bytes.
    pub async fn finish(self) -> Result<RenderedPage, ViewError> {
        require_executor()?;
        // Only owned Send buffers survive this metadata tick; all rendering is already done.
        let chunks = self
            .head
            .inject_meta_context(stream::iter([self.html]))
            .await;
        let html = chunks.collect::<Vec<_>>().await.concat();
        RenderedPage::with_limits(html, self.limits)
    }
}

/// Scope guard makes cleanup explicit even when a trusted component panics.
struct RenderOwner(Owner);

impl Drop for RenderOwner {
    /// Clean signals/effects before the render exits or unwinds.
    fn drop(&mut self) {
        self.0.cleanup();
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
    let owner = RenderOwner(Owner::new());
    let (meta, head) = ServerMetaContext::new();
    let body = owner.0.with(|| {
        provide_context(meta);
        leptos_meta::provide_meta_context();
        crate::routing::provide_route_context(assets.base.clone());
        // Router SSR sees only the canonical public path, never an origin/cookie/header dump.
        provide_context(leptos_router::location::RequestUrl::new(&snapshot.location));
        let body = component()?.to_html();
        // leptos_meta 0.8.7 injects title text as raw text. Reserve its final formatter
        // for escaping so an untrusted plain-text title cannot terminate the title tag.
        let _ = view! { <leptos_meta::Title formatter=escape_html/> }.to_html();
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
        "<!DOCTYPE html><html lang=\"{}\" data-theme=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><div id=\"app\">{body}</div><script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"application/json\" id=\"stackpress-assets\">{bootstrap}</script><script type=\"module\" src=\"{}\"></script></body></html>",
        snapshot.locale,
        snapshot.theme,
        escape_html(module.into())
    );
    Ok(PreparedPage {
        html,
        head,
        limits: contract.limits,
    })
}

/// Encode plain text for HTML text/attribute contexts; browser Title uses its safe DOM API.
fn escape_html(value: String) -> String {
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
