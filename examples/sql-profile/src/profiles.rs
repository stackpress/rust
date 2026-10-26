//! Profiles policy, owned public DTO and a single capability shared by every entry path.

use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::{DialectId, Engine, Expr, Order, Select, TrustedSql};

/// The same validated result policy as the reference in-memory profiles plugin.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilesConfig {
    /// Maximum returned rows in `1..=100`; requests cannot override it.
    pub limit: usize,
}

impl Default for ProfilesConfig {
    /// Show all three canonical seed rows unless the application narrows its policy.
    fn default() -> Self {
        Self { limit: 3 }
    }
}

/// Owned application response contract, independent from SQL rows, ordinals and native types.
#[derive(Debug, Serialize)]
pub struct Profile {
    /// Case-sensitive public identifier.
    pub id: String,
    /// Public display name.
    pub name: String,
}

/// Queue the profiles policy, named capability and explicit route; database is an app prerequisite.
///
/// # Errors
/// Returns duplicate configuration or startup registration failures.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("profiles", ProfilesConfig::default(), validate)?;
    app.on_listen(listen)?;
    app.on_route(route)?;
    Ok(())
}

/// Preserve the original limit validation message and bounds exactly.
fn validate(config: &ProfilesConfig) -> Result<()> {
    if !(1..=100).contains(&config.limit) {
        return Err(Error::invalid("profiles.limit must be in 1..=100"));
    }
    Ok(())
}

/// Install one business capability; no generic query/setup operation is exposed.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("profile-search", profile_search)?;
    Ok(Flow::Continue)
}

/// Register only the existing GET route; no native listener is started here.
async fn route(start: &mut StartupContext) -> Result<Flow> {
    start.get("/profiles/:id", profile_route)?;
    Ok(Flow::Continue)
}

/// Build exact UTF-8 byte-prefix matching, without SQL LIKE's default case folding or wildcards.
fn search(id: &str, limit: usize) -> Select {
    // The audited expression is constant. Both occurrences of user input remain owned binds.
    // BLOB length/substr also preserve embedded NUL bytes, matching Rust str::starts_with.
    let prefix = Expr::Trusted(
        DialectId::Sqlite,
        TrustedSql::audited(
            "substr(CAST(\"id\" AS BLOB), 1, length(CAST(? AS BLOB))) = CAST(? AS BLOB)",
        ),
        vec![id.into(), id.into()],
    );
    // Filter before limiting and sort by the explicit seed/insertion ordinal, never by ID.
    Select::new(["id", "name"])
        .from("profile")
        .and_where(prefix)
        .order_by("ordinal", Order::Asc)
        .limit(limit as u64)
}

/// Return the same bounded rows/returned-row total and validation errors as capability-host.
///
/// # Errors
/// Rejects missing/empty IDs, propagates typed input/decode/database failures and never substitutes
/// an empty success after a failed query. Request data cannot choose file paths, SQL or limits.
pub async fn profile_search(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    if id.is_empty() {
        return Err(Error::invalid("profile search requires a nonempty id"));
    }
    let limit = inv.config::<ProfilesConfig>()?.limit;
    let rows = inv
        .service::<Engine>()?
        .fetch(&search(&id, limit))
        .await
        .map_err(Error::from_source)?;
    // Decode owned DTOs explicitly. No ordinal, SqlValue or driver metadata enters the envelope.
    let profiles = rows
        .into_iter()
        .map(|row| -> stackpress_sql::Result<Profile> {
            Ok(Profile {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
            })
        })
        .collect::<stackpress_sql::Result<Vec<_>>>()
        .map_err(Error::from_source)?;
    res.rows(&profiles, profiles.len() as u64)?;
    Ok(Flow::Continue)
}

/// Delegate to the selected operation within the existing top-level lifecycle.
async fn profile_route(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("profile-search", req, res).await
}
