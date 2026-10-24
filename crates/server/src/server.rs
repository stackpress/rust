//! Immutable runtime table, host inspection and explicitly raw dispatch entry points.
//!
//! Top-level lifecycle lives in lifecycle; construction/startup mutation cannot be reopened
//! through this API. Sharing the host requires an explicit application-owned Arc.

use crate::context::{CallContext, HostContext};
use crate::plugin::PluginDescriptor;
use crate::report::LifecycleStage;
use stackpress_lib::{
    DispatchOutcome, EventEmitter, HandlerFuture, IntoRequest, Request, Resolution, Response,
    Result,
};
use std::sync::{Arc, atomic::AtomicU64};

/// Immutable capability host; explicitly wrap in Arc to share it across tasks.
pub struct Server {
    /// Selected plugin order and provenance, with no executable callbacks.
    plugins: Vec<PluginDescriptor>,
    /// Checked local allocation, shared only when the application explicitly shares Server.
    pub(crate) next_correlation: AtomicU64,
    /// Optional application prefix; local-only trust semantics remain unchanged.
    pub(crate) prefix: Option<String>,
    /// Frozen table has no registry or per-callback mutex.
    pub(crate) emitter: EventEmitter,
    /// Read-only registry has no reference back to the emitter or Server.
    pub(crate) host: Arc<HostContext>,
}

impl Server {
    /// Publish already-completed construction without retaining builder tasks or callbacks.
    pub(crate) fn from_parts(
        plugins: Vec<PluginDescriptor>,
        prefix: Option<String>,
        emitter: EventEmitter,
        host: HostContext,
    ) -> Self {
        // Only immutable runtime ownership escapes; each new host has an independent sequence.
        Self {
            plugins,
            prefix,
            emitter,
            host: Arc::new(host),
            next_correlation: AtomicU64::new(1),
        }
    }

    /// Inspect selected plugins in the original application order.
    pub fn plugins(&self) -> impl Iterator<Item = &PluginDescriptor> {
        // Descriptors carry no mutation or plugin-loading authority.
        self.plugins.iter()
    }

    /// Inspect immutable listener provenance and matching descriptors.
    pub fn listeners(&self) -> impl Iterator<Item = &stackpress_lib::ListenerDescriptor> {
        // Foundation descriptors retain IDs and original/replacement sites.
        self.emitter.listeners()
    }

    /// Inspect immutable host services/config without exposing registration mutation.
    pub fn host(&self) -> &HostContext {
        // Borrowing the Arc target does not implicitly clone a service or host owner.
        &self.host
    }

    /// Dispatch raw capability work with host context and no top-level lifecycle.
    ///
    /// # Errors
    /// Returns original annotated matching/handler errors. No automatic input load,
    /// status default, error event, native output or top-level correlation is added.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// let server = app.build().await?;
    /// let mut req = Request::new();
    /// let mut res = Response::new();
    /// let outcome = server.emit("greet", &mut req, &mut res).await?;
    /// assert_eq!(outcome, stackpress_lib::DispatchOutcome::Completed);
    /// assert!(!req.loaded()); // Raw dispatch does not run Input or lifecycle hooks.
    /// # Ok(())
    /// # }
    /// ```
    pub fn emit<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
    ) -> HandlerFuture<'a, Result<DispatchOutcome>> {
        Box::pin(async move {
            // Validate event metadata before entering raw callbacks, without allocating
            // correlation.
            crate::lifecycle::validate_event(event)?;
            // This local context remains owned by the returned future until every child returns.
            let context = self.raw_context();
            self.emitter
                .emit_with_context(event, req, res, &context)
                .await
        })
    }

    /// Resolve raw work using fresh owned values and inherited host access.
    ///
    /// # Errors
    /// Returns input-conversion/dispatch failures without lifecycle or native output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// let server = app.build().await?;
    /// let resolved = server.resolve("greet", serde_json::json!({})).await?;
    /// assert_eq!(resolved.outcome, stackpress_lib::DispatchOutcome::Completed);
    /// assert_eq!(resolved.response.results.unwrap().as_str(), Some("hello"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve<'a, I: IntoRequest + 'a>(
        &'a self,
        event: &'a str,
        input: I,
    ) -> HandlerFuture<'a, Result<Resolution>> {
        Box::pin(async move {
            crate::lifecycle::validate_event(event)?;
            // The explicit Arc host handle is immutable and contains no runtime back-reference.
            let context = self.raw_context();
            self.emitter
                .resolve_with_context(event, input, &context)
                .await
        })
    }

    /// Construct metadata for one raw root, deliberately without a top-level correlation.
    fn raw_context(&self) -> CallContext {
        // A separate root owns a separate context; there is no mutable global current stage.
        CallContext {
            host: Arc::clone(&self.host),
            correlation: None,
            stage: LifecycleStage::Dispatch,
            failure: None,
            adapter_context: None,
        }
    }
}
