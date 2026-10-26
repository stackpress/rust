//! Consuming startup over real TaskQueues, with no partially published runtime table.

use crate::config::{ConfigLayers, Sections};
use crate::context::HostContext;
use crate::plugin::{Plugin, PluginDescriptor, PluginSpec, preflight};
use crate::services::Registry;
use crate::startup::{StartupMetadata, StartupTask, run_phases};
use stackpress_lib::{EmitterBuilder, Error, Result, SourceLocation};
use std::collections::HashMap;

// Keep the existing internal re-export paths used by adapters and the crate facade.
pub use crate::server::Server;
pub use crate::startup::{BuildError, BuildResult, StartupContext, StartupSpec, StartupStage};

/// Owned non-Clone startup accumulator, consumed before immutable runtime publication.
///
/// Direct contributions precede selected-plugin callbacks. Building preflights the complete
/// selected list, decodes every typed section, then consumes Config/Listen/Route in order.
/// Dropping a build stops future work; completed external effects are not rolled back.
pub struct ServerBuilder {
    /// State is consumed into the immutable runtime only after every task succeeds.
    pub(crate) state: StartupContext,
    /// Separate ordered queues prevent a phase from recursively invoking runtime lifecycle.
    pub(crate) phases: [Vec<StartupTask>; 3],
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_server::ServerBuilder;
    /// let app = ServerBuilder::new();
    /// # let _ = app;
    /// ```
    pub fn new() -> Self {
        // Default constructs no listener, native handle or shared runtime table.
        Self::default()
    }

    /// Queue a compiled registration function in explicit application order.
    ///
    /// # Errors
    /// Rejects invalid/duplicate IDs or selection after build has sealed the plugin list.
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
    /// fn greetings(app: &mut ServerBuilder) -> Result<()> {
    ///     app.on("greet", greet)?;
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.plugin("greetings", greetings)?;
    /// let server = app.build().await?;
    /// assert!(server.call("greet", serde_json::json!({})).await?.report.failures.is_empty());
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::Result;
    /// use stackpress_server::{PluginSpec, ServerBuilder};
    /// fn storage(_: &mut ServerBuilder) -> Result<()> {
    ///     Ok(())
    /// }
    /// fn profiles(_: &mut ServerBuilder) -> Result<()> {
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.plugin("storage", storage)?;
    /// app.plugin_with(PluginSpec::new("profiles").requires(&["storage"]), profiles)?;
    /// let server = app.build().await?;
    /// assert_eq!(server.plugins().count(), 2);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Rejects invalid specification metadata, duplicate IDs, sealed selection, or the
    /// selection-count limit without changing previous selections. This queues registration;
    /// missing dependencies, cycles/order violations and callback failures surface from
    /// [`Self::build`] before a Server is published.
    #[track_caller]
    pub fn plugin_with(
        &mut self,
        spec: PluginSpec,
        register: fn(&mut ServerBuilder) -> Result<()>,
    ) -> Result<&mut Self> {
        // Check IDs and dependency declarations before mutating the selected set.
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
        // Retain the real selection caller; callback execution waits for complete graph preflight.
        self.plugins.push(Plugin::new(spec, register));
        Ok(self)
    }

    /// Append explicit config layers after earlier file/code inputs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::Result;
    /// use stackpress_server::{ConfigLayers, ServerBuilder};
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct Settings {
    ///     limit: u64,
    /// }
    /// fn validate(value: &Settings) -> Result<()> {
    ///     if value.limit == 0 {
    ///         return Err(stackpress_lib::Error::invalid("limit must be positive"));
    ///     }
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.config_section("settings", Settings { limit: 2 }, validate)?;
    /// let mut layers = ConfigLayers::new();
    /// layers.json("application", r#"{"settings":{"limit":5}}"#)?;
    /// app.config_layers(layers)?;
    /// let server = app.build().await?;
    /// assert_eq!(server.host().config::<Settings>()?.limit, 5);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Rejects aggregate layer byte/count/depth/node limits. Earlier accepted layers remain
    /// unchanged; the supplied owned layers are consumed on either result.
    pub fn config_layers(&mut self, layers: ConfigLayers) -> Result<&mut Self> {
        // Layers remain data-only and cannot enable a plugin.
        self.layers.append(layers)?;
        Ok(self)
    }

    /// Claim one typed package-owned section with lowest-precedence defaults.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::Result;
    /// use stackpress_server::{ConfigLayers, ServerBuilder};
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct Settings {
    ///     limit: u64,
    /// }
    /// fn validate(value: &Settings) -> Result<()> {
    ///     if value.limit == 0 {
    ///         return Err(stackpress_lib::Error::invalid("limit must be positive"));
    ///     }
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.config_section("settings", Settings { limit: 2 }, validate)?;
    /// let mut layers = ConfigLayers::new();
    /// layers.json("application", r#"{"settings":{"limit":5}}"#)?;
    /// app.config_layers(layers)?;
    /// let server = app.build().await?;
    /// assert_eq!(server.host().config::<Settings>()?.limit, 5);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Rejects invalid/duplicate section keys or concrete types, serialization failure, or
    /// default-value budgets before retaining the section. Typed decoding and validation
    /// run later during [`Self::build`]; a failed build publishes no Server.
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
    ///
    /// # Errors
    /// Rejects sealed/excessive or duplicate plugin selections, conflicting section keys/types,
    /// layer budgets, or listener sequence exhaustion. The receiver remains unchanged;
    /// the supplied builder is consumed on success or failure.
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
        // Preflight singleton claims before any imported contribution can mutate the receiver.
        self.sections.compatible(&other.sections)?;
        // Stage the entire layer append so a late aggregate-budget error leaves this owner
        // untouched.
        let layers = self.layers.combined(other.layers)?;
        // Foundation checks its complete sequence range before moving listener ownership.
        self.state.events.extend(other.state.events)?;
        // Move wrapper policies with stable listener IDs; future replacement still uses host
        // injection.
        self.state.routes.extend(other.state.routes);
        self.plugins.extend(other.plugins);
        self.sections.extend(other.sections);
        self.layers = layers;
        // Preserve each phase's callback insertion order and its original metadata.
        for (target, source) in self.phases.iter_mut().zip(other.phases) {
            target.extend(source);
        }
        Ok(self)
    }

    /// Configure a printable host-local correlation prefix; does not create global uniqueness.
    ///
    /// # Errors
    /// Rejects empty, oversized or control-containing diagnostic identities without
    /// changing the previous prefix.
    pub fn correlation_prefix(&mut self, prefix: &str) -> Result<&mut Self> {
        crate::plugin::identity(prefix)?;
        // Prefix is explicit application configuration, never an incoming header.
        self.prefix = Some(prefix.into());
        Ok(self)
    }

    /// Preflight and seal selection, then run compiled registration in supplied order.
    fn register_plugins(&mut self) -> BuildResult<Vec<PluginDescriptor>> {
        // Seal plugin selection before invoking any user registration function.
        // Validate the entire graph before any registration callback can produce effects.
        preflight(&self.plugins)?;
        // Callbacks may add contributions but cannot expand the preflighted plugin selection.
        self.sealed = true;
        let plugins = std::mem::take(&mut self.plugins);
        let descriptors = plugins.iter().map(|p| p.descriptor.clone()).collect();
        // Run compiled registration in supplied order; priority-sensitive composition is never
        // sorted.
        for plugin in plugins {
            self.state.current = StartupMetadata {
                plugin: Some(plugin.descriptor.id),
                label: "registration".into(),
                registered: plugin.descriptor.registered,
            };
            (plugin.register)(self).map_err(|error| self.state.failure(error))?;
        }
        Ok(descriptors)
    }

    /// Consume registration and all three phases before publishing an immutable Server.
    ///
    /// # Errors
    /// Returns plugin dependency/order failures, configuration decoding/validation/budget
    /// failures, the original callback error, or a named startup-abort error. Failure,
    /// cancellation and panic never return a partially registered runtime or retry owner.
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
    /// fn greetings(app: &mut ServerBuilder) -> Result<()> {
    ///     app.on("greet", greet)?;
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.plugin("greetings", greetings)?;
    /// let server = app.build().await?;
    /// assert!(server.call("greet", serde_json::json!({})).await?.report.failures.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(mut self) -> BuildResult<Server> {
        // Collect all selected contributions before decoding sections or starting resources.
        let descriptors = self.register_plugins()?;
        // Decode every section before any Config task can observe or allocate resources.
        self.sections.decode(self.layers, &mut self.state.config)?;
        // Consume asynchronous startup before immutable publication can expose the table.
        run_phases(&mut self.state, self.phases).await?;
        // No callbacks/queues go into HostContext, so it cannot retain its own emitter.
        Ok(Server::from_parts(
            descriptors,
            self.prefix,
            self.state.events.build(),
            HostContext {
                config: self.state.config,
                services: self.state.services,
            },
        ))
    }
}
