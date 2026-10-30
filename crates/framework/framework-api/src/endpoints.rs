//! The source endpoint loop, shared by selected-plugin and direct registration.

use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

use crate::{ApiEndpoint, cors, routes};

/// Register configured CORS and public/app/session callbacks without selecting a plugin.
///
/// Declaration order is retained, including duplicate routes/events. The caller registers
/// target/detail events separately. No OAuth/page, webhook or database work occurs here.
///
/// # Examples
///
/// ```
/// use stackpress_framework_api::{ApiEndpoint, endpoints};
/// use stackpress_server::ServerBuilder;
/// let endpoint: ApiEndpoint = serde_json::from_value(serde_json::json!({
///     "method": "GET", "route": "/api/articles", "type": "public",
///     "event": "article-search", "data": {"status": "PUBLISHED"}
/// }))?;
/// let mut app = ServerBuilder::new();
/// endpoints::routes(&mut app, &[endpoint])?;
/// // Direct registration requires no API config section or implicit target-event registration.
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// Returns inherited method/path/data/registration errors. On failure discard the builder;
/// earlier direct contributions are not rolled back.
pub fn routes(app: &mut ServerBuilder, endpoints: &[ApiEndpoint]) -> Result<()> {
    for route in prepare(endpoints)? {
        // Use the same prepared actions as the plugin's Route phase.
        app.route_with(route.spec, route.action)?;
    }
    Ok(())
}

/// Prepare one ordered source inventory, preserving CORS-before-endpoint registration ties.
pub(crate) fn prepare(endpoints: &[ApiEndpoint]) -> Result<Vec<routes::PreparedRoute>> {
    let mut prepared = Vec::new();
    for endpoint in endpoints {
        // CORS has its own fixed priority; it neither authorizes nor replaces the endpoint.
        prepared.extend(cors::prepare(endpoint)?);
        prepared.push(routes::prepare(endpoint.clone(), endpoint.api_type)?);
    }
    Ok(prepared)
}
