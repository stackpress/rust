//! Optional compiled presentation plugin; the generated search operation stays unchanged.

use stackpress_example_profile_ui_contracts::SearchReply;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

/// Register the visible label customization after the generated operation's priority zero.
///
/// # Errors
/// Returns ordinary listener registration errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.on_priority("profile-search", -10, label_profiles)?;
    Ok(())
}

/// Project generated public rows, change only their displayed names, and retain zero totals.
async fn label_profiles(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let mut reply =
        SearchReply::from_envelope(&res.try_to_json()?).map_err(stackpress_lib::Error::invalid)?;
    for row in &mut reply.rows {
        // The compiled plugin changes presentation at the operation boundary, not in each UI.
        row.name = format!("Demo: {}", row.name);
    }
    res.rows(&reply.rows, reply.total)?;
    Ok(Flow::Continue)
}
