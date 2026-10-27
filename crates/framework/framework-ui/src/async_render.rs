//! Buffered component Resource/Suspense rendering, separate from synchronous SSR.
//!
//! Loaders must use [`crate::resources::cancel_on_cleanup`], return owned public
//! data, remain yielding/bounded, and never mutate another root's UI state.
//! Native component state retained in the Owner must be Send/Sync; thread-local
//! LocalStorage/Rc state must remain synchronous or client-only. Each
//! poll enters/restores its own Owner; no entered context guard crosses await.
//! Drop signals cooperative cancellation. The host must keep its executor alive
//! to poll awakened tasks; Drop cannot synchronously join them or interrupt CPU
//! work. This API buffers a complete page and makes no streaming claim.

use crate::render::{escape_html, require_executor};
use futures::{FutureExt, StreamExt};
use hydration_context::{SharedContext, SsrSharedContext};
use leptos::prelude::*;
use leptos_meta::ServerMetaContext;
use serde::Serialize;
use stackpress_framework_view::{
    AsyncPageRenderer, AsyncRenderFuture, PageAssets, PageContract, PageSnapshot, RenderedPage,
    ViewError,
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

/// Explicit complete-page renderer; the existing synchronous renderer is unchanged.
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

/// One buffered future owns root cleanup, independently of where it is polled.
struct OwnedRender {
    /// Root remains alive through body, replay and reactive Head assembly.
    owner: Owner,
    /// Own replay queues explicitly so cancellation can break Resource/context cycles.
    shared: Arc<SsrSharedContext>,
    /// An erased Send assembly future contains no caller/request borrow.
    inner: Option<AsyncRenderFuture<'static>>,
}

/// Restore the actual poll/drop caller even if upstream with/cleanup unwinds.
pub(super) struct Restore(pub(super) Option<Owner>);

impl Drop for Restore {
    /// Never retain a thread-local entered context across an await boundary.
    fn drop(&mut self) {
        if let Some(previous) = &self.0 {
            previous.set();
        } else if let Some(current) = Owner::current() {
            current.unset();
        }
    }
}

impl Future for OwnedRender {
    type Output = Result<RenderedPage, ViewError>;

    /// Only the synchronous poll runs under the render Owner.
    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let _restore = Restore(Owner::current());
        let owner = self.owner.clone();
        owner.with(|| {
            self.inner
                .as_mut()
                .expect("future not polled after completion")
                .as_mut()
                .poll(context)
        })
    }
}

impl Drop for OwnedRender {
    /// Drop assembly references first, then wake all guarded Resource loaders.
    fn drop(&mut self) {
        let _restore = Restore(Owner::current());
        self.inner.take();
        self.owner.cleanup();
        // In pinned hydration_context, consume_buffers takes both queues before
        // its first await. Poll once then drop to discard serialization waiters
        // even when an admitted Resource never resolves. Otherwise context ->
        // ready future -> Resource Owner -> context forms a retention cycle.
        // A cancelled render cannot publish/retry a panicking custom serializer.
        let drain = std::panic::AssertUnwindSafe(self.shared.consume_buffers()).catch_unwind();
        drop(drain.now_or_never());
        // Blocking-resource waiters have an independent queue with the same cycle.
        drop(self.shared.await_deferred());
    }
}

/// Build a Send stream while the factory's possibly local view is still synchronous.
fn prepare<T: Serialize, V: IntoView>(
    snapshot: &PageSnapshot<T>,
    contract: &PageContract,
    assets: &PageAssets,
    component: impl FnOnce() -> Result<V, ViewError>,
) -> Result<OwnedRender, ViewError> {
    require_executor()?;
    assets.validate(contract)?;
    snapshot.validate(contract.revision, &contract.build)?;
    if snapshot.locale != contract.locale || snapshot.theme != contract.theme {
        return Err(ViewError::ContractMismatch);
    }
    let seed = snapshot.script_json_with(contract.limits)?;
    let bootstrap = assets.script_json()?;
    let module = assets.base.join(assets.module.as_str())?;
    let shared = Arc::new(SsrSharedContext::new());
    let _restore = Restore(Owner::current());
    let owner = Owner::new_root(Some(shared.clone()));
    // Install the cleanup guard before any trusted component evaluation can unwind.
    let mut render = OwnedRender {
        owner: owner.clone(),
        shared: shared.clone(),
        inner: None,
    };
    let (meta, head) = ServerMetaContext::new();
    let body = owner.with(|| {
        provide_context(meta);
        leptos_meta::provide_meta_context();
        crate::routing::provide_route_context(assets.base.clone());
        provide_context(leptos_router::location::RequestUrl::new(&snapshot.location));
        let stream = component()?.to_html_stream_in_order();
        // The final owned formatter protects plain-text reactive titles too.
        let _ =
            view! { <leptos_meta::Title formatter=|value: String| escape_html(&value)/> }.to_html();
        // Resolve the checked stylesheet under the same mount as the bootstrap.
        if let Some(style) = &assets.stylesheet {
            let href = assets.base.join(style.as_str())?.as_str().to_owned();
            let _ = view! { <leptos_meta::Stylesheet id="stackpress-style" href/> }.to_html();
        }
        Ok::<_, ViewError>(stream)
    })?;
    let locale = snapshot.locale.clone();
    let theme = snapshot.theme.clone();
    let limits = contract.limits;
    let assembly = async move {
        // Bound accumulation while polling; arbitrary component CPU/allocation is trusted.
        let mut body = Box::pin(body);
        let mut content = String::new();
        while let Some(chunk) = body.next().await {
            if content.len().saturating_add(chunk.len()) > limits.html_bytes {
                return Err(ViewError::HtmlLimit);
            }
            content.push_str(&chunk);
        }
        // Public API yields raw serialized values; no inline script/eval is necessary.
        let values = shared
            .consume_buffers()
            .await
            .into_iter()
            .map(|(id, value)| (id.into_inner(), value))
            .collect();
        let replay =
            crate::replay::ResourceReplay::new(values, limits.snapshot_bytes)?.script_json()?;
        let html = format!(
            "<!DOCTYPE html><html lang=\"{locale}\" data-theme=\"{theme}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><div id=\"app\">{content}</div><script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"application/json\" id=\"stackpress-resources\">{replay}</script><script type=\"application/json\" id=\"stackpress-assets\">{bootstrap}</script><script type=\"module\" src=\"{}\"></script></body></html>",
            escape_html(module.as_str())
        );
        let html = head
            .inject_meta_context(futures::stream::iter([html]))
            .await
            .collect::<String>()
            .await;
        RenderedPage::with_limits(html, limits)
    };
    // A failed future is consumed once, never resumed or retried after panic.
    render.inner = Some(Box::pin(async move {
        std::panic::AssertUnwindSafe(assembly)
            .catch_unwind()
            .await
            .map_err(|payload| ViewError::from_source(AsyncRenderPanic::new(payload.as_ref())))?
    }));
    Ok(render)
}

/// Safe developer cause for an unwound async component or reactive Head getter.
#[derive(Debug)]
struct AsyncRenderPanic(String);
impl AsyncRenderPanic {
    /// Keep textual panic diagnostics as a native cause without exposing them publicly.
    fn new(payload: &(dyn std::any::Any + Send)) -> Self {
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("non-text panic payload");
        Self(message.to_owned())
    }
}
impl std::fmt::Display for AsyncRenderPanic {
    /// Public ViewError Display stays separate from this native cause.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("asynchronous component or Head evaluation panicked: ")?;
        formatter.write_str(&self.0)
    }
}
impl std::error::Error for AsyncRenderPanic {}
