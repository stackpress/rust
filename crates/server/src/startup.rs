//! Exclusive startup state, typed resource access and consumed phase-task execution.
//!
//! The builder owns these values until Config/Listen/Route all succeed. No task or mutable
//! registry enters the published Server; cancellation drops the attempted construction.

use crate::builder::ServerBuilder;
use crate::router::CapturePolicy;
use crate::services::Registry;
use stackpress_lib::queue::{QueueOutcome, TaskHandler, TaskQueue, task};
use stackpress_lib::{EmitterBuilder, Error, ListenerId, Result, SourceLocation};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

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

/// Explicit phase ordering and diagnostics for one startup task.
#[must_use = "the configured value must be used"]
pub struct StartupSpec {
    /// Config, Listen or Route; Registration is not a task phase.
    pub(crate) stage: StartupStage,
    /// Higher priorities run first; ties preserve actual registration order.
    pub(crate) priority: i64,
    /// Bounded callback identity in BuildError.
    pub(crate) label: String,
}

impl StartupSpec {
    /// Describe a callback in one phase at priority zero.
    pub fn new(stage: StartupStage) -> Self {
        // Invalid Registration specifications fail at insertion rather than execute as tasks.
        Self {
            stage,
            priority: 0,
            label: format!("{stage:?}"),
        }
    }
    /// Set signed phase priority.
    pub fn priority(mut self, priority: i64) -> Self {
        // TaskQueue remains the ordering authority.
        self.priority = priority;
        self
    }
    /// Set a callback diagnostic label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        // Validation happens before the callback enters a queue.
        self.label = label.into();
        self
    }
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
pub(crate) struct StartupMetadata {
    /// Registration/phase callback contributor.
    pub(crate) plugin: Option<String>,
    /// Human-readable callback identity.
    pub(crate) label: String,
    /// Caller site captured synchronously before type erasure.
    pub(crate) registered: SourceLocation,
}

/// One sealed task contribution before conversion into the phase's TaskQueue.
pub(crate) struct StartupTask {
    /// Callback provenance is moved unchanged during builder composition.
    pub(crate) metadata: StartupMetadata,
    /// Higher priority first; stable queue insertion resolves equal priorities.
    pub(crate) priority: i64,
    /// Stored Send callback using the existing sealed lib contract.
    pub(crate) action: Arc<dyn TaskHandler<StartupContext>>,
}

/// Exclusive construction state; no reference to this value survives a successful build.
///
/// Config may insert/replace services after typed sections were validated. Listen registers
/// operations, and Route registers explicit HTTP exposure; both may read frozen services/config.
/// Direct plugin registration may contribute operations and routes before these async phases.
/// Resource cleanup is ordinary RAII, with no hidden asynchronous cleanup or replay.
pub struct StartupContext {
    /// Runtime contributions remain mutable only until the final table is built.
    pub(crate) events: EmitterBuilder,
    /// Service slots freeze after Config; no runtime registry lock is introduced.
    pub(crate) services: Registry,
    /// Capture policies follow host-route IDs through replacement/removal.
    pub(crate) routes: HashMap<ListenerId, CapturePolicy>,
    /// Typed sections are decoded before Config; test fixtures may also seed this privately.
    pub(crate) config: Registry,
    /// The builder advances phases sequentially, with exclusive access to this field.
    pub(crate) stage: StartupStage,
    /// Metadata belongs to the one currently executing startup callback.
    pub(crate) current: StartupMetadata,
}

impl StartupContext {
    /// Inspect the phase to which mutation permissions currently apply.
    pub fn stage(&self) -> StartupStage {
        // Phase cannot be mutated through an immutable handler borrow.
        self.stage
    }

    /// Read a typed decoded config section without cloning its payload.
    ///
    /// # Errors
    /// Reports a missing concrete section type; decoding/validation precedes all startup tasks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use stackpress_lib::{Flow, Result};
    /// use stackpress_server::{ServerBuilder, StartupContext};
    /// async fn configure(start: &mut StartupContext) -> Result<Flow> {
    ///     let limit = *start.config::<u64>()?;
    ///     start.register(Arc::new(format!("limit={limit}")))?;
    ///     assert_eq!(start.service::<String>()?, "limit=4");
    ///     Ok(Flow::Continue)
    /// }
    /// fn validate(_: &u64) -> Result<()> {
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.config_section("limit", 4_u64, validate)?;
    /// app.on_config(configure)?;
    /// let server = app.build().await?;
    /// assert_eq!(server.host().service::<String>()?, "limit=4");
    /// # Ok(())
    /// # }
    /// ```
    pub fn config<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Config is never sourced from request data or a process-global registry.
        self.config.get()
    }

    /// Insert an explicitly shared singleton during Config only.
    ///
    /// # Errors
    /// Rejects duplicate types and writes outside Config without replacing earlier services.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use stackpress_lib::{Flow, Result};
    /// use stackpress_server::{ServerBuilder, StartupContext};
    /// async fn configure(start: &mut StartupContext) -> Result<Flow> {
    ///     let limit = *start.config::<u64>()?;
    ///     start.register(Arc::new(format!("limit={limit}")))?;
    ///     assert_eq!(start.service::<String>()?, "limit=4");
    ///     Ok(Flow::Continue)
    /// }
    /// fn validate(_: &u64) -> Result<()> {
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.config_section("limit", 4_u64, validate)?;
    /// app.on_config(configure)?;
    /// let server = app.build().await?;
    /// assert_eq!(server.host().service::<String>()?, "limit=4");
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn register<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        // Runtime/read-only phases cannot secretly reopen service registration.
        // Keep register's existing error contract while denying mutation outside exclusive Config.
        if self.stage != StartupStage::Config {
            // Do not touch earlier services when the phase is already frozen.
            return Err(Error::new("services may only be registered during Config"));
        }
        // Insertion shares only the caller-supplied Arc and rejects an existing singleton.
        self.services.insert(value)
    }

    /// Register a deliberately named instance during Config only.
    ///
    /// # Errors
    /// Rejects calls outside Config, invalid/duplicate names, or registry budgets.
    /// Previously registered services remain unchanged.
    #[track_caller]
    pub fn register_named<T: Any + Send + Sync>(
        &mut self,
        name: &str,
        value: Arc<T>,
    ) -> Result<()> {
        self.require_config()?;
        // A name conflict cannot replace an earlier service of any concrete type.
        self.services.insert_named(name, value)
    }

    /// Customize an existing singleton explicitly during Config only.
    ///
    /// # Errors
    /// Rejects calls outside Config or a missing concrete singleton. The prior service
    /// and its provenance remain unchanged on rejection.
    #[track_caller]
    pub fn replace_service<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        self.require_config()?;
        // Missing slots remain errors; replacement retains original and new sites.
        self.services.replace(value)
    }

    /// Borrow a named service with a checked concrete type.
    ///
    /// # Errors
    /// Returns an error when the name is absent or its concrete type differs.
    pub fn service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T> {
        // No registry lock is held during subsequent application awaits.
        self.services.named(name)
    }

    /// Check the service mutation boundary before any slot changes.
    fn require_config(&self) -> Result<()> {
        // Only the resource-initialization phase may insert/replace named or singleton services.
        if self.stage == StartupStage::Config {
            // The checked Config phase owns exclusive construction state for this mutation.
            Ok(())
        } else {
            // Listen and Route can read services but cannot reopen their registry.
            Err(Error::invalid("services may only be changed during Config"))
        }
    }

    /// Read a service constructed by an earlier Config callback.
    ///
    /// # Errors
    /// Reports the missing concrete service type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use stackpress_lib::{Flow, Result};
    /// use stackpress_server::{ServerBuilder, StartupContext};
    /// async fn configure(start: &mut StartupContext) -> Result<Flow> {
    ///     let limit = *start.config::<u64>()?;
    ///     start.register(Arc::new(format!("limit={limit}")))?;
    ///     assert_eq!(start.service::<String>()?, "limit=4");
    ///     Ok(Flow::Continue)
    /// }
    /// fn validate(_: &u64) -> Result<()> {
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.config_section("limit", 4_u64, validate)?;
    /// app.on_config(configure)?;
    /// let server = app.build().await?;
    /// assert_eq!(server.host().service::<String>()?, "limit=4");
    /// # Ok(())
    /// # }
    /// ```
    pub fn service<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Borrowing a service does not retain a registry lock over await.
        self.services.get()
    }

    /// Own the original error together with the currently executing startup identity.
    pub(crate) fn failure(&self, error: Error) -> BuildError {
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

impl ServerBuilder {
    /// Schedule one named Send async Config callback.
    ///
    /// # Errors
    /// Returns checked queue-registration errors.
    #[track_caller]
    pub fn on_config(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()> {
        // Phase priority defaults to zero, with stable TaskQueue insertion ties.
        self.startup(StartupSpec::new(StartupStage::Config), step)
    }

    /// Schedule operation registration after Config succeeds.
    ///
    /// # Errors
    /// Returns checked queue-registration errors.
    #[track_caller]
    pub fn on_listen(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()> {
        // Startup uses builder tasks rather than ordinary runtime events.
        self.startup(StartupSpec::new(StartupStage::Listen), step)
    }

    /// Schedule explicit route exposure after Listen succeeds.
    ///
    /// # Errors
    /// Returns checked queue-registration errors.
    #[track_caller]
    pub fn on_route(&mut self, step: impl TaskHandler<StartupContext>) -> Result<()> {
        // There is no native endpoint/listener until a separately authorized adapter owns one.
        self.startup(StartupSpec::new(StartupStage::Route), step)
    }

    /// Wrap startup identity around an existing TaskHandler without changing lib's erasure.
    ///
    /// # Errors
    /// Rejects invalid diagnostic labels or the non-queued Registration stage. Existing
    /// queued callbacks remain unchanged. Callback failures occur later in [`Self::build`].
    #[track_caller]
    pub fn startup(
        &mut self,
        spec: StartupSpec,
        step: impl TaskHandler<StartupContext>,
    ) -> Result<()> {
        // Callbacks may schedule steps during selected-plugin registration only.
        crate::plugin::identity(&spec.label)?;
        // Map only the three queued phases; compiled registration is not itself a queued task.
        let index = match spec.stage {
            StartupStage::Config => 0,
            StartupStage::Listen => 1,
            StartupStage::Route => 2,
            StartupStage::Registration => return Err(Error::invalid("not a queued startup phase")),
        };
        let metadata = StartupMetadata {
            plugin: self.state.current.plugin.clone(),
            label: spec.label,
            registered: SourceLocation::caller(),
        };
        // Collection is construction-only; conversion to a queue occurs after it is sealed.
        self.phases[index].push(StartupTask {
            metadata,
            priority: spec.priority,
            action: Arc::new(step),
        });
        Ok(())
    }
}

/// Drain Config/Listen/Route through TaskQueue, preserving callback metadata and failure order.
pub(crate) async fn run_phases(
    state: &mut StartupContext,
    phases: [Vec<StartupTask>; 3],
) -> BuildResult<()> {
    // Every phase owns a consumed queue; empty is a valid successful phase.
    let stages = [
        StartupStage::Config,
        StartupStage::Listen,
        StartupStage::Route,
    ];
    // Earlier phases must finish successfully before the next phase becomes active.
    for (stage, contributions) in stages.into_iter().zip(phases) {
        // Each phase gets one consumed queue; no callback list or retry owner enters runtime state.
        let mut queue = TaskQueue::new();
        // Insert in actual registration order so equal priorities retain the lib tie rule.
        for contribution in contributions {
            let metadata = contribution.metadata;
            let action = contribution.action;
            // Existing TaskQueue owns stable priority sequencing and one-attempt consumption.
            queue
                .add(
                    task(move |state: &mut StartupContext| {
                        let metadata = metadata.clone();
                        let action = Arc::clone(&action);
                        Box::pin(async move {
                            // Set contributor identity before awaiting user startup work.
                            state.current = metadata;
                            action.run(state).await
                        })
                    }),
                    contribution.priority,
                )
                .map_err(|error| state.failure(error))?;
        }
        // Advance mutation permissions only after constructing the next phase queue.
        state.stage = stage;
        // Await this whole phase without a framework lock or detached/background startup task.
        let outcome = queue
            .run(state)
            .await
            .map_err(|error| state.failure(error))?;
        // Cooperative startup Abort is a failed build; it must never publish a partial Server.
        if outcome == QueueOutcome::Aborted {
            return Err(state.failure(Error::new("startup callback aborted")));
        }
    }
    Ok(())
}
