//! An independently selected customization that leaves the profiles plugin unchanged.
//!
//! Select this plugin with `PluginSpec::new("customization").requires(&["profiles"])`.
//! A higher-priority named handler reuses the same service/input policy, marks display names,
//! then returns Abort so the original handler cannot overwrite the customized rows.

use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};

use crate::profiles::{Profile, Profiles};

/// Queue this customization after selecting the profiles prerequisite.
///
/// # Errors
/// Returns startup-task registration errors; plugin ordering is checked by the host.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // Defer capability registration to Listen, like the original plugin.
    app.on_listen(listen_customization)?;
    Ok(())
}

/// Add the overriding listener with an explicit priority above the ordinary operation.
async fn listen_customization(start: &mut StartupContext) -> Result<Flow> {
    // Priority, rather than plugin insertion order, establishes which operation runs first.
    start.on_priority("profile-search", 100, customized_search)?;
    Ok(Flow::Continue)
}

/// Reuse profile policy while deliberately replacing the visible names and stopping dispatch.
async fn customized_search(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Share the original service's validation, ordering and configured limit.
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    let rows = inv.service::<Profiles>()?.search(&id).await?;

    // Copy only public fields into customized output; the shared seed service stays immutable.
    let rows: Vec<Profile> = rows
        .into_iter()
        .map(|profile| Profile {
            id: profile.id.clone(),
            name: format!("Customized {}", profile.name),
        })
        .collect();
    res.rows(&rows, rows.len() as u64)?;

    // Abort stops the original operation and subsequent response hooks under G12.
    Ok(Flow::Abort)
}
