//! Profiles owns its configuration, immutable in-memory service, operation and route.
//!
//! The plain [registration function](crate::generated::server::profile::register) queues
//! named Config, Listen and Route steps. Direct calls and route delegation share
//! [profile_search](crate::generated::server::profile::profile_search), with no adapter-specific
//! search policy or lifecycle. Seed rows contain public demo data only.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};

/// The largest accepted row limit, keeping this demonstration's output bounded.
const MAX_SEARCH_LIMIT: usize = 100;

/// Package-owned configuration decoded and validated before Config startup runs.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilesConfig {
    /// Maximum returned rows, in `1..=100`; the default is three.
    pub limit: usize,
}

impl Default for ProfilesConfig {
    /// Supply the lowest configuration layer, before explicit JSON and code overrides.
    fn default() -> Self {
        // All three seed rows are visible unless a later layer narrows the result limit.
        Self { limit: 3 }
    }
}

// Public data lives in the generated contracts partition.
use crate::generated::contracts::profile::Profile;

/// Immutable demo data and validated result limit, shared explicitly through an Arc.
pub struct Profiles {
    /// Rows remain in insertion order and are never mutated after Config startup.
    rows: Vec<Profile>,
    /// Copied from validated config once; requests cannot override host policy.
    limit: usize,
}

impl Profiles {
    /// Create the private seed service after the host has validated its configuration.
    fn new(limit: usize) -> Self {
        // Fix the seed order so prefix searches and limit changes have predictable results.
        let rows = [
            ("abc123", "Ada Lovelace"),
            ("abc456", "Grace Hopper"),
            ("abc789", "Margaret Hamilton"),
        ]
        .into_iter()
        .map(|(id, name)| Profile {
            id: id.into(),
            name: name.into(),
        })
        .collect();
        Self { rows, limit }
    }

    /// Return at most the configured number of rows whose IDs start with `id`.
    ///
    /// Search is case-sensitive and preserves seed order. Returned rows borrow immutable
    /// service data. This async service boundary performs no I/O, locks, or spawned work.
    ///
    /// # Errors
    /// Rejects an empty ID instead of treating missing input as an unrestricted list operation.
    pub async fn search(&self, id: &str) -> Result<Vec<&Profile>> {
        // An absent/empty identifier must not silently broaden the requested search.
        if id.is_empty() {
            return Err(Error::invalid("profile search requires a nonempty id"));
        }

        // Filter before limiting: unrelated rows never consume the caller's result budget.
        Ok(self
            .rows
            .iter()
            .filter(|profile| profile.id.starts_with(id))
            .take(self.limit)
            .collect())
    }
}

/// Register typed defaults and the three named startup steps, with no plugin trait.
///
/// # Errors
/// Returns duplicate-section or startup-registration errors from the builder.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // Register defaults before Config; the host layers and validates before creating services.
    app.config_section("profiles", ProfilesConfig::default(), validate_profiles)?;
    app.on_config(configure)?;
    app.on_listen(listen_profiles)?;
    app.on_route(route_profiles)?;
    Ok(())
}

/// Reject zero or excessive limits after all layers have been applied.
fn validate_profiles(config: &ProfilesConfig) -> Result<()> {
    // A positive finite bound keeps both ordinary and customized searches predictable.
    if !(1..=MAX_SEARCH_LIMIT).contains(&config.limit) {
        return Err(Error::invalid("profiles.limit must be in 1..=100"));
    }
    Ok(())
}

/// Construct one explicitly shared service from the final validated configuration.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    // Copy policy before inserting the service; the Config phase owns this only mutation.
    let limit = start.config::<ProfilesConfig>()?.limit;
    start.register(Arc::new(Profiles::new(limit)))?;
    Ok(Flow::Continue)
}

/// Register the named capability after the service registry has frozen.
async fn listen_profiles(start: &mut StartupContext) -> Result<Flow> {
    // This operation is independent of HTTP route selection and CLI argument parsing.
    start.on("profile-search", profile_search)?;
    Ok(Flow::Continue)
}

/// Expose only this read operation through an explicit GET route.
async fn route_profiles(start: &mut StartupContext) -> Result<Flow> {
    // The host injects the matched id before invoking profile_route.
    start.get("/profiles/:id", profile_route)?;
    Ok(Flow::Continue)
}

/// Read `id` from request data and return bounded rows plus the returned-row total.
///
/// # Errors
/// Returns invalid/missing input, unavailable service, or response-serialization errors.
pub async fn profile_search(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Typed lookup rejects non-string input; the service rejects missing or empty strings.
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    let rows = inv.service::<Profiles>()?.search(&id).await?;

    // Serialize public row fields while the service borrow is valid; total is returned rows.
    res.rows(&rows, rows.len() as u64)?;
    Ok(Flow::Continue)
}

/// Delegate to the shared operation without starting another top-level lifecycle.
async fn profile_route(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Preserve child Abort and errors so the top-level host remains the lifecycle authority.
    inv.delegate("profile-search", req, res).await
}
