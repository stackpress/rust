//! Consuming startup over real TaskQueues, with no partially published runtime table.

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use stackpress_lib::queue::{QueueOutcome, TaskHandler, TaskQueue, task};
use stackpress_lib::{
    DispatchOutcome, EmitterBuilder, Error, EventEmitter, Handler, HandlerFuture, IntoRequest,
    ListenerId, ListenerSpec, MethodPattern, Request, Resolution, Response, Result, SourceLocation,
};

use crate::config::{ConfigLayers, Sections};
use crate::context::{CallContext, HostContext};
use crate::plugin::{Plugin, PluginDescriptor, PluginSpec, preflight};
use crate::report::LifecycleStage;
use crate::router::{CapturePolicy, RouteSpec, wrap};
use crate::services::Registry;
use std::sync::atomic::AtomicU64;

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
pub struct StartupSpec {
    /// Config, Listen or Route; Registration is not a task phase.
    stage: StartupStage,
    /// Higher priorities run first; ties preserve actual registration order.
    priority: i64,
    /// Bounded callback identity in BuildError.
    label: String,
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
struct StartupMetadata {
    /// Registration/phase callback contributor.
    plugin: Option<String>,
    /// Human-readable callback identity.
    label: String,
    /// Caller site captured synchronously before type erasure.
    registered: SourceLocation,
}

/// One sealed task contribution before conversion into the phase's TaskQueue.
struct StartupTask {
    /// Callback provenance is moved unchanged during builder composition.
    metadata: StartupMetadata,
    /// Higher priority first; stable queue insertion resolves equal priorities.
    priority: i64,
    /// Stored Send callback using the existing sealed lib contract.
    action: Arc<dyn TaskHandler<StartupContext>>,
}

/// Exclusive construction state; no reference to this value survives a successful build.
pub struct StartupContext {
    /// Runtime contributions remain mutable only until the final table is built.
    events: EmitterBuilder,
    /// Service slots freeze after Config; no runtime registry lock is introduced.
    services: Registry,
    /// Capture policies follow host-route IDs through replacement/removal.
    routes: HashMap<ListenerId, CapturePolicy>,
    /// Typed sections are decoded before Config; test fixtures may also seed this privately.
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

    /// Read a typed decoded config section without cloning its payload.
    ///
    /// # Errors
    /// Reports a missing concrete section type; decoding/validation precedes all startup tasks.
    pub fn config<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Config is never sourced from request data or a process-global registry.
        self.config.get()
    }

    /// Insert an explicitly shared singleton during Config only.
    ///
    /// # Errors
    /// Rejects duplicate types and writes outside Config without replacing earlier services.
    #[track_caller]
    pub fn register<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        // Runtime/read-only phases cannot secretly reopen service registration.
        if self.stage != StartupStage::Config {
            return Err(Error::new("services may only be registered during Config"));
        }
        self.services.insert(value)
    }

    /// Register a deliberately named instance during Config only.
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
    #[track_caller]
    pub fn replace_service<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        self.require_config()?;
        // Missing slots remain errors; replacement retains original and new sites.
        self.services.replace(value)
    }

    /// Borrow a named service with a checked concrete type.
    pub fn service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T> {
        // No registry lock is held during subsequent application awaits.
        self.services.named(name)
    }

    /// Check the service mutation boundary before any slot changes.
    fn require_config(&self) -> Result<()> {
        if self.stage == StartupStage::Config {
            Ok(())
        } else {
            Err(Error::invalid("services may only be changed during Config"))
        }
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
        // Route-phase callbacks cannot reopen operation registration.
        self.listen(ListenerSpec::exact(event), action)
    }

    /// Register an operation at an explicit signed priority.
    #[track_caller]
    pub fn on_priority(
        &mut self,
        event: &str,
        priority: i64,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // Stable ties remain foundation registration order.
        self.listen(ListenerSpec::exact(event).priority(priority), action)
    }

    /// Register a foundation listener with current contributor provenance.
    #[track_caller]
    pub fn listen(
        &mut self,
        spec: ListenerSpec,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        self.require_listen()?;
        // Synchronous forwarding preserves the original external caller site.
        self.events.listen(self.spec(spec), action)
    }

    /// Replace an owned contribution while retaining any host capture wrapper.
    #[track_caller]
    pub fn replace(&mut self, id: &ListenerId, action: impl Handler + 'static) -> Result<()> {
        if let Some(policy) = self.routes.get(id).copied() {
            self.require_route()?;
            // Replacement never accidentally changes a host route into a raw action.
            self.events.replace(id, wrap(policy, action))
        } else {
            self.require_listen()?;
            self.events.replace(id, action)
        }
    }

    /// Remove exactly the contribution identified by its stable handle.
    pub fn remove(&mut self, id: &ListenerId) -> Result<bool> {
        if self.routes.contains_key(id) {
            self.require_route()?;
        } else {
            self.require_listen()?;
        }
        // Only remove the policy after foundation ownership validation succeeds.
        let removed = self.events.remove(id)?;
        if removed {
            self.routes.remove(id);
        }
        Ok(removed)
    }

    /// Require operation mutation during direct/plugin registration or Listen.
    fn require_listen(&self) -> Result<()> {
        if matches!(
            self.stage,
            StartupStage::Registration | StartupStage::Listen
        ) {
            Ok(())
        } else {
            Err(Error::invalid(
                "operation registration requires Registration or Listen",
            ))
        }
    }

    /// Require route mutation during direct/plugin registration or Route.
    fn require_route(&self) -> Result<()> {
        if matches!(self.stage, StartupStage::Registration | StartupStage::Route) {
            Ok(())
        } else {
            Err(Error::invalid(
                "route registration requires Registration or Route",
            ))
        }
    }

    /// Register a host route with incoming named capture precedence.
    #[track_caller]
    pub fn route(
        &mut self,
        method: MethodPattern,
        path: &str,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // The explicit descriptor carries exactly the default route capture policy.
        self.route_with(RouteSpec::new(method, path), action)
    }

    /// Register an explicit host-route descriptor, preserving all-matching semantics.
    #[track_caller]
    pub fn route_with(
        &mut self,
        spec: RouteSpec,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        self.require_route()?;
        let policy = spec.captures;
        let listener = self.spec(spec.listener()?);
        // Install the wrapped action before publishing its replacement policy.
        let id = self.events.listen(listener, wrap(policy, action))?;
        self.routes.insert(id, policy);
        Ok(id)
    }

    /// Register a GET host route with the default capture policy.
    #[track_caller]
    pub fn get(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::GET, path, action)
    }

    /// Register a POST host route with the default capture policy.
    #[track_caller]
    pub fn post(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::POST, path, action)
    }

    /// Register a PUT host route with the default capture policy.
    #[track_caller]
    pub fn put(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::PUT, path, action)
    }

    /// Register a PATCH host route with the default capture policy.
    #[track_caller]
    pub fn patch(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::PATCH, path, action)
    }

    /// Register a DELETE host route with the default capture policy.
    #[track_caller]
    pub fn delete(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::DELETE, path, action)
    }

    /// Register a HEAD host route with the default capture policy.
    #[track_caller]
    pub fn head(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::HEAD, path, action)
    }

    /// Register a OPTIONS host route with the default capture policy.
    #[track_caller]
    pub fn options(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::OPTIONS, path, action)
    }

    /// Register a CONNECT host route with the default capture policy.
    #[track_caller]
    pub fn connect(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::CONNECT, path, action)
    }

    /// Register a TRACE host route with the default capture policy.
    #[track_caller]
    pub fn trace(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::TRACE, path, action)
    }

    /// Register a ALL host route with the default capture policy.
    #[track_caller]
    pub fn all(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::Any, path, action)
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

/// Owned non-Clone startup accumulator, consumed before immutable runtime publication.
pub struct ServerBuilder {
    /// State is consumed into the immutable runtime only after every task succeeds.
    pub(crate) state: StartupContext,
    /// Separate ordered queues prevent a phase from recursively invoking runtime lifecycle.
    phases: [Vec<StartupTask>; 3],
    /// Selection order is retained and preflighted before any callback executes.
    plugins: Vec<Plugin>,
    /// Registration is closed before any selected plugin function is called.
    sealed: bool,
    /// Config is claimed during direct/plugin registration and frozen before startup.
    sections: Sections,
    /// Explicit external layers remain ordered after package defaults.
    layers: ConfigLayers,
    /// Optional bounded local identity prefix; no incoming request can replace it.
    prefix: Option<String>,
}

impl Default for ServerBuilder {
    /// Construct an empty host without starting any external resource.
    fn default() -> Self {
        // Only owned accumulators are created; there is no hidden executor/service singleton.
        Self {
            state: StartupContext {
                events: EmitterBuilder::new(),
                services: Registry::default(),
                routes: HashMap::new(),
                config: Registry::default(),
                stage: StartupStage::Registration,
                current: StartupMetadata {
                    plugin: None,
                    label: "application".into(),
                    registered: SourceLocation::caller(),
                },
            },
            phases: std::array::from_fn(|_| Vec::new()),
            plugins: Vec::new(),
            sealed: false,
            sections: Sections::default(),
            layers: ConfigLayers::new(),
            prefix: None,
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
        // Ordinary selection uses exactly the same preflight path as dependency declarations.
        self.plugin_with(PluginSpec::new(id), register)
    }

    /// Select a compiled plugin with checked presence/order declarations.
    #[track_caller]
    pub fn plugin_with(
        &mut self,
        spec: PluginSpec,
        register: fn(&mut ServerBuilder) -> Result<()>,
    ) -> Result<&mut Self> {
        spec.validate()?;
        // Registration cannot expand the already-preflighted selected graph.
        if self.sealed || self.plugins.len() >= 1024 {
            return Err(Error::invalid(
                "plugin selection is sealed or exceeds budget",
            ));
        }
        if self.plugins.iter().any(|p| p.descriptor.id == spec.id) {
            return Err(Error::invalid(format!("duplicate plugin ID: {}", spec.id)));
        }
        self.plugins.push(Plugin::new(spec, register));
        Ok(self)
    }

    /// Append explicit config layers after earlier file/code inputs.
    pub fn config_layers(&mut self, layers: ConfigLayers) -> Result<&mut Self> {
        // Layers remain data-only and cannot enable a plugin.
        self.layers.append(layers)?;
        Ok(self)
    }

    /// Claim one typed package-owned section with lowest-precedence defaults.
    #[track_caller]
    pub fn config_section<T>(
        &mut self,
        key: &str,
        defaults: T,
        validate: fn(&T) -> Result<()>,
    ) -> Result<()>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
    {
        // Current plugin identity is recorded at registration, before any decoding.
        self.sections
            .register(key, defaults, validate, self.state.current.plugin.clone())
    }

    /// Move construction contributions, preserving IDs, sites, owners and insertion order.
    ///
    /// Plugin callbacks still run after direct contributions; imported direct registrations
    /// retain their original owner metadata. Neither builder may have started construction.
    pub fn extend(&mut self, other: Self) -> Result<&mut Self> {
        // Check singleton and selection conflicts before moving any receiver contributions.
        if self.sealed || other.sealed || self.plugins.len() + other.plugins.len() > 1024 {
            return Err(Error::invalid(
                "cannot compose sealed or excessive plugin selections",
            ));
        }
        if other.plugins.iter().any(|new| {
            self.plugins
                .iter()
                .any(|old| old.descriptor.id == new.descriptor.id)
        }) {
            return Err(Error::invalid("duplicate composed plugin ID"));
        }
        self.sections.compatible(&other.sections)?;
        let layers = self.layers.combined(other.layers)?;
        // Foundation checks its complete sequence range before moving listener ownership.
        self.state.events.extend(other.state.events)?;
        self.state.routes.extend(other.state.routes);
        self.plugins.extend(other.plugins);
        self.sections.extend(other.sections);
        self.layers = layers;
        for (target, source) in self.phases.iter_mut().zip(other.phases) {
            target.extend(source);
        }
        Ok(self)
    }

    /// Configure a printable host-local correlation prefix; does not create global uniqueness.
    pub fn correlation_prefix(&mut self, prefix: &str) -> Result<&mut Self> {
        crate::plugin::identity(prefix)?;
        // Prefix is explicit application configuration, never an incoming header.
        self.prefix = Some(prefix.into());
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

    /// Forward on_priority during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn on_priority(
        &mut self,
        event: &str,
        priority: i64,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.on_priority(event, priority, action)
    }

    /// Forward listen during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn listen(
        &mut self,
        spec: ListenerSpec,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.listen(spec, action)
    }

    /// Forward route during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn route(
        &mut self,
        method: MethodPattern,
        path: &str,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.route(method, path, action)
    }

    /// Forward route_with during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn route_with(
        &mut self,
        spec: RouteSpec,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.route_with(spec, action)
    }

    /// Forward replace during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn replace(&mut self, id: &ListenerId, action: impl Handler + 'static) -> Result<()> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.replace(id, action)
    }

    /// Forward remove during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn remove(&mut self, id: &ListenerId) -> Result<bool> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.remove(id)
    }

    /// Forward get during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn get(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.get(path, action)
    }

    /// Forward post during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn post(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.post(path, action)
    }

    /// Forward put during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn put(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.put(path, action)
    }

    /// Forward patch during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn patch(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.patch(path, action)
    }

    /// Forward delete during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn delete(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.delete(path, action)
    }

    /// Forward head during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn head(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.head(path, action)
    }

    /// Forward options during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn options(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.options(path, action)
    }

    /// Forward connect during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn connect(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.connect(path, action)
    }

    /// Forward trace during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn trace(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.trace(path, action)
    }

    /// Forward all during direct/plugin registration with the actual caller site.
    #[track_caller]
    pub fn all(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.all(path, action)
    }

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
    #[track_caller]
    pub fn startup(
        &mut self,
        spec: StartupSpec,
        step: impl TaskHandler<StartupContext>,
    ) -> Result<()> {
        // Callbacks may schedule steps during selected-plugin registration only.
        crate::plugin::identity(&spec.label)?;
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

    /// Consume registration and all three phases before publishing an immutable Server.
    ///
    /// # Errors
    /// Returns the original callback failure or a named startup-abort error. Failure,
    /// cancellation and panic never return a partially registered runtime or retry owner.
    pub async fn build(mut self) -> BuildResult<Server> {
        // Seal plugin selection before invoking any user registration function.
        preflight(&self.plugins)?;
        self.sealed = true;
        let plugins = std::mem::take(&mut self.plugins);
        let descriptors = plugins.iter().map(|p| p.descriptor.clone()).collect();
        for plugin in plugins {
            self.state.current = StartupMetadata {
                plugin: Some(plugin.descriptor.id),
                label: "registration".into(),
                registered: plugin.descriptor.registered,
            };
            (plugin.register)(&mut self).map_err(|error| self.state.failure(error))?;
        }
        // Decode every section before any Config task can observe or allocate resources.
        self.sections.decode(self.layers, &mut self.state.config)?;
        // Every phase owns a consumed queue; empty is a valid successful phase.
        let stages = [
            StartupStage::Config,
            StartupStage::Listen,
            StartupStage::Route,
        ];
        for (stage, contributions) in stages.into_iter().zip(self.phases) {
            let mut queue = TaskQueue::new();
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
                                state.current = metadata;
                                action.run(state).await
                            })
                        }),
                        contribution.priority,
                    )
                    .map_err(|error| self.state.failure(error))?;
            }
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
            plugins: descriptors,
            next_correlation: AtomicU64::new(1),
            prefix: self.prefix,
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
    pub fn emit<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
    ) -> HandlerFuture<'a, Result<DispatchOutcome>> {
        Box::pin(async move {
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
