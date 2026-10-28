//! Opt-in in-order SSR: owned precommit Head/status, then bounded Send chunks.
//!
//! Buffered SSR remains the default. Each page owns a private current-thread
//! runtime and LocalSet: the component/Owner never leaves that thread. A
//! two-chunk channel is the only bridge to the server's Send body. First body
//! demand starts production, after response hooks. Drop cancels, joins the
//! supervisor thread and tears down its runtime, including upstream Resource
//! tasks. No render task is detached from this body.
//!
//! Loaders must use [`crate::resources::cancel_on_cleanup`] and remain yielding
//! and bounded. Native Drop joins synchronously; it cannot preempt blocking CPU
//! work or a blocking destructor. Hosts must enforce admission and output
//! deadlines (the server-http host does). Late Head changes are not published.
//! Bootstrap must validate/merge `data-stackpress-replay` templates, remove them
//! before hydration, and hydrate only after `stackpress-stream-complete` appears.
//! On interruption the precommitted recovery notice remains available; no write
//! is retried, status changed, or error lifecycle reentered.
use crate::{
    owner::RestoreOwner,
    render::{escape_html, require_executor},
};
use futures::{FutureExt, Stream, StreamExt, future::poll_fn};
use hydration_context::SharedContext;
use leptos::prelude::*;
use leptos_meta::ServerMetaContext;
use serde::Serialize;
use stackpress_framework_view::{
    PageAssets, PageContract, PageSnapshot, PublicLocation, ViewError,
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    thread::JoinHandle,
};
use tokio::sync::{mpsc, oneshot};
#[path = "stream_replay.rs"]
mod replay;
use replay::{ReplayBudget, StreamContext};

/// Fixed inert failure signal; diagnostic details never enter HTML.
const FAILURE: &str = "<template id=\"stackpress-stream-failed\"></template>";

/// Optional external classic script for pre-completion recovery status.
///
/// Hosts serve these fixed bytes as a same-origin JavaScript asset, permit that
/// source in their CSP, and select it with [`StreamLeptosRenderer::with_status_script`].
/// It never hydrates, decodes replay data, retries a request, or executes page data.
/// Its six-second notice describes unfinished delivery, not a transport diagnosis.
/// With scripting disabled or the asset blocked, the static reload link remains.
pub const STREAM_STATUS_SCRIPT: &str = include_str!("stream_status.js");

/// Finite native rendering policy; aggregate HTML/replay limits come from PageContract.
///
/// Defaults are 64 KiB per fragment and 30 seconds for staging + production.
/// Zero is invalid, including for the deadline; it never means unlimited.
/// Reserve the fixed failure marker inside the aggregate HTML budget. Host
/// transport deadlines can be tighter and still cancel/reap this renderer.
#[derive(Clone, Copy, Debug)]
pub struct StreamLimits {
    /// Maximum owned fragment size, including its escaped replay template.
    pub max_chunk_bytes: usize,
    /// Absolute precommit + production deadline. This also bounds a pinned
    /// Leptos internal effect panic whose completion channel remains pending.
    pub render_timeout: std::time::Duration,
}
impl Default for StreamLimits {
    /// Use 64 KiB fragments and a finite thirty-second render deadline.
    fn default() -> Self {
        Self {
            max_chunk_bytes: 64 * 1024,
            render_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Checked optional renderer; it does not alter either buffered renderer trait.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_ui::stream::{StreamLeptosRenderer, StreamLimits};
/// use stackpress_framework_view::{BasePath, PageAssets, PageContract, PageSnapshot, PublicLocation};
/// let contract = PageContract::new(1, "app-build")?;
/// let assets = PageAssets {
///     theme_bootstrap: None,
///     build: "app-build".into(), base: BasePath::default(),
///     module: PublicLocation::new("/start.js")?, stylesheet: None,
/// };
/// let _renderer = StreamLeptosRenderer::new(contract, assets, StreamLimits::default(), |page: &PageSnapshot<String>| {
///     Ok(view! { <p>{page.data.clone()}</p> })
/// })?;
/// # Ok::<(), stackpress_framework_view::ViewError>(())
/// ```
pub struct StreamLeptosRenderer<F> {
    /// Shared checked seed/HTML policy.
    contract: PageContract,
    /// Same checked external bootstrap/mount as buffered SSR.
    assets: PageAssets,
    /// Optional mounted external status asset; never executable inline content.
    status_script: Option<PublicLocation>,
    /// Explicit owned-chunk policy.
    policy: StreamLimits,
    /// Factory crosses the thread boundary; its local result never does.
    component: Arc<F>,
}
impl<F> StreamLeptosRenderer<F> {
    /// Validate a streamed renderer without starting a thread or evaluating a view.
    ///
    /// # Errors
    /// Rejects invalid assets/contracts or a chunk too small for safe recovery.
    pub fn new<T, V>(
        contract: PageContract,
        assets: PageAssets,
        policy: StreamLimits,
        component: F,
    ) -> Result<Self, ViewError>
    where
        F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
        V: IntoView,
    {
        assets.validate(&contract)?;
        if policy.render_timeout.is_zero()
            || std::time::Instant::now()
                .checked_add(policy.render_timeout)
                .is_none()
            || policy.max_chunk_bytes < FAILURE.len()
            || policy.max_chunk_bytes > contract.limits.html_bytes
        {
            return Err(ViewError::InvalidPolicy);
        }
        Ok(Self {
            contract,
            assets,
            status_script: None,
            policy,
            component: Arc::new(component),
        })
    }

    /// Opt into an early external status script without changing bootstrap ordering.
    ///
    /// The supplied application-relative asset is joined to the checked base.
    /// The host must serve [`STREAM_STATUS_SCRIPT`] there and explicitly allow its
    /// source in CSP. This does not add nonce support or relax an existing policy.
    /// Default renderers emit no status script and retain their existing assets.
    ///
    /// # Errors
    /// Rejects a joined location exceeding the public-location policy.
    pub fn with_status_script(mut self, script: PublicLocation) -> Result<Self, ViewError> {
        // Freeze the mounted location before starting any render or worker thread.
        self.status_script = Some(self.assets.base.join(script.as_str())?);
        Ok(self)
    }

    /// Stage owned Head/status before handing an exclusive Send body to the host.
    ///
    /// The host calls `HttpResponseExt::stream` with this body's mapped errors,
    /// applies `status`, and lets its one normal response lifecycle finish. A
    /// failed staging future yields an ordinary fresh-response error. Cancelling
    /// staging also joins the owned worker. Head is frozen before publication.
    ///
    /// # Errors
    /// Returns checked seed/policy/factory/Head errors before body publication.
    /// Factory/body-poll panic diagnostics are retained only as a ViewError source.
    /// A panic in Leptos's internal effect task is observed as deadline expiry:
    /// its discarded task handle is not joinable through stable public APIs.
    pub async fn render<T, V>(&self, snapshot: PageSnapshot<T>) -> Result<StreamPage, ViewError>
    where
        T: Serialize + Send + 'static,
        F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
        V: IntoView,
    {
        require_executor()?;
        snapshot.validate(self.contract.revision, &self.contract.build)?;
        if snapshot.locale != self.contract.locale || snapshot.theme != self.contract.theme {
            return Err(ViewError::ContractMismatch);
        }
        let seed = snapshot.script_json_with(self.contract.limits)?;
        let contract = self.contract.clone();
        let assets = self.assets.clone();
        let status_script = self.status_script.clone();
        let component = self.component.clone();
        let policy = self.policy;
        let status = snapshot.status.code();
        let (ready_send, ready) = oneshot::channel();
        let (cancel, cancelled) = oneshot::channel();
        let (start, started) = oneshot::channel();
        let (send, receive) = mpsc::channel(2);
        // Own the thread immediately after spawn; cancellation at every await joins it.
        let input = RenderInput {
            snapshot,
            seed,
            contract,
            assets,
            status_script,
            component,
            policy,
        };
        let bridge = Bridge {
            ready: Some(ready_send),
            send,
            cancelled: Some(cancelled),
            started,
        };
        let thread = std::thread::Builder::new()
            .name("stackpress-ssr".into())
            .spawn(move || {
                worker(input, bridge);
            })
            .map_err(ViewError::from_source)?;
        let supervisor = Supervisor {
            cancel: Some(cancel),
            thread: Some(thread),
        };
        let prefix = ready.await.map_err(|_| {
            ViewError::from_source(std::io::Error::other(
                "SSR supervisor stopped before staging",
            ))
        })??;
        Ok(StreamPage {
            status,
            body: StreamBody {
                prefix: Some(prefix),
                start: Some(start),
                receive,
                finished: false,
                supervisor,
            },
        })
    }
}

/// Precommitted application status and one exclusive owned body.
pub struct StreamPage {
    /// Apply once before handing the stream to the HTTP lifecycle.
    pub status: u16,
    /// Owned Send stream; dropping it cancels and reaps all render work.
    pub body: StreamBody,
}

/// One response's bounded receiver and its complete supervision ownership.
pub struct StreamBody {
    /// Owned frozen document opening; no render object crosses the bridge.
    prefix: Option<Vec<u8>>,
    /// Producer may start only when the published body is first demanded.
    start: Option<oneshot::Sender<()>>,
    /// Exactly two chunks can wait ahead of a slow consumer.
    receive: mpsc::Receiver<BridgeMessage>,
    /// Only an explicit completion message is successful EOF; failure is fused.
    finished: bool,
    /// Owns cancellation and join, including when the body is never polled.
    supervisor: Supervisor,
}
impl Stream for StreamBody {
    type Item = Result<Vec<u8>, ViewError>;
    /// Return owned bytes only; this Send stream never polls a local Leptos view.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(prefix) = self.prefix.take() {
            if let Some(start) = self.start.take() {
                let _ = start.send(());
            }
            return Poll::Ready(Some(Ok(prefix)));
        }
        if self.finished {
            return Poll::Ready(None);
        }
        match self.receive.poll_recv(cx) {
            Poll::Ready(Some(BridgeMessage::Chunk(bytes))) => Poll::Ready(Some(Ok(bytes))),
            Poll::Ready(Some(BridgeMessage::Complete(bytes))) => {
                self.finished = true;
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Ready(Some(BridgeMessage::Failure(error))) => {
                self.finished = true;
                Poll::Ready(Some(Err(error)))
            }
            Poll::Ready(None) => {
                self.finished = true;
                Poll::Ready(Some(Err(ViewError::from_source(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "SSR supervisor ended without completion",
                )))))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
impl Drop for StreamBody {
    /// Close capacity waiters before signalling cancellation and joining.
    fn drop(&mut self) {
        self.receive.close();
        self.supervisor.stop();
    }
}

/// Both staging futures and published bodies retain this join-on-drop guard.
struct Supervisor {
    /// Cancellation wins even if the producer is blocked by channel capacity.
    cancel: Option<oneshot::Sender<()>>,
    /// A joinable dedicated executor owns upstream detached Resource tasks too.
    thread: Option<JoinHandle<()>>,
}
impl Supervisor {
    /// Runtime teardown completes on the worker before join returns.
    fn stop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Drop for Supervisor {
    /// Cancellation before precommit cannot abandon a worker thread.
    fn drop(&mut self) {
        self.stop();
    }
}

/// Owner and serialization queues are always disposed before runtime teardown.
struct Root {
    /// Local reactive root, cleaned before runtime teardown.
    owner: Owner,
    /// Serialization/deferred queues cleared after root cleanup to break cycles.
    shared: Arc<StreamContext>,
}
impl Root {
    /// Poll only under this root and restore the actual caller even on unwind.
    async fn run<F: Future>(&self, future: F) -> F::Output {
        let mut future = std::pin::pin!(future);
        poll_fn(|cx| {
            let _restore = RestoreOwner(Owner::current());
            self.owner.with(|| future.as_mut().poll(cx))
        })
        .await
    }
}
impl Drop for Root {
    /// Wake guarded loaders, then break serialization/Owner retention cycles.
    fn drop(&mut self) {
        let _restore = RestoreOwner(Owner::current());
        self.owner.cleanup();
        self.shared.clear();
    }
}

/// Immutable admitted input; the view is constructed only inside the worker.
struct RenderInput<T, F> {
    /// Checked public projection, never a live invocation.
    snapshot: PageSnapshot<T>,
    /// Bounded escaped public seed, prepared before thread admission.
    seed: String,
    /// Aggregate output and identity policy.
    contract: PageContract,
    /// Checked mounted external bootstrap assets.
    assets: PageAssets,
    /// Optional mounted status asset, independent of the completion-gated module.
    status_script: Option<PublicLocation>,
    /// Send/Sync factory whose produced view may remain local.
    component: Arc<F>,
    /// Lowerable chunk and finite render deadline policy.
    policy: StreamLimits,
}

/// Completion is an explicit protocol event, never inferred from sender Drop.
enum BridgeMessage {
    /// An ordinary bounded owned fragment.
    Chunk(Vec<u8>),
    /// Final checked trailer; no further message may be published.
    Complete(Vec<u8>),
    /// A single safe error after the optional failure marker.
    Failure(ViewError),
}

/// The worker owns one end of each channel; the body owns the other end.
struct Bridge {
    /// Taken exactly once: fresh precommit error or owned opening bytes.
    ready: Option<oneshot::Sender<Result<Vec<u8>, ViewError>>>,
    /// The two-slot bounded output queue.
    send: mpsc::Sender<BridgeMessage>,
    /// Drop wakes the worker even if it is waiting for capacity or a Resource.
    cancelled: Option<oneshot::Receiver<()>>,
    /// First body demand gates production after the response lifecycle.
    started: oneshot::Receiver<()>,
}

/// Construct the local executor, supervise its work, then tear it down in place.
fn worker<T, F, V>(input: RenderInput<T, F>, mut bridge: Bridge)
where
    T: Serialize + 'static,
    F: Fn(&PageSnapshot<T>) -> Result<V, ViewError>,
    V: IntoView,
{
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            if let Some(ready) = bridge.ready.take() {
                let _ = ready.send(Err(ViewError::from_source(error)));
            }
            return;
        }
    };
    let local = tokio::task::LocalSet::new();
    runtime.block_on(local.run_until(async {
        let shared = Arc::new(StreamContext::new());
        let root = Root { owner: Owner::new_root(Some(shared.clone())), shared };
        let deadline = tokio::time::sleep(input.policy.render_timeout);
        // Poll under the root, but move cancellation outside produce's mutable
        // bridge borrow so cancellation can always interrupt a capacity wait.
        let mut cancelled = bridge.cancelled.take().expect("worker takes cancellation once");
        let outcome = {
            let work = std::panic::AssertUnwindSafe(root.run(produce(input, &root, &mut bridge))).catch_unwind();
            tokio::select! {
                biased;
                _ = &mut cancelled => return,
                outcome = work => outcome,
                _ = deadline => Ok(Err(ViewError::from_source(std::io::Error::new(std::io::ErrorKind::TimedOut, "streamed render deadline exceeded")))),
            }
        };
        // Dispose the root before reporting failure. Resource serialization
        // cycles are broken before the failure marker enters the bridge.
        drop(root);
        let outcome = outcome.unwrap_or_else(|payload| Err(ViewError::from_source(StreamPanic::new(payload.as_ref()))));
        if let Err(error) = outcome {
            if let Some(ready) = bridge.ready.take() { let _ = ready.send(Err(error)); }
            else {
                // A slow/disconnected client cannot prevent supervisor teardown.
                tokio::select! {
                    biased;
                    _ = &mut cancelled => {},
                    _ = async {
                        if bridge.send.send(BridgeMessage::Chunk(FAILURE.as_bytes().to_vec())).await.is_ok() {
                            let _ = bridge.send.send(BridgeMessage::Failure(error)).await;
                        }
                    } => {}
                }
            }
        }
    }));
    // Both local tasks and upstream tokio::spawn tasks are reaped here, before
    // the supervisor thread can return to its body's join-on-drop guard.
    drop(local);
    drop(runtime);
}

/// Stage Head once, then publish in-order body/replay under bounded demand.
async fn produce<T, F, V>(
    input: RenderInput<T, F>,
    root: &Root,
    bridge: &mut Bridge,
) -> Result<(), ViewError>
where
    T: Serialize,
    F: Fn(&PageSnapshot<T>) -> Result<V, ViewError>,
    V: IntoView,
{
    let RenderInput {
        snapshot,
        seed,
        contract,
        assets,
        status_script,
        component,
        policy,
    } = input;
    let (meta, head) = ServerMetaContext::new();
    provide_context(meta);
    leptos_meta::provide_meta_context();
    crate::routing::provide_route_context(assets.base.clone());
    provide_context(leptos_router::location::RequestUrl::new(&snapshot.location));
    let mut body = Box::pin((component)(&snapshot)?.to_html_stream_in_order());
    let _ = view! { <leptos_meta::Title formatter=|value: String| escape_html(&value)/> }.to_html();
    if let Some(style) = &assets.stylesheet {
        let href = assets.base.join(style.as_str())?.as_str().to_owned();
        let _ = view! { <leptos_meta::Stylesheet id="stackpress-style" href/> }.to_html();
    }
    // Explicit blocking Resources delay commit; ordinary Suspense does not.
    while let Some(deferred) = root.shared.await_deferred() {
        deferred.await;
    }
    let module = assets.base.join(assets.module.as_str())?;
    let bootstrap = assets.script_json()?;
    // Async classic execution can update the notice while parsing is still pending.
    // Only checked external bytes execute; the ordinary module remains deferred.
    let status_script = status_script
        .map(|script| {
            format!(
                "<script async src=\"{}\"></script>",
                escape_html(script.as_str())
            )
        })
        .unwrap_or_default();
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
    let opening = format!(
        "<!DOCTYPE html><html lang=\"{}\" data-theme=\"{}\"{theme_marker}><head>{theme_head}<meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><aside id=\"stackpress-stream-recovery\" role=\"status\">Loading or interrupted; reload if this page does not finish. <a href=\"{}\">Reload this page</a>.</aside>{status_script}<script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"application/json\" id=\"stackpress-assets\">{bootstrap}</script><script type=\"module\" src=\"{}\"></script><div id=\"app\">",
        snapshot.locale,
        snapshot.theme.as_deref().unwrap_or("light"),
        escape_html(&snapshot.location),
        escape_html(module.as_str())
    );
    let opening = head
        .inject_meta_context(futures::stream::iter([opening]))
        .await
        .collect::<String>()
        .await;
    let mut budget = HtmlBudget {
        used: 0,
        limit: contract.limits.html_bytes,
        chunk: policy.max_chunk_bytes,
    };
    budget.check(opening.len())?;
    // Head/factory/seed and opening-byte failures are ordinary precommit errors.
    if bridge
        .ready
        .take()
        .expect("staging reports once")
        .send(Ok(opening.into_bytes()))
        .is_err()
    {
        return Ok(());
    }
    if (&mut bridge.started).await.is_err() {
        return Ok(());
    }
    let mut replay = ReplayBudget::new(contract.limits.snapshot_bytes);
    loop {
        // Reserve before polling rendering: capacity two bounds producer lookahead.
        let permit = match bridge.send.reserve().await {
            Ok(permit) => permit,
            Err(_) => return Ok(()),
        };
        let Some(content) = body.next().await else {
            break;
        };
        let values = poll_fn(|cx| Poll::Ready(root.shared.ready(cx))).await?;
        let content = content + &replay.append(values)?;
        budget.check(content.len())?;
        permit.send(BridgeMessage::Chunk(content.into_bytes()));
    }
    // Resources not represented by a Suspense section still need exact replay.
    loop {
        let permit = match bridge.send.reserve().await {
            Ok(permit) => permit,
            Err(_) => return Ok(()),
        };
        let tail = poll_fn(|cx| {
            let values = root.shared.ready(cx)?;
            let fragment = replay.append(values)?;
            if !fragment.is_empty() || root.shared.is_empty() {
                Poll::Ready(Ok::<_, ViewError>(fragment))
            } else {
                Poll::Pending
            }
        })
        .await?;
        if !tail.is_empty() {
            budget.check(tail.len())?;
            permit.send(BridgeMessage::Chunk(tail.into_bytes()));
        }
        if root.shared.is_empty() {
            break;
        }
    }
    let trailer =
        "</div><template id=\"stackpress-stream-complete\"></template></body></html>".to_owned();
    budget.check(trailer.len())?;
    let _ = bridge
        .send
        .send(BridgeMessage::Complete(trailer.into_bytes()))
        .await;
    Ok::<(), ViewError>(())
}

/// Aggregate and per-fragment ceilings, including a reserved late-failure marker.
struct HtmlBudget {
    /// Previously admitted bytes, including opening/replay/trailer fragments.
    used: usize,
    /// Whole-document ceiling; each admission also reserves the failure marker.
    limit: usize,
    /// Independent ceiling on one owned producer fragment.
    chunk: usize,
}
impl HtmlBudget {
    /// Reject before publishing a fragment; saturating arithmetic cannot wrap.
    fn check(&mut self, bytes: usize) -> Result<(), ViewError> {
        let next = self.used.saturating_add(bytes);
        if bytes > self.chunk || next.saturating_add(FAILURE.len()) > self.limit {
            return Err(ViewError::HtmlLimit);
        }
        self.used = next;
        Ok(())
    }
}

/// Native-only panic cause; safe ViewError Display/serialization remains unchanged.
#[derive(Debug)]
struct StreamPanic(String);
impl StreamPanic {
    /// Preserve textual diagnostics without exposing them in stream markers.
    fn new(payload: &(dyn std::any::Any + Send)) -> Self {
        Self(
            payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-text panic payload")
                .to_owned(),
        )
    }
}
impl std::fmt::Display for StreamPanic {
    /// Native source traversal is the only route to these diagnostics.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "streamed component evaluation panicked: {}", self.0)
    }
}
impl std::error::Error for StreamPanic {}

#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;
