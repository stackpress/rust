//! Complete buffered Leptos rendering with a request-local Owner retained through Head.
use crate::{
    owner::RestoreOwner,
    render::{RenderError, RenderLimits, RenderPanic, require_executor},
    safety::escape_html,
};
use futures::{FutureExt, StreamExt};
use hydration_context::{SharedContext, SsrSharedContext};
use leptos::prelude::*;
use leptos_meta::ServerMetaContext;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
/// One owned Send assembly future; a local view never crosses this boundary.
pub type RenderFuture<E> =
    Pin<Box<dyn Future<Output = Result<String, RenderError<E>>> + Send + 'static>>;
/// Explicit buffered Resource/Suspense execution without page metadata.
pub struct AsyncRenderer;
impl AsyncRenderer {
    /// Construct local view state synchronously, retaining the root until assembly ends.
    /// Retained native reactive state must be Send/Sync because the owned future may
    /// migrate threads. LocalStorage/Rc state is synchronous or client-only. Cleanup
    /// is cooperative; the host must keep its executor alive to poll awakened loaders.
    /// # Errors
    /// Rejects executor/policy/factory errors before returning the owned future.
    /// # Panics
    /// Synchronous factory unwind propagates after root cleanup; async unwind is consumed.
    pub fn try_prepare<V: IntoView, E: Send + 'static>(
        limits: RenderLimits,
        component: impl FnOnce() -> Result<V, E>,
        after_body: impl FnOnce() -> Result<(), E>,
        shell: impl FnOnce(String, String) -> Result<String, E> + Send + 'static,
    ) -> Result<OwnedRender<E>, RenderError<E>> {
        require_executor()?;
        limits.validate()?;
        let shared = Arc::new(SsrSharedContext::new());
        let _restore = RestoreOwner(Owner::current());
        let owner = Owner::new_root(Some(shared.clone()));
        let mut render = OwnedRender {
            owner: owner.clone(),
            shared: shared.clone(),
            inner: None,
        };
        let (meta, head) = ServerMetaContext::new();
        let body = owner.with(|| {
            provide_context(meta);
            leptos_meta::provide_meta_context();
            let body = component()
                .map_err(RenderError::Component)?
                .to_html_stream_in_order();
            let _ =
                view! {<leptos_meta::Title formatter=|value:String|escape_html(&value)/>}.to_html();
            after_body().map_err(RenderError::Component)?;
            Ok::<_, RenderError<E>>(body)
        })?;
        let assembly = async move {
            let mut body = Box::pin(body);
            let mut content = String::new();
            while let Some(chunk) = body.next().await {
                if content.len().saturating_add(chunk.len()) > limits.html_bytes {
                    return Err(RenderError::HtmlLimit);
                }
                content.push_str(&chunk);
            }
            let values = shared
                .consume_buffers()
                .await
                .into_iter()
                .map(|(id, value)| (id.into_inner(), value))
                .collect();
            let replay = crate::replay::ResourceReplay::new(values, limits.replay_bytes)
                .map_err(RenderError::Replay)?
                .script_json()
                .map_err(RenderError::Replay)?;
            let html = shell(content, replay).map_err(RenderError::Component)?;
            let html = head
                .inject_meta_context(futures::stream::iter([html]))
                .await
                .collect::<String>()
                .await;
            limits.finish(html)
        };
        render.inner = Some(Box::pin(async move {
            std::panic::AssertUnwindSafe(assembly)
                .catch_unwind()
                .await
                .map_err(|p| {
                    RenderError::from_source(RenderPanic::new(
                        "asynchronous component or Head evaluation panicked",
                        p.as_ref(),
                    ))
                })?
        }));
        Ok(render)
    }
}
/// One buffered future owns both root and serializer-cycle teardown.
pub struct OwnedRender<E> {
    /// Request-local root retained until complete Head/body assembly or cancellation.
    owner: Owner,
    /// Serialization/deferred queues explicitly cleared to break Resource cycles.
    shared: Arc<SsrSharedContext>,
    /// Taken before Owner cleanup so no assembly borrow survives disposal.
    inner: Option<RenderFuture<E>>,
}
impl<E> Future for OwnedRender<E> {
    type Output = Result<String, RenderError<E>>;
    /// Enter the root only for a synchronous poll, restoring the actual caller afterwards.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let _restore = RestoreOwner(Owner::current());
        let owner = self.owner.clone();
        owner.with(|| {
            self.inner
                .as_mut()
                .expect("future not polled after completion")
                .as_mut()
                .poll(cx)
        })
    }
}
impl<E> Drop for OwnedRender<E> {
    /// Drop assembly first, wake guarded loaders, then break serializer/deferred cycles.
    fn drop(&mut self) {
        let _restore = RestoreOwner(Owner::current());
        self.inner.take();
        self.owner.cleanup();
        let drain = std::panic::AssertUnwindSafe(self.shared.consume_buffers()).catch_unwind();
        drop(drain.now_or_never());
        drop(self.shared.await_deferred());
    }
}
