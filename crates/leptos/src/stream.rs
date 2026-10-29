//! Supervised native Leptos stream execution with bounded demand and owned teardown.
use crate::{
    owner::RestoreOwner,
    render::{RenderError, RenderLimits, RenderPanic, require_executor},
};
use futures::{FutureExt, Stream, StreamExt, future::poll_fn};
use hydration_context::SharedContext;
use leptos::prelude::*;
use leptos_meta::ServerMetaContextOutput;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    thread::JoinHandle,
};
use tokio::sync::{mpsc, oneshot};
#[path = "stream_replay.rs"]
mod replay;
use replay::{ReplayBudget, StreamContext};
/// Finite native rendering policy; aggregate HTML/replay limits come from RenderLimits.
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

/// Generic stream execution; the caller supplies trusted document framing.
pub struct StreamRenderer;
impl StreamRenderer {
    /// Stage a local Leptos render inside one owned worker, then return only Send bytes.
    /// The local view is constructed inside the worker; only the factory crosses
    /// threads. Dropping staging/body cancels and joins that worker. Trusted work
    /// must yield; blocking CPU or destructors cannot be preempted by cancellation.
    /// # Errors
    /// Rejects invalid limits, missing executor and worker/precommit failure.
    pub async fn render<E: Send + 'static, F, Fut>(
        limits: RenderLimits,
        policy: StreamLimits,
        failure: String,
        work: F,
    ) -> Result<StreamBody<E>, RenderError<E>>
    where
        F: FnOnce(StreamWriter<E>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), RenderError<E>>> + 'static,
    {
        require_executor()?;
        limits.validate()?;
        if policy.max_chunk_bytes == 0
            || policy.render_timeout.is_zero()
            || std::time::Instant::now()
                .checked_add(policy.render_timeout)
                .is_none()
            || policy.max_chunk_bytes < failure.len()
            || policy.max_chunk_bytes > limits.html_bytes
        {
            return Err(RenderError::InvalidPolicy);
        }
        let (ready_send, ready) = oneshot::channel();
        let ready_send = Arc::new(Mutex::new(Some(ready_send)));
        let (cancel, cancelled) = oneshot::channel();
        let (start, started) = oneshot::channel();
        let (send, receive) = mpsc::channel(2);
        let thread=std::thread::Builder::new().name("leptos-ssr".into()).spawn(move|| {
            let runtime=match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(runtime)=>runtime,Err(error)=>{if let Some(ready)=ready_send.lock().unwrap_or_else(|e|e.into_inner()).take(){let _=ready.send(Err(RenderError::from_source(error)));}return;}
            };
            let local=tokio::task::LocalSet::new();
            runtime.block_on(local.run_until(async {
                let shared=Arc::new(StreamContext::new());let root=Root{owner:Owner::new_root(Some(shared.clone())),shared:shared.clone()};
                let writer=StreamWriter{shared,ready:ready_send.clone(),send:send.clone(),started:Some(started),limits,budget:HtmlBudget{used:0,limit:limits.html_bytes,chunk:policy.max_chunk_bytes,reserve:failure.len()}};
                let deadline=tokio::time::sleep(policy.render_timeout);let mut cancelled=cancelled;
                let outcome={let work=std::panic::AssertUnwindSafe(root.run(async move {work(writer).await})).catch_unwind();tokio::select!{biased;_= &mut cancelled=>return,outcome=work=>outcome,_=deadline=>Ok(Err(RenderError::from_source(std::io::Error::new(std::io::ErrorKind::TimedOut,"streamed render deadline exceeded"))))}};
                drop(root);
                let outcome=outcome.unwrap_or_else(|p|Err(RenderError::from_source(RenderPanic::new("streamed component evaluation panicked",p.as_ref()))));
                if let Err(error)=outcome {
                    if let Some(ready)=ready_send.lock().unwrap_or_else(|e|e.into_inner()).take(){let _=ready.send(Err(error));}
                    else{tokio::select!{biased;_= &mut cancelled=>{},_=async{if send.send(BridgeMessage::Chunk(failure.into_bytes())).await.is_ok(){let _=send.send(BridgeMessage::Failure(error)).await;}}=>{}}}
                }
            }));drop(local);drop(runtime);
        }).map_err(RenderError::from_source)?;
        let supervisor = Supervisor {
            cancel: Some(cancel),
            thread: Some(thread),
        };
        let prefix = ready.await.map_err(|_| {
            RenderError::from_source(std::io::Error::other(
                "SSR supervisor stopped before staging",
            ))
        })??;
        Ok(StreamBody {
            prefix: Some(prefix),
            start: Some(start),
            receive,
            finished: false,
            supervisor,
        })
    }
}
/// A precommit result shared by the supervisor and local producer; consumed exactly once.
type ReadySignal<E> = Arc<Mutex<Option<oneshot::Sender<Result<Vec<u8>, RenderError<E>>>>>>;

/// Owned local producer context; only its resulting byte body crosses threads.
pub struct StreamWriter<E> {
    shared: Arc<StreamContext>,
    ready: ReadySignal<E>,
    send: mpsc::Sender<BridgeMessage<E>>,
    started: Option<oneshot::Receiver<()>>,
    limits: RenderLimits,
    budget: HtmlBudget,
}
impl<E> StreamWriter<E> {
    /// Freeze Head/opening, then render body and replay under two-slot demand.
    /// # Errors
    /// Rejects caller framing/replay/output failures before publishing each affected chunk.
    pub async fn render<S, Fr>(
        mut self,
        body: S,
        head: ServerMetaContextOutput,
        opening: String,
        mut frame: Fr,
        trailer: String,
    ) -> Result<(), RenderError<E>>
    where
        S: Stream<Item = String>,
        Fr: FnMut(String, String) -> Result<String, E>,
    {
        // Blocking Resources delay precommit; ordinary Suspense keeps its in-order semantics.
        while let Some(deferred) = self.shared.await_deferred() {
            deferred.await;
        }
        let opening = head
            .inject_meta_context(futures::stream::iter([opening]))
            .await
            .collect::<String>()
            .await;
        self.budget.check(opening.len())?;
        if self
            .ready
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .expect("staging reports once")
            .send(Ok(opening.into_bytes()))
            .is_err()
        {
            return Ok(());
        }
        if self.started.take().expect("starts once").await.is_err() {
            return Ok(());
        }
        let mut body = Box::pin(body);
        let mut replay = ReplayBudget::new(self.limits.replay_bytes);
        loop {
            // Reserve capacity before polling local rendering, bounding producer lookahead.
            let permit = match self.send.reserve().await {
                Ok(p) => p,
                Err(_) => return Ok(()),
            };
            let Some(content) = body.next().await else {
                break;
            };
            let values = poll_fn(|cx| Poll::Ready(self.shared.ready(cx)))
                .await
                .map_err(RenderError::Replay)?;
            let json = replay.append(values).map_err(RenderError::Replay)?;
            let content = frame(content, json).map_err(RenderError::Component)?;
            self.budget.check(content.len())?;
            permit.send(BridgeMessage::Chunk(content.into_bytes()));
        }
        loop {
            let permit = match self.send.reserve().await {
                Ok(p) => p,
                Err(_) => return Ok(()),
            };
            let tail = poll_fn(|cx| {
                let values = self.shared.ready(cx).map_err(RenderError::Replay)?;
                let fragment = replay.append(values).map_err(RenderError::Replay)?;
                if !fragment.is_empty() || self.shared.is_empty() {
                    Poll::Ready(Ok::<_, RenderError<E>>(fragment))
                } else {
                    Poll::Pending
                }
            })
            .await?;
            if !tail.is_empty() {
                let tail = frame(String::new(), tail).map_err(RenderError::Component)?;
                self.budget.check(tail.len())?;
                permit.send(BridgeMessage::Chunk(tail.into_bytes()));
            }
            if self.shared.is_empty() {
                break;
            }
        }
        self.budget.check(trailer.len())?;
        let _ = self
            .send
            .send(BridgeMessage::Complete(trailer.into_bytes()))
            .await;
        Ok(())
    }
}
/// Exclusive Send byte receiver; Drop cancels, closes capacity waits and joins the worker.
pub struct StreamBody<E> {
    prefix: Option<Vec<u8>>,
    start: Option<oneshot::Sender<()>>,
    receive: mpsc::Receiver<BridgeMessage<E>>,
    finished: bool,
    supervisor: Supervisor,
}
impl<E> Stream for StreamBody<E> {
    type Item = Result<Vec<u8>, RenderError<E>>;
    /// Only an explicit completion message establishes successful EOF.
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
            Poll::Ready(Some(BridgeMessage::Chunk(b))) => Poll::Ready(Some(Ok(b))),
            Poll::Ready(Some(BridgeMessage::Complete(b))) => {
                self.finished = true;
                Poll::Ready(Some(Ok(b)))
            }
            Poll::Ready(Some(BridgeMessage::Failure(e))) => {
                self.finished = true;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => {
                self.finished = true;
                Poll::Ready(Some(Err(RenderError::from_source(std::io::Error::other(
                    "SSR supervisor ended without completion",
                )))))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
impl<E> Drop for StreamBody<E> {
    /// Closing capacity waiters precedes cancellation and synchronous worker join.
    fn drop(&mut self) {
        self.receive.close();
        self.supervisor.stop();
    }
}
enum BridgeMessage<E> {
    Chunk(Vec<u8>),
    Complete(Vec<u8>),
    Failure(RenderError<E>),
}
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

/// Aggregate framed bytes reserve the caller's fixed late-failure signal.
struct HtmlBudget {
    used: usize,
    limit: usize,
    chunk: usize,
    reserve: usize,
}
impl HtmlBudget {
    /// Check saturating totals before a fragment can cross the native bridge.
    fn check<E>(&mut self, bytes: usize) -> Result<(), RenderError<E>> {
        let next = self.used.saturating_add(bytes);
        if bytes > self.chunk || next.saturating_add(self.reserve) > self.limit {
            return Err(RenderError::HtmlLimit);
        }
        self.used = next;
        Ok(())
    }
}

#[cfg(test)]
mod extracted_tests {
    use super::*;
    use std::convert::Infallible;
    const FAILURE: &str = "failure";
    /// Preserve exact budget, cumulative replay and overflow behavior after extraction.
    #[test]
    fn budget_replay_and_pending_overflow() {
        let mut budget = HtmlBudget {
            used: 0,
            limit: FAILURE.len() + 10,
            chunk: 10,
            reserve: FAILURE.len(),
        };
        budget.check::<Infallible>(10).unwrap();
        assert!(budget.check::<Infallible>(1).is_err());
        assert!(budget.check::<Infallible>(usize::MAX).is_err());
        let mut replay = ReplayBudget::new(1024);
        replay.append(vec![(0, "</template>&".into())]).unwrap();
        assert!(replay.append(vec![(0, "duplicate".into())]).is_err());
        let mut replay = ReplayBudget::new(10);
        assert!(replay.append(vec![(1, "too long".into())]).is_err());
        let context = StreamContext::new();
        for id in 0..65 {
            context.write_async(
                hydration_context::SerializedDataId::new(id),
                Box::pin(std::future::pending()),
            );
        }
        assert!(futures::executor::block_on(poll_fn(|cx| Poll::Ready(context.ready(cx)))).is_err());
    }
    /// A vanished worker and a delivered failure both produce one fused error.
    #[test]
    fn explicit_completion_protocol() {
        for message in [
            None,
            Some(BridgeMessage::Failure(RenderError::<Infallible>::HtmlLimit)),
            Some(BridgeMessage::Complete(b"finished".to_vec())),
        ] {
            let (send, receive) = mpsc::channel(2);
            if let Some(message) = message {
                assert!(send.try_send(message).is_ok());
            }
            drop(send);
            let mut body = StreamBody {
                prefix: None,
                start: None,
                receive,
                finished: false,
                supervisor: Supervisor {
                    cancel: None,
                    thread: None,
                },
            };
            let first = futures::executor::block_on(body.next()).unwrap();
            match first {
                Ok(bytes) => assert_eq!(bytes, b"finished"),
                Err(error) => assert!(!error.to_string().contains("SSR supervisor")),
            }
            assert!(futures::executor::block_on(body.next()).is_none());
        }
    }
}
