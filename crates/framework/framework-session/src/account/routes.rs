//! Source account route order and optional -100 presentation contributions.
use super::*;
use crate::auth::AuthSection;
use stackpress_lib::Flow;
use stackpress_server::StartupContext;

/// Identity registrar calls this only with session enabled; missing auth adds no routes.
pub(crate) async fn routes(start: &mut StartupContext) -> Result<Flow> {
    let Ok(section) = start.config::<AuthSection>() else {
        return Ok(Flow::Continue);
    };
    let Some(config) = &section.0 else {
        return Ok(Flow::Continue);
    };
    let base = config.base.as_deref().unwrap_or("/auth").to_owned();
    start.get(&format!("{base}/account"), account)?;
    start.get(&format!("{base}/account/update"), update)?;
    start.get(&format!("{base}/account/security"), account)?;
    start.get(&format!("{base}/account/security/export"), export)?;
    start.get(&format!("{base}/account/security/remove"), remove)?;
    start.get(&format!("{base}/account/security/password"), password)?;
    start.get(&format!("{base}/account/security/2fa"), enroll)?;
    start.get(
        &format!("{base}/account/security/2fa/remove"),
        remove_factor,
    )?;
    start.post(&format!("{base}/account/update"), update)?;
    start.post(&format!("{base}/account/security/remove"), remove)?;
    start.post(&format!("{base}/account/security/password"), password)?;
    start.post(&format!("{base}/account/security/2fa"), enroll)?;
    start.post(
        &format!("{base}/account/security/2fa/remove"),
        remove_factor,
    )?;
    #[cfg(feature = "leptos-ssr")]
    crate::presentation::account_routes(start, &base)?;
    Ok(Flow::Continue)
}
