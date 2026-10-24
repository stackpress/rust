//! One construction-time registrar with ergonomic ServerBuilder forwarding.
//!
//! StartupContext owns all phase, provenance, stable-handle and route-wrapper policy.
//! Builder methods forward synchronously with track_caller instead of duplicating it.

use crate::builder::ServerBuilder;
use crate::router::{RouteSpec, wrap};
use crate::startup::{StartupContext, StartupStage};
use stackpress_lib::{Error, Handler, ListenerId, ListenerSpec, MethodPattern, Result};

impl StartupContext {
    /// Register an exact runtime listener, retaining plugin and actual caller provenance.
    ///
    /// # Errors
    /// Returns the foundation's matcher/identity/metadata errors.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.on("greet", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_listen(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn on(&mut self, event: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Route-phase callbacks cannot reopen operation registration.
        self.listen(ListenerSpec::exact(event), action)
    }

    /// Register an operation at an explicit signed priority.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.on_priority("greet", 10, greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_listen(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.listen(stackpress_lib::ListenerSpec::exact("greet").label("greeting"), greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_listen(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
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
        // Direct selection and Listen may add operations; Config/Route cannot reopen that phase.
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
        // Only direct registration and Route may change explicit HTTP exposure.
        if matches!(self.stage, StartupStage::Registration | StartupStage::Route) {
            Ok(())
        } else {
            Err(Error::invalid(
                "route registration requires Registration or Route",
            ))
        }
    }

    /// Register a host route with incoming named capture precedence.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.route(stackpress_lib::MethodPattern::GET, "/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     let spec = stackpress_server::RouteSpec::new(
    ///         stackpress_lib::MethodPattern::GET,
    ///         "/hello",
    ///     );
    ///     start.route_with(spec, greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.get("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn get(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::GET, path, action)
    }

    /// Register a POST host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.post("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn post(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::POST, path, action)
    }

    /// Register a PUT host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.put("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn put(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::PUT, path, action)
    }

    /// Register a PATCH host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.patch("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn patch(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::PATCH, path, action)
    }

    /// Register a DELETE host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.delete("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn delete(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::DELETE, path, action)
    }

    /// Register a HEAD host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.head("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn head(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::HEAD, path, action)
    }

    /// Register a OPTIONS host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.options("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn options(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::OPTIONS, path, action)
    }

    /// Register a CONNECT host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.connect("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn connect(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::CONNECT, path, action)
    }

    /// Register a TRACE host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.trace("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[track_caller]
    pub fn trace(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Convenience preserves the same matcher and all-matching priority contract.
        self.route(MethodPattern::TRACE, path, action)
    }

    /// Register a ALL host route with the default capture policy.
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
    /// use stackpress_server::StartupContext;
    /// async fn register(start: &mut StartupContext) -> Result<Flow> {
    ///     start.all("/hello", greet)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_route(register)?;
    /// app.build().await?;
    /// # Ok(())
    /// # }
    /// ```
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
}

impl ServerBuilder {
    /// Register a named exact capability with selected-plugin provenance.
    ///
    /// # Errors
    /// Returns checked foundation registration errors.
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
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn on(&mut self, event: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // Preserve the user's site through each synchronous forwarding layer.
        self.state.on(event, action)
    }

    /// Forward on_priority during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.on_priority("greet", 10, greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
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
    /// let mut app = ServerBuilder::new();
    /// app.listen(stackpress_lib::ListenerSpec::exact("greet").label("greeting"), greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
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
    /// let mut app = ServerBuilder::new();
    /// app.route(stackpress_lib::MethodPattern::GET, "/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
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
    /// let mut app = ServerBuilder::new();
    /// let spec = stackpress_server::RouteSpec::new(
    ///     stackpress_lib::MethodPattern::GET,
    ///     "/hello",
    /// );
    /// app.route_with(spec, greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
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
    /// let mut app = ServerBuilder::new();
    /// app.get("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn get(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.get(path, action)
    }

    /// Forward post during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.post("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn post(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.post(path, action)
    }

    /// Forward put during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.put("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn put(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.put(path, action)
    }

    /// Forward patch during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.patch("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn patch(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.patch(path, action)
    }

    /// Forward delete during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.delete("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn delete(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.delete(path, action)
    }

    /// Forward head during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.head("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn head(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.head(path, action)
    }

    /// Forward options during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.options("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn options(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.options(path, action)
    }

    /// Forward connect during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.connect("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn connect(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.connect(path, action)
    }

    /// Forward trace during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.trace("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn trace(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.trace(path, action)
    }

    /// Forward all during direct/plugin registration with the actual caller site.
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
    /// let mut app = ServerBuilder::new();
    /// app.all("/hello", greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn all(&mut self, path: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // StartupContext enforces phase and stable handle/capture policy.
        self.state.all(path, action)
    }
}
