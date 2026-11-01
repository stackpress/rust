//! Source configuration paths, without choosing executable providers from data.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use stackpress_lib::{Error, Result};
use stackpress_server::ServerBuilder;
use std::path::PathBuf;

/// Explicit process inputs captured by the application, never read at registration.
#[derive(Clone, Default)]
pub struct Environment {
    /// Source process argv, including executable and entrypoint at indices zero and one.
    pub argv: Vec<String>,
    /// Source process working directory; relative paths retain native filesystem meaning.
    pub cwd: PathBuf,
    /// Source loader working directory, used for the default schema.idea path.
    pub loader_cwd: PathBuf,
    /// Source terminal force flag, including bare --force / -f handling by the host.
    pub force: bool,
}

macro_rules! section {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Value);
        impl Default for $name {
            /// Use the source's empty configuration fallback.
            fn default() -> Self {
                Self(json!({}))
            }
        }
    };
}
section!(
    ServerOptions,
    "Source server paths/options; native resources stay in application services."
);
section!(
    ClientOptions,
    "Source client configuration, separate from a compiled catalog."
);
section!(
    DatabaseOptions,
    "Source database command paths/populate events, without an implicit Engine."
);
section!(
    CliOptions,
    "Source CLI configuration, including the schema input path."
);

/// Claim aggregate-owned source roots once; direct composition may call this independently.
/// # Errors
/// Preserves the server's duplicate-section and configuration-budget errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // Disabled leaves do not claim unrelated input roots or revive another registrar.
    #[cfg(any(
        feature = "server-commands",
        feature = "schema-client",
        feature = "view"
    ))]
    app.config_section("server", ServerOptions::default(), |_| Ok(()))?;
    #[cfg(any(feature = "schema-client", feature = "sql"))]
    app.config_section("client", ClientOptions::default(), |_| Ok(()))?;
    #[cfg(feature = "schema-client")]
    app.config_section("cli", CliOptions::default(), |_| Ok(()))?;
    #[cfg(feature = "sql")]
    app.config_section("database", DatabaseOptions::default(), |_| Ok(()))?;
    let _ = app;
    Ok(())
}

/// Source JavaScript truthiness for JSON configuration and translation values.
#[cfg(any(
    feature = "schema-client",
    feature = "language",
    feature = "sql",
    feature = "view"
))]
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64() != Some(0.0),
        Value::String(value) => !value.is_empty(),
        // Empty objects and arrays are truthy in the source runtime.
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// Resolve a source dotted path, retaining explicit null separately from absence.
pub(crate) fn path<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    key.split('.').try_fold(value, |value, key| value.get(key))
}

/// Typed Rust string boundary, evaluated when the corresponding source operation uses it.
pub(crate) fn string(value: Option<&Value>, fallback: &str) -> Result<String> {
    value.map_or_else(
        || Ok(fallback.into()),
        |value| {
            value.as_str().map(str::to_owned).ok_or_else(|| {
                Error::invalid("source option requires a string at this native boundary")
            })
        },
    )
}
