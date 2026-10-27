//! Real generated store consumer. Installation, policy and transaction completion are explicit.

/// Source emitted by the independent tool before compiling this application.
#[allow(
    missing_docs,
    reason = "Idea aggregate emits module declarations only; generated leaves have Rustdoc"
)]
#[cfg(not(any(feature = "matrix", feature = "names")))]
pub mod generated;
/// Separately generated complete storage matrix, selected only by its explicit test feature.
#[cfg(feature = "matrix")]
#[path = "matrix_generated/mod.rs"]
#[allow(missing_docs, reason = "generated aggregate module declarations")]
pub mod generated;

/// Small reserved-name fixture, isolated from the full matrix's output byte budget.
#[cfg(all(feature = "names", not(feature = "matrix")))]
#[path = "names_generated/mod.rs"]
#[allow(missing_docs, reason = "generated aggregate module declarations")]
pub mod generated;

#[cfg(all(feature = "names", feature = "matrix"))]
compile_error!("select one generated fixture feature at a time");

#[cfg(not(any(feature = "matrix", feature = "names")))]
use stackpress_framework_sql::DataSession;
#[cfg(not(any(feature = "matrix", feature = "names")))]
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
#[cfg(not(any(feature = "matrix", feature = "names")))]
use stackpress_server::ServerInvocationExt;
#[cfg(not(any(feature = "matrix", feature = "names")))]
use stackpress_sql::Engine;

/// App-owned authorization function; selection/registration is explicit.
#[cfg(not(any(feature = "matrix", feature = "names")))]
pub struct ProfileReadPolicy(pub fn(&Request) -> Result<()>);

/// A named Send handler borrows the registered Engine and projects only the approved name.
///
/// This example deliberately exposes one selected operation; no routes or maintenance events
/// are installed by importing a model. Authorization belongs to the app-owned typed policy.
#[cfg(not(any(feature = "matrix", feature = "names")))]
pub async fn profile_name(
    req: &mut Request,
    res: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let policy = invocation.service::<ProfileReadPolicy>()?;
    (policy.0)(req)?;
    let id = req
        .data
        .path_as::<String>("id")?
        .ok_or_else(|| Error::invalid("profile id is required"))?;
    let engine = invocation.service::<Engine>()?;
    let mut session = DataSession::pooled(engine);
    let profile = generated::stores::profile::Profiles::get(&mut session, &id)
        .await
        .map_err(|cause| {
            Error::new("profile lookup failed")
                .with_kind(stackpress_lib::ErrorKind::External)
                .with_cause(cause)
        })?;
    let profile = profile.ok_or_else(|| {
        Error::new("profile was not found")
            .with_kind(stackpress_lib::ErrorKind::Domain)
            .with_code(404)
    })?;
    // An explicit projection avoids serializing an entire private model/relation graph.
    res.results(&serde_json::json!({ "name": profile.name }))?;
    Ok(Flow::Continue)
}
