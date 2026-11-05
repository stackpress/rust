//! Invocation-local binding to the application's compiled model family, never a global slot.
use super::bridge as b;
use crate::storage::AccountActions;
use stackpress_framework_sql::source::SessionModels;
use stackpress_lib::{Invocation, Result};
use stackpress_server::ServerInvocationExt;
use stackpress_sql::Engine;
/// AuthActions.make reads config only at its source access point; absence keeps abc123 fallback.
pub(crate) fn actions<M: SessionModels>(inv: &Invocation<'_>) -> Result<AccountActions<M>> {
    let environment = b::environment(inv);
    let password = b::optional_config(inv)?
        .as_ref()
        .map(|config| config.password.clone())
        .unwrap_or_default();
    Ok(AccountActions::new(
        inv.shared_service::<Engine>()?,
        environment.database_seed.unwrap_or_else(|| "abc123".into()),
        environment.offset_minutes,
        password,
    ))
}
