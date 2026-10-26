//! Compile-time domain selection and explicit, bounded application configuration.
//!
//! Only native hosts depend on this module. SQLite reuses the existing example's
//! read-only database and profiles plugins; startup never calls its setup helper.

use stackpress_lib::{Error, Result};
use stackpress_server::{BuildError, ConfigLayers, ServerBuilder};
use std::{io::Read, path::Path};

/// Read one explicitly selected JSON config file, or use registered defaults.
///
/// Sections follow sql-profile's `sqlite.path` and `profiles.limit` convention.
/// Paths inside JSON resolve from the process working directory. No environment
/// lookup, database creation or installation occurs here.
///
/// # Errors
/// Returns file access, UTF-8, bounded-input or configuration decoding errors.
pub fn configuration(path: Option<&Path>) -> Result<ConfigLayers> {
    let mut layers = ConfigLayers::new();
    if let Some(path) = path {
        // Reject directories and special files before a blocking file read/open.
        // The operator owns this path; concurrent replacement is not a supported input.
        if !std::fs::metadata(path)
            .map_err(Error::from_source)?
            .is_file()
        {
            return Err(Error::invalid(
                "profile UI configuration must be a regular JSON file",
            ));
        }
        // Read at most one byte past the configuration parser's finite budget.
        let limit = stackpress_lib::ParseLimits::default().max_bytes;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(Error::from_source)?
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(Error::from_source)?;
        if bytes.len() > limit {
            return Err(Error::limit("profile UI configuration exceeds byte limit"));
        }
        // Keep contents out of diagnostics; startup validates selected sections.
        let text = std::str::from_utf8(&bytes).map_err(Error::from_source)?;
        layers.json("profile-ui", text)?;
    }
    Ok(layers)
}

/// Add the explicit setup remedy to database startup failures, retaining their cause.
///
/// Configuration and unrelated plugin errors preserve their original identity/message.
/// This helper has no side effects and never retries, installs or seeds a database.
pub fn startup_error(failure: BuildError) -> BuildError {
    #[cfg(feature = "sqlite")]
    {
        let mut failure = failure;
        // Only the database plugin owns this remedy; validation failures keep theirs.
        if failure.plugin.as_deref() == Some("database") {
            failure.error = Error::new(
                "SQLite profile UI requires an installed sqlite.path; run sql-profile-setup explicitly"
            ).with_cause(failure.error);
        }
        failure
    }
    #[cfg(not(feature = "sqlite"))]
    failure
}

/// Register exactly the compiled backend before optional customization and pages.
pub(crate) fn register(app: &mut ServerBuilder) -> Result<()> {
    #[cfg(not(feature = "sqlite"))]
    {
        stackpress_example_generated_profile::generated::register(app)?;
    }
    #[cfg(feature = "sqlite")]
    {
        use stackpress_example_sql_profile::{database, profiles};
        use stackpress_server::PluginSpec;

        // Startup eagerly verifies the existing read-only file before publishing it.
        app.plugin("database", database::register)?;
        app.plugin_with(
            PluginSpec::new("profiles").requires(&["database"]),
            profiles::register,
        )?;
    }
    Ok(())
}
