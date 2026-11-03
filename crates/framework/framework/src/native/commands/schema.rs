//! Source generation input precedence and the explicit Rust build-host boundary.

use super::super::config::{
    CliOptions, ClientOptions, Environment, ServerOptions, path, string, truthy,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[cfg(feature = "codegen")]
use stackpress_lib::Error;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ConfigPresence, ServerInvocationExt};
use std::path::PathBuf;

/// Original generator options; TS-only settings remain visible to a selected host adapter.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Options {
    /// Request i overrides input, which overrides cli.idea, then loader cwd/schema.idea.
    pub idea: String,
    /// Source mandatory client.tsconfig; the Rust adapter receives it as source metadata.
    pub tsconfig: String,
    /// Explicit client.build or cwd/node_modules/client.package.
    pub build: PathBuf,
    /// Source lang default js; Rust generation is selected through a compiled provider.
    pub lang: String,
    /// Original formatter settings, for source-compatible external build adapters.
    pub prettier: Value,
}

/// Resolve source input/output paths without performing IO or generation.
/// # Errors
/// Returns a typed native-boundary error for source options requiring string values.
pub fn options(
    req: &Request,
    client: &ClientOptions,
    server: &ServerOptions,
    cli: &CliOptions,
    env: &Environment,
) -> Result<Options> {
    let fallback = env
        .loader_cwd
        .join("schema.idea")
        .to_string_lossy()
        .into_owned();
    let configured = string(path(&cli.0, "idea"), &fallback)?;
    let input = req.data.get("input").map(|v| v.to_json()).transpose()?;
    let short = req.data.get("i").map(|v| v.to_json()).transpose()?;
    let idea = string(short.as_ref().or(input.as_ref()), &configured)?;
    let cwd = string(path(&server.0, "cwd"), &env.cwd.to_string_lossy())?;
    let build = match path(&client.0, "build").filter(|v| truthy(v)) {
        Some(value) => PathBuf::from(string(Some(value), "")?),
        None => PathBuf::from(cwd)
            .join("node_modules")
            .join(string(path(&client.0, "package"), "stackpress-client")?),
    };
    Ok(Options {
        idea,
        tsconfig: string(path(&client.0, "tsconfig"), "")?,
        build,
        lang: string(path(&client.0, "lang"), "js")?,
        prettier: path(&client.0, "prettier").cloned().unwrap_or(json!({})),
    })
}

/// Source Generate event: prior-error and missing-client skips precede required tsconfig.
/// # Errors
/// Propagates native option/provider failures; intentionally public source errors stay in response.
pub async fn event(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if super::server::skipped(res) || !inv.config::<ConfigPresence>()?.contains("client") {
        return Ok(Flow::Continue);
    }
    let client = inv.config::<ClientOptions>()?;
    if path(&client.0, "tsconfig").is_none_or(|v| !truthy(v)) {
        return super::server::error(res, "Missing tsconfig path");
    }
    let env = inv.service::<Environment>().cloned().unwrap_or_default();
    let options = options(
        req,
        client,
        inv.config::<ServerOptions>()?,
        inv.config::<CliOptions>()?,
        &env,
    )?;
    let flow = super::server::invoke("generate", serde_json::to_value(options)?, res, inv).await?;
    res.status_code(200, None)?;
    Ok(flow)
}

#[cfg(feature = "codegen")]
pub mod rust {
    //! Real existing Rust generation APIs; executable selection remains compiled application code.
    use super::*;
    pub use stackpress_framework_schema::codegen::{DefinitionRegistry, NormalizeOptions};
    pub use stackpress_idea::{GenerateRequest, GenerationMode, generate, parser, transformer};

    /// Add the three shipped package sources to an explicit application source loader.
    ///
    /// Imports are `stackpress/stackpress.idea`, `stackpress-api/schema.idea` and
    /// `stackpress-session/schema.idea`. This build-host catalog selects no runtime plugin.
    /// # Errors
    /// Preserves invalid/duplicate catalog registration errors from PackageLoader.
    pub fn package_loader<L: transformer::SourceLoader>(
        fallback: L,
    ) -> std::result::Result<transformer::PackageLoader<L>, transformer::LoadError> {
        transformer::PackageLoader::new(fallback)
            .with_source("stackpress/stackpress.idea", crate::SCHEMA_SOURCE)?
            .with_source(
                "stackpress-api/schema.idea",
                stackpress_framework_api::SCHEMA_SOURCE,
            )?
            .with_source(
                "stackpress-session/schema.idea",
                stackpress_framework_session::SCHEMA_SOURCE,
            )
    }

    /// Statically selected build-host callbacks; dynamic TS modules become Rust functions.
    pub struct Host {
        /// Construct each run's independent registry, normally using register below.
        pub registry: fn() -> transformer::Result<transformer::Registry>,
        /// Render the application-owned generated root module.
        pub aggregates:
            fn(&parser::SchemaConfig) -> transformer::Result<Vec<transformer::GeneratedFile>>,
        /// Explicit output comparison/installation decision.
        pub mode: GenerationMode,
    }

    /// Binding for the generate event using real composition, normalization, emission and output IO.
    ///
    /// Bind this with `Bindings::bind("generate", rust::provider)` and publish Host during Config.
    /// TS config/lang/Prettier remain source metadata; Rust emission uses the existing toolchain.
    /// # Errors
    /// Preserves source loading, registry/transform, ownership, formatting and filesystem failures.
    pub async fn provider(
        req: &mut Request,
        _: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        let options: Options = serde_json::from_value(req.data.to_json()?)?;
        let host = inv.service::<Host>()?;
        let env = inv.service::<Environment>().cloned().unwrap_or_default();
        let cwd = string(
            path(&inv.config::<ServerOptions>()?.0, "cwd"),
            &env.cwd.to_string_lossy(),
        )?;
        let loader =
            package_loader(transformer::ProjectLoader::new(cwd).map_err(Error::from_source)?)
                .map_err(Error::from_source)?;
        let workspace =
            transformer::OutputWorkspace::new(&options.build).map_err(Error::from_source)?;
        let registry = (host.registry)().map_err(Error::from_source)?;
        generate(
            GenerateRequest::new(&options.idea, host.aggregates, host.mode),
            loader,
            &registry,
            &workspace,
        )
        .map_err(Error::from_source)?;
        Ok(Flow::Continue)
    }

    /// Register source schema then SQL transforms with their existing prerequisite/partition rules.
    /// A view transform remains app-owned because compiled Leptos modules replace Node imports.
    /// # Errors
    /// Preserves actual registry duplicate, dependency and output ownership errors.
    pub fn register(
        registry: &mut transformer::Registry,
        schema: transformer::ContextTransform,
        sql: Option<transformer::ContextTransform>,
        view: Option<transformer::ContextTransform>,
    ) -> transformer::Result<()> {
        stackpress_framework_schema::codegen::register(registry, schema)?;
        if let Some(sql) = sql {
            stackpress_framework_sql::codegen::register(registry, sql)?;
        }
        if let Some(view) = view {
            registry.add(
                transformer::TransformSpec::new("framework-view")
                    .owns(&["views"])
                    .requires(&["framework-schema"]),
                view,
            )?;
        }
        Ok(())
    }
}
