//! Consuming startup over real TaskQueues, with no partially published runtime table.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

use stackpress_lib::queue::{QueueOutcome, TaskHandler, TaskQueue, task};
use stackpress_lib::{
    DispatchOutcome, EmitterBuilder, Error, EventEmitter, EventPattern, Handler, HandlerFuture,
    IntoRequest, ListenerId, ListenerSpec, MethodPattern, Nest, Request, Resolution, Response,
    Result, SourceLocation, handler,
};

use crate::context::{CallContext, HostContext, LifecycleStage, Registry};

/// Startup phase is distinct from the handled-invocation lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupStage {
    /// Selected compiled plugin registration, before asynchronous phases.
    Registration,
    /// Service construction and environment-derived mechanisms.
    Config,
    /// Reusable operation registration after services freeze.
    Listen,
    /// Explicit HTTP exposure after capabilities exist.
    Route,
}

/// Owned startup error with the failing callback's actual plugin/stage/site.
#[derive(Debug)]
pub struct BuildError {
    /// Phase in which construction failed or aborted.
    pub stage: StartupStage,
    /// Explicit contributing plugin, if one was selected.
    pub plugin: Option<String>,
    /// Callback label, useful before a runtime listener exists.
    pub label: String,
    /// Actual callback/plugin registration site.
    pub registered: SourceLocation,
    /// Original cause chain; no request/session/config document is added automatically.
    pub error: Error,
}

/// Startup returns no Server on abort, callback failure or cancellation.
pub type BuildResult<T> = std::result::Result<T, BuildError>;

impl fmt::Display for BuildError {
    /// Render bounded framework identity before the explicit original error message.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display contains callback identity, not registry payloads or configuration contents.
        write!(
            formatter,
            "startup {:?}, plugin {:?}, {}: {}",
            self.stage, self.plugin, self.label, self.error
        )
    }
}

impl std::error::Error for BuildError {
    /// Preserve the original Error for normal source-chain traversal/downcasting.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // No wrapper re-creates or stringifies the original diagnostic cause.
        Some(&self.error)
    }
}

impl From<Error> for BuildError {
    /// Convert an application composition failure before any selected callback runs.
    #[track_caller]
    fn from(error: Error) -> Self {
        // Callback-specific failures use StartupContext::failure instead of this root identity.
        Self {
            stage: StartupStage::Registration,
            plugin: None,
            label: "application".into(),
            registered: SourceLocation::caller(),
            error,
        }
    }
}

/// Immutable identifying metadata, never a shared mutable current-invocation slot.
#[derive(Clone)]
struct StartupMetadata {
    /// Registration/phase callback contributor.
    plugin: Option<String>,
    /// Human-readable callback identity.
    label: String,
    /// Caller site captured synchronously before type erasure.
    registered: SourceLocation,
}

/// Exclusive construction state; no reference to this value survives a successful build.
pub struct StartupContext {
    /// Runtime contributions remain mutable only until the final table is built.
    events: EmitterBuilder,
    /// Service slots freeze after Config; no runtime registry lock is introduced.
    services: Registry,
    /// M0 uses private typed fixtures; JSON section decoding is the next milestone.
    pub(crate) config: Registry,
    /// The builder advances phases sequentially, with exclusive access to this field.
    stage: StartupStage,
    /// Metadata belongs to the one currently executing startup callback.
    current: StartupMetadata,
}

impl StartupContext {
    /// Inspect the phase to which mutation permissions currently apply.
    pub fn stage(&self) -> StartupStage {
        // Phase cannot be mutated through an immutable handler borrow.
        self.stage
    }

    /// Read a typed decoded config fixture/section without cloning its payload.
    ///
    /// # Errors
    /// Reports a missing section type. Full section registration/decoding arrives in M1.
    pub fn config<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Config is never sourced from request data or a process-global registry.
        self.config.get()
    }

    /// Insert an explicitly shared singleton during Config only.
    ///
    /// # Errors
    /// Rejects duplicate types and writes outside Config without replacing earlier services.
    pub fn register<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        // Runtime/read-only phases cannot secretly reopen service registration.
        if self.stage != StartupStage::Config {
            return Err(Error::new("services may only be registered during Config"));
        }
        self.services.insert(value)
    }

    /// Read a service constructed by an earlier Config callback.
    ///
    /// # Errors
    /// Reports the missing concrete service type.
    pub fn service<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Borrowing a service does not retain a registry lock over await.
        self.services.get()
    }

    /// Register an exact runtime listener, retaining plugin and actual caller provenance.
    ///
    /// # Errors
    /// Returns the foundation's matcher/identity/metadata errors.
    #[track_caller]
    pub fn on(&mut self, event: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Forward synchronously so the final listener site remains the user's call site.
        self.events
            .listen(self.spec(ListenerSpec::exact(event)), action)
    }

    /// Register a host route wrapper that injects only named captures as literal keys.
    ///
    /// # Errors
    /// Returns route compilation or checked registration errors.
    #[track_caller]
    pub fn route(
        &mut self,
        method: MethodPattern,
        path: &str,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // Compile before registration and retain the wrapper at the same listener identity.
        let spec = self.spec(ListenerSpec::pattern(EventPattern::route(method, path)?));
        let action = Arc::new(action);
        self.events.listen(
            spec,
            handler(move |req, res, inv| {
                let action = Arc::clone(&action);
                Box::pin(async move {
                    // Build a small incoming object before atomically overlaying aggregate data.
                    let mut captures = Nest::new();
                    for (name, value) in &inv.captures().params {
                        captures.set(name, value.clone())?;
                    }
                    req.data.merge(captures)?;
                    // Sealed supertraits compose outside lib without new forwarding methods.
                    action.invoke(req, res, inv).await
                })
            }),
        )
    }

    /// Register an explicit GET route using the same checked wrapper.
    ///
    /// # Errors
    /// Returns the same errors as [`Self::route`].
    #[track_caller]
    pub fn get(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Method convenience adds no specificity or hidden dispatch priority.
        self.route(MethodPattern::GET, path, action)
    }

    /// Add current plugin identity without changing the caller's listener specification.
    fn spec(&self, spec: ListenerSpec) -> ListenerSpec {
        // Application-owned direct contributions honestly have no selected-plugin identity.
        match &self.current.plugin {
            Some(plugin) => spec.plugin(plugin),
            None => spec,
        }
    }

    /// Own the original error together with the currently executing startup identity.
    fn failure(&self, error: Error) -> BuildError {
        // A consumed builder cannot escape in this error or be retried implicitly.
        BuildError {
            stage: self.stage,
            plugin: self.current.plugin.clone(),
            label: self.current.label.clone(),
            registered: self.current.registered.clone(),
            error,
        }
    }
}

/// Selected registration function and provenance, without a trait/macro plugin contract.
struct Plugin {
    /// Explicit unique ID; dependency policy is the next milestone.
    id: String,
    /// Compiled function, not a runtime module pathname.
    register: fn(&mut ServerBuilder) -> Result<()>,
    /// Site at which application code selected this plugin.
    registered: SourceLocation,
}

/// Owned startup accumulator. M0 proves phases/wrappers, not the full M1 policy surface.
pub struct ServerBuilder {
    /// State is consumed into the immutable runtime only after every task succeeds.
    pub(crate) state: StartupContext,
    /// Separate ordered queues prevent a phase from recursively invoking runtime lifecycle.
    phases: [TaskQueue<StartupContext>; 3],
    /// Selection order is retained; M1 adds declared dependency preflight.
    plugins: Vec<Plugin>,
    /// Registration is closed before any selected plugin function is called.
    sealed: bool,
}

impl Default for ServerBuilder {
    /// Construct an empty host without starting any external resource.
    fn default() -> Self {
        // Only owned accumulators are created; there is no hidden executor/service singleton.
        Self {
            state: StartupContext {
                events: EmitterBuilder::new(),
                services: Registry::default(),
                config: Registry::default(),
                stage: StartupStage::Registration,
                current: StartupMetadata {
                    plugin: None,
                    label: "application".into(),
                    registered: SourceLocation::caller(),
                },
            },
            phases: std::array::from_fn(|_| TaskQueue::new()),
            plugins: Vec::new(),
            sealed: false,
        }
    }
}

impl ServerBuilder {
    /// Start an empty compile-time-selected application host.
    pub fn new() -> Self {
        // Default constructs no listener, native handle or shared runtime table.
        Self::default()
    }

    /// Queue a compiled registration function in explicit application order.
    ///
    /// # Errors
    /// Rejects invalid/duplicate IDs or selection after build has sealed the plugin list.
    #[track_caller]
    pub fn plugin(
        &mut self,
        id: &str,
        register: fn(&mut ServerBuilder) -> Result<()>,
    ) -> Result<&mut Self> {
        // Validate before mutating selection; arbitrary strings never instantiate code.
        if self.sealed || id.is_empty() || id.len() > 256 || id.chars().any(char::is_control) {
            return Err(Error::invalid(
                "plugin selection is sealed or its ID is invalid",
            ));
        }
        if self.plugins.iter().any(|plugin| plugin.id == id) {
            return Err(Error::invalid(format!("duplicate plugin ID: {id}")));
        }
        self.plugins.push(Plugin {
            id: id.into(),
            register,
            registered: SourceLocation::caller(),
        });
        Ok(self)
    }

    /// Register a named exact capability with selected-plugin provenance.
    ///
    /// # Errors
    /// Returns checked foundation registration errors.
    #[track_caller]
    pub fn on(&mut self, event: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Preserve the user's site through each synchronous forwarding layer.
        self.state.on(event, action)
    }

    /// Schedule one named Send async Config callback.
    ///
    /// # Errors
    /// Returns checked queue-registration errors.
    #[track_caller]
    pub fn on_config(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()> {
        // Phase priority defaults to zero, with stable TaskQueue insertion ties.
        self.startup(StartupStage::Config, "config", step)
    }

    /// Schedule operation registration after Config succeeds.
    ///
    /// # Errors
    /// Returns checked queue-registration errors.
    #[track_caller]
    pub fn on_listen(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()> {
        // Startup uses builder tasks rather than ordinary runtime events.
        self.startup(StartupStage::Listen, "listen", step)
    }

    /// Schedule explicit route exposure after Listen succeeds.
    ///
    /// # Errors
    /// Returns checked queue-registration errors.
    #[track_caller]
    pub fn on_route(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()> {
        // There is no native endpoint/listener until a separately authorized adapter owns one.
        self.startup(StartupStage::Route, "route", step)
    }

    /// Wrap startup identity around an existing TaskHandler without changing lib's erasure.
    #[track_caller]
    fn startup(
        &mut self,
        stage: StartupStage,
        label: &str,
        step: impl TaskHandler<StartupContext>,
    ) -> Result<()> {
        // Callbacks may schedule steps during selected-plugin registration only.
        let index = match stage {
            StartupStage::Config => 0,
            StartupStage::Listen => 1,
            StartupStage::Route => 2,
            StartupStage::Registration => return Err(Error::invalid("not a queued startup phase")),
        };
        let metadata = StartupMetadata {
            plugin: self.state.current.plugin.clone(),
            label: label.into(),
            registered: SourceLocation::caller(),
        };
        let step = Arc::new(step);
        self.phases[index].add(
            task(move |state: &mut StartupContext| {
                let metadata = metadata.clone();
                let step = Arc::clone(&step);
                Box::pin(async move {
                    // Metadata is set under the one exclusive startup-state borrow before awaiting.
                    state.current = metadata;
                    step.run(state).await
                })
            }),
            0,
        )?;
        Ok(())
    }

    /// Consume registration and all three phases before publishing an immutable Server.
    ///
    /// # Errors
    /// Returns the original callback failure or a named startup-abort error. Failure,
    /// cancellation and panic never return a partially registered runtime or retry owner.
    pub async fn build(mut self) -> BuildResult<Server> {
        // Seal plugin selection before invoking any user registration function.
        self.sealed = true;
        let plugins = std::mem::take(&mut self.plugins);
        for plugin in plugins {
            self.state.current = StartupMetadata {
                plugin: Some(plugin.id),
                label: "registration".into(),
                registered: plugin.registered,
            };
            (plugin.register)(&mut self).map_err(|error| self.state.failure(error))?;
        }
        // Every phase owns a consumed queue; empty is a valid successful phase.
        let stages = [
            StartupStage::Config,
            StartupStage::Listen,
            StartupStage::Route,
        ];
        for (stage, mut queue) in stages.into_iter().zip(self.phases) {
            self.state.stage = stage;
            let outcome = queue
                .run(&mut self.state)
                .await
                .map_err(|error| self.state.failure(error))?;
            if outcome == QueueOutcome::Aborted {
                return Err(self.state.failure(Error::new("startup callback aborted")));
            }
        }
        // No callbacks/queues go into HostContext, so it cannot retain its own emitter.
        Ok(Server {
            emitter: self.state.events.build(),
            host: Arc::new(HostContext {
                config: self.state.config,
                services: self.state.services,
            }),
        })
    }
}

/// Immutable capability host; explicitly wrap in Arc to share it across tasks.
pub struct Server {
    /// Frozen table has no registry or per-callback mutex.
    pub(crate) emitter: EventEmitter,
    /// Read-only registry has no reference back to the emitter or Server.
    pub(crate) host: Arc<HostContext>,
}

impl Server {
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
    pub fn emit<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
    ) -> HandlerFuture<'a, Result<DispatchOutcome>> {
        Box::pin(async move {
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
    pub fn resolve<'a, I: IntoRequest + 'a>(
        &'a self,
        event: &'a str,
        input: I,
    ) -> HandlerFuture<'a, Result<Resolution>> {
        Box::pin(async move {
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
        }
    }
}
