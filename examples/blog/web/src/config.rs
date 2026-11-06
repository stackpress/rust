//! Shared computed application configuration, below the caller's `-b` environment layer.

use serde_json::{Value, json};
use stackpress_lib::Result;
use stackpress_server::ConfigLayers;

/// App-owned admin presentation section; executable page selection stays in Rust.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Admin(pub Value);
/// App-owned branding data used by native-to-public view adapters.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Brand(pub Value);
impl Brand {
    /// Apply the source setViewProps allowlist and its truthy defaults at the public boundary.
    /// Native config remains unchanged; this projection is shared by blog/auth/admin views.
    pub fn public(&self) -> Value {
        [
            ("name", "Stackpress"),
            ("logo", "/logo.png"),
            ("icon", "/icon.png"),
            ("favicon", "/favicon.ico"),
        ]
        .into_iter()
        .map(|(key, fallback)| {
            let value = self
                .0
                .get(key)
                .filter(|value| truthy(value))
                .cloned()
                .unwrap_or_else(|| fallback.into());
            (key.to_owned(), value)
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
    }
}
/// Original cookie defaults, owned by the application's HTTP adapter.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Cookie(pub Value);
/// Original terminal reference values; the Rust CLI owns argument parsing.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Terminal(pub Value);

/// Raw view/CSRF sections used only by the pure pre-registration configuration resolver.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
struct View(Value);

/// Resolved source presentation passed to existing registration-time admin adapters.
pub(crate) struct Presentation {
    pub admin: Value,
    pub brand: Value,
    pub view: Value,
    pub csrf: stackpress_framework_csrf::Service,
}

/// Read the compiled source reference without accessing another checkout at runtime.
///
/// # Errors
/// Propagates malformed checked-in JSON rather than silently replacing it with defaults.
pub fn reference() -> Result<Value> {
    Ok(serde_json::from_str(include_str!("common.json"))?)
}

/// Supply file SQLite and ordered population events without opening a database.
///
/// Runtime plugin selection is code. The caller appends its environment file and overrides.
///
/// # Errors
/// Preserves source data decoding and configuration validation failures.
pub fn shared(database_seed: &str) -> Result<ConfigLayers> {
    let mut database = reference()?["database"].clone();
    database["seed"] = database_seed.into();
    database["populate"] = crate::seed::events()?;
    let mut layers = ConfigLayers::new();
    layers.value(
        "blog shared",
        json!({
            "sqlite": {"path": ".build/blog.sqlite"},
            "client": {"revisions":".build/revisions"},
            "database": database,
        }),
    )?;
    Ok(layers)
}

/// Supply the complete compiled source configuration for the selected web constituents.
/// # Errors
/// Retains JSON/layer validation failures, before any file is opened.
pub fn application(database_seed: &str) -> Result<ConfigLayers> {
    let mut value = reference()?;
    value["sqlite"] = json!({"path":".build/blog.sqlite"});
    value["database"]["seed"] = database_seed.into();
    value["database"]["populate"] = crate::seed::events()?;
    // The original develop/production configs append these public build-resource grants.
    // Vite-only paths are absent from this Rust build pipeline; the public paths stay literal.
    for permits in value["session"]["access"]
        .as_object_mut()
        .into_iter()
        .flat_map(|access| access.values_mut())
    {
        if let Some(permits) = permits.as_array_mut() {
            permits.extend(
                [
                    "/assets/**",
                    "/client/**",
                    "/images/**",
                    "/styles/**",
                    "/favicon.ico",
                    "/favicon.png",
                ]
                .into_iter()
                .map(|route| json!({"method":"GET","route":route})),
            );
        }
    }
    let mut layers = ConfigLayers::new();
    layers.value("blog shared application", value)?;
    Ok(layers)
}

/// Resolve presentation with the existing typed layer decoder and no database/network startup.
/// The real host subsequently validates the complete same ordered inputs. This avoids a second
/// handwritten merge algorithm for the registration-time source-admin Config contract.
pub(crate) async fn presentation(
    layers: &ConfigLayers,
) -> stackpress_server::BuildResult<Presentation> {
    let mut app = stackpress_server::ServerBuilder::new();
    app.config_section("admin", Admin(json!({})), |_| Ok(()))?;
    app.config_section("brand", Brand(json!({})), |_| Ok(()))?;
    app.config_section("view", View(json!({})), |_| Ok(()))?;
    stackpress_framework_csrf::plugin::register(&mut app)?;
    for (label, value) in layers.entries() {
        let selected = ["admin", "brand", "view", "csrf"]
            .into_iter()
            .filter_map(|key| value.get(key).map(|value| (key.to_owned(), value.clone())))
            .collect::<serde_json::Map<_, _>>();
        let mut projection = ConfigLayers::new();
        projection.value(label, Value::Object(selected))?;
        app.config_layers(projection)?;
    }
    let resolved = app.build().await?;
    Ok(Presentation {
        admin: resolved.host().config::<Admin>()?.0.clone(),
        brand: resolved.host().config::<Brand>()?.public(),
        view: resolved.host().config::<View>()?.0.clone(),
        csrf: resolved
            .host()
            .service::<stackpress_framework_csrf::Service>()?
            .clone(),
    })
}

/// Claim app-owned reference sections; plugin-owned roots are claimed by their registrars.
pub(crate) fn register(app: &mut stackpress_server::ServerBuilder) -> Result<()> {
    app.config_section("admin", Admin(json!({})), |_| Ok(()))?;
    app.config_section("brand", Brand(json!({})), |_| Ok(()))?;
    app.config_section("cookie", Cookie(json!({})), |_| Ok(()))?;
    app.config_section("terminal", Terminal(json!({})), |_| Ok(()))
}

/// Preserve JavaScript truthiness at the app's source value boundaries.
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64() != Some(0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}
