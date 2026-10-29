//! Native synchronous Leptos staging with owned eager Head and explicit output limits.
use crate::{owner::RestoreOwner, safety::escape_html};
use futures::{FutureExt, StreamExt, stream};
use leptos::prelude::*;
use leptos_meta::{ServerMetaContext, ServerMetaContextOutput};
use std::{
    convert::Infallible,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// A finite output policy, independent of any application page protocol.
#[derive(Clone, Copy, Debug)]
pub struct RenderLimits {
    /// Complete framed HTML ceiling; zero denies output.
    pub html_bytes: usize,
    /// Logical resource JSON ceiling before script escaping.
    pub replay_bytes: usize,
}
impl Default for RenderLimits {
    /// Retain the existing two-MiB HTML and one-MiB replay defaults.
    fn default() -> Self {
        Self {
            html_bytes: 2 * 1024 * 1024,
            replay_bytes: 1024 * 1024,
        }
    }
}
impl RenderLimits {
    /// Reject expanded limits before retaining application work.
    pub fn validate<E>(self) -> Result<(), RenderError<E>> {
        if self.html_bytes > 2 * 1024 * 1024 || self.replay_bytes > 1024 * 1024 {
            return Err(RenderError::InvalidPolicy);
        }
        Ok(())
    }
    /// Admit only a completely framed document under the selected ceiling.
    pub(crate) fn finish<E>(self, html: String) -> Result<String, RenderError<E>> {
        if html.len() > self.html_bytes {
            Err(RenderError::HtmlLimit)
        } else {
            Ok(html)
        }
    }
}
/// Generic execution failure, retaining a caller's typed component/assembly error.
#[derive(Debug)]
pub enum RenderError<E = Infallible> {
    /// Explicit startup and an active native executor are required.
    ExecutorUnavailable,
    /// The supplied finite rendering policy is invalid.
    InvalidPolicy,
    /// Framed output exceeds the selected ceiling.
    HtmlLimit,
    /// Resource replay failed generic admission.
    Replay(crate::replay::ReplayError),
    /// A caller's component or trusted framing closure rejected the render.
    Component(E),
    /// Native-only diagnostics; Display never includes the cause text.
    Failure(Arc<dyn std::error::Error + Send + Sync>),
}
impl<E> RenderError<E> {
    /// Retain a native diagnostic without making it part of public HTML.
    pub fn from_source(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Failure(Arc::new(error))
    }
}
impl<E: std::fmt::Display> std::fmt::Display for RenderError<E> {
    /// Keep native cause text out of public output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ExecutorUnavailable => {
                "Native SSR requires initialize_ssr() inside an active Tokio runtime"
            }
            Self::InvalidPolicy => "Invalid render configuration",
            Self::HtmlLimit => "Rendered output is too large",
            Self::Replay(_) => "Invalid resource replay",
            Self::Component(_) => "Component rendering failed",
            Self::Failure(_) => "Could not render content",
        })
    }
}
impl<E: std::error::Error + 'static> std::error::Error for RenderError<E> {
    /// Expose causes only through explicit diagnostic traversal.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Component(e) => Some(e),
            Self::Replay(e) => Some(e),
            Self::Failure(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}
static INITIALIZED: AtomicBool = AtomicBool::new(false);
/// Initialize the native Leptos executor explicitly and idempotently.
/// # Errors
/// Rejects calls outside an active Tokio runtime.
pub fn initialize_ssr() -> Result<(), RenderError> {
    tokio::runtime::Handle::try_current().map_err(|_| RenderError::ExecutorUnavailable)?;
    match any_spawner::Executor::init_tokio() {
        Ok(()) | Err(any_spawner::ExecutorError::AlreadySet) => {
            INITIALIZED.store(true, Ordering::Release);
            Ok(())
        }
    }
}
/// Reject an unavailable executor before invoking application code.
pub fn require_executor<E>() -> Result<(), RenderError<E>> {
    if !INITIALIZED.load(Ordering::Acquire) || tokio::runtime::Handle::try_current().is_err() {
        return Err(RenderError::ExecutorUnavailable);
    }
    Ok(())
}
/// Synchronous rendering entry, with no implicit document or application policy.
pub struct Renderer;
/// Owned HTML and eager metadata; the reactive Owner has already been disposed.
pub struct PreparedRender<E = Infallible> {
    html: String,
    head: ServerMetaContextOutput,
    limits: RenderLimits,
    error: PhantomData<fn() -> E>,
}
impl Renderer {
    /// Evaluate a component synchronously, then assemble the caller's owned shell.
    /// # Errors
    /// Rejects missing executor, invalid limits or a caller error before publication.
    /// # Panics
    /// Trusted component unwind propagates after Owner cleanup and restoration.
    pub fn try_prepare<V: IntoView, E>(
        limits: RenderLimits,
        component: impl FnOnce() -> Result<V, E>,
        shell: impl FnOnce(String) -> Result<String, E>,
    ) -> Result<PreparedRender<E>, RenderError<E>> {
        require_executor()?;
        limits.validate()?;
        let owner = RenderOwner::new();
        let (meta, head) = ServerMetaContext::new();
        let html = owner.owner.with(|| {
            provide_context(meta);
            leptos_meta::provide_meta_context();
            let body = component().map_err(RenderError::Component)?.to_html();
            // Pinned leptos_meta inserts title text raw; reserve its final formatter.
            let _ =
                view! {<leptos_meta::Title formatter=|value:String|escape_html(&value)/>}.to_html();
            shell(body).map_err(RenderError::Component)
        })?;
        drop(owner);
        Ok(PreparedRender {
            html,
            head,
            limits,
            error: PhantomData,
        })
    }
}
impl<E> PreparedRender<E> {
    /// Finish eager Head injection using only owned buffers.
    /// # Errors
    /// Rejects executor loss, getter unwind or the final framed-output ceiling.
    pub async fn finish(self) -> Result<String, RenderError<E>> {
        require_executor()?;
        let assembly = async move {
            self.head
                .inject_meta_context(stream::iter([self.html]))
                .await
                .collect::<Vec<_>>()
                .await
                .concat()
        };
        let html = std::panic::AssertUnwindSafe(assembly)
            .catch_unwind()
            .await
            .map_err(|p| {
                RenderError::from_source(RenderPanic::new(
                    "SSR Head requires eager owned values (reactive SSR Head is S5-F6)",
                    p.as_ref(),
                ))
            })?;
        self.limits.finish(html)
    }
}
/// Native cause retained separately from public generic errors.
#[derive(Debug)]
pub(crate) struct RenderPanic(String);
impl RenderPanic {
    /// Retain only textual native diagnostics; never retry an unwound future.
    pub(crate) fn new(context: &str, payload: &(dyn std::any::Any + Send)) -> Self {
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("non-text panic payload");
        Self(format!("{context}: {message}"))
    }
}
impl std::fmt::Display for RenderPanic {
    /// Cause traversal is explicitly native-only.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for RenderPanic {}
/// Cleanup and caller restoration belong to the same synchronous staging scope.
struct RenderOwner {
    owner: Owner,
    previous: Option<Owner>,
}
impl RenderOwner {
    /// Retain the caller before constructing the request-local root.
    fn new() -> Self {
        Self {
            previous: Owner::current(),
            owner: Owner::new(),
        }
    }
}
impl Drop for RenderOwner {
    /// Restore even if an application cleanup callback unwinds.
    fn drop(&mut self) {
        let _restore = RestoreOwner(self.previous.take());
        self.owner.cleanup();
    }
}

impl<E> From<E> for RenderError<E> {
    /// Preserve a caller's original error at a generic execution boundary.
    fn from(error: E) -> Self {
        Self::Component(error)
    }
}
