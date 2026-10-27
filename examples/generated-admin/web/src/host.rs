//! Fixed compiled page/API routes delegating to the actual generated operation handlers.
use generated_admin_shared::ui::pages;
use serde_json::{Value, json};
use stackpress_framework_admin::{
    pages::{Action, PageSpec, ParentScope, Snapshot},
    search::{SearchQuery, decode, encode},
};
use stackpress_framework_sql::operations::{Operation, OperationPolicy, Policy, Registration};
use stackpress_lib::{Body, Error, Flow, Invocation, Nest, Request, Response, Result, queue::task};
use stackpress_server::{Server, ServerBuilder, ServerInvocationExt, StartupContext};
mod origin;
pub mod transfer;

use std::{collections::BTreeMap, sync::Arc};

/// Explicit finite-example host configuration. Authentication/session integration is app-owned.
pub struct Config {
    /// Validated route/asset mount.
    pub mount: String,
    /// Explicit finite fixture anti-forgery token; never a production/session implementation.
    pub csrf: String,
    /// CSR emits the same public seed with an empty initial root.
    pub csr: bool,
    /// Downward-only G36 import/export bounds.
    pub transfer_limits: stackpress_framework_admin::transfer::Limits,
}

/// Labeled fixture policy: the trusted finite harness supplies an actor header.
/// It is intentionally not a cookie/session/authentication implementation.
pub struct FixturePolicy;
impl OperationPolicy for FixturePolicy {
    /// Unknown actors and every reader mutation fail before decoding or database work.
    fn authorize(&self, _: &'static str, operation: Operation, request: &Request) -> Result<()> {
        match request.headers.get("x-admin-actor") {
            Some("editor") => Ok(()),
            Some("reader")
                if matches!(
                    operation,
                    Operation::Get | Operation::Detail | Operation::Search
                ) =>
            {
                Ok(())
            }
            _ => Err(Error::new("ADMIN_PRIVATE_POLICY_REFUSAL")),
        }
    }
}

/// Build explicit selected capabilities and fixed routes without binding a socket.
/// # Errors
/// Preserves configuration/startup errors and validates the mount before route registration.
pub async fn build(
    engine: Arc<stackpress_sql::Engine>,
    config: Config,
    policy: Arc<dyn OperationPolicy>,
) -> Result<Server> {
    let mut builder = ServerBuilder::new();
    configure(
        &mut builder,
        engine,
        config,
        policy,
        "https://admin.invalid".into(),
    )?;
    builder.build().await.map_err(|error| error.error)
}
/// Exact trusted HTTP mutation origin, independent of browser DTO input.
struct MutationOrigin(String);
/// Add current generated routes and operation registration to a caller-owned host.
/// # Errors
/// Validates mount, origin, CSRF and transfer limits before startup.
pub fn configure(
    builder: &mut ServerBuilder,
    engine: Arc<stackpress_sql::Engine>,
    config: Config,
    policy: Arc<dyn OperationPolicy>,
    origin: String,
) -> Result<()> {
    origin::validate(&origin)?;
    stackpress_framework_admin::pages::mount(&config.mount).map_err(invalid_cause)?;
    if config.csrf.is_empty() || config.csrf.len() > 128 {
        return Err(Error::invalid("invalid admin CSRF configuration"));
    }
    config.transfer_limits.validate().map_err(invalid_cause)?;
    let mount = config.mount.clone();
    let config = Arc::new(config);
    let policy = Arc::new(Policy(policy));
    builder.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        let config = config.clone();
        let policy = policy.clone();
        let origin = origin.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(config)?;
            start.register(policy)?;
            start.register(Arc::new(MutationOrigin(origin)))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.on_listen(task(|start: &mut StartupContext| {
        Box::pin(async move {
            crate::generated::pages_host::operations(start, &mut Registration::default())?;
            Ok(Flow::Continue)
        })
    }))?;
    crate::generated::pages_host::routes(builder, &mount)?;
    Ok(())
}

/// Resolve only explicitly generated page models; caller text never selects an event import.
pub fn spec(module: &str) -> Result<PageSpec> {
    pages::spec(module).map_err(invalid_cause)
}

/// Map one fixed suffix to the generated data capability; no batch/upsert/delete exposure.
fn operation(name: &str) -> Result<Operation> {
    match name {
        "get" => Ok(Operation::Get),
        "detail" => Ok(Operation::Detail),
        "search" => Ok(Operation::Search),
        "create" | "copy" => Ok(Operation::Create),
        "update" => Ok(Operation::Update),
        "remove" => Ok(Operation::Remove),
        "restore" => Ok(Operation::Restore),
        "import" => Ok(Operation::Batch),
        "export" => Ok(Operation::Search),
        _ => Err(Error::invalid("admin action unavailable")),
    }
}

/// Fixed page action vocabulary; Copy authorizes Create and loads through generated Get.
fn action(name: &str) -> Result<Action> {
    match name {
        "detail" => Ok(Action::Detail),
        "search" => Ok(Action::Search),
        "create" => Ok(Action::Create),
        "copy" => Ok(Action::Copy),
        "update" => Ok(Action::Update),
        "remove" => Ok(Action::Remove),
        "restore" => Ok(Action::Restore),
        "import" => Ok(Action::Import),
        "export" => Ok(Action::Export),
        _ => Err(Error::invalid("admin page unavailable")),
    }
}

/// Admit every capability required to render an action, including its initial read.
fn authorize_action(
    policy: &Policy,
    model: &'static str,
    action: Action,
    request: &Request,
) -> Result<()> {
    stackpress_framework_sql::operations::authorize(
        policy,
        model,
        operation(action.name())?,
        request,
    )?;
    if action == Action::Import {
        stackpress_framework_sql::operations::authorize(policy, model, Operation::Create, request)?;
        stackpress_framework_sql::operations::authorize(policy, model, Operation::Get, request)?;
    }
    if matches!(action, Action::Copy | Action::Update) {
        stackpress_framework_sql::operations::authorize(policy, model, Operation::Get, request)?;
    }
    Ok(())
}

/// Derive the static model spelling retained by the actual generated policy boundary.
fn model(module: &str) -> Result<&'static str> {
    pages::identity(module)
        .map(|(_, name)| name)
        .ok_or_else(|| Error::invalid("admin model unavailable"))
}

/// Native URL selection is checked against the fixed route shape after compiled routing matched.
fn route(request: &Request, config: &Config, api: bool) -> Result<(&'static str, &'static str)> {
    let path = request
        .url
        .path()
        .strip_prefix(&config.mount)
        .ok_or_else(|| Error::invalid("admin mount mismatch"))?;
    let path = if api {
        path.strip_prefix("api/")
            .ok_or_else(|| Error::invalid("admin API unavailable"))?
    } else {
        path
    };
    let mut parts = path.split('/');
    let module = parts.next().unwrap_or_default();
    let action = parts.next().unwrap_or_default();
    if parts.next().is_some() {
        return Err(Error::invalid("admin path unavailable"));
    }
    let module = pages::identity(module)
        .map(|(module, _)| module)
        .ok_or_else(|| Error::invalid("admin model unavailable"))?;
    let action = match action {
        "get" => "get",
        "preview" => "preview",
        "confirm" => "confirm",
        name => self::action(name)?.name(),
    };
    Ok((module, action))
}

/// Strict one-decode parameters. Neither repeated values nor extra keys are accepted.
fn parameters(request: &Request) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for pair in request
        .url
        .query()
        .filter(|query| !query.is_empty())
        .into_iter()
        .flat_map(|query| query.split('&'))
    {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| Error::invalid("invalid admin URL"))?;
        if !matches!(key, "state" | "key" | "parent") || values.contains_key(key) {
            return Err(Error::invalid("invalid admin URL parameters"));
        }
        values.insert(key.into(), decode(value).map_err(invalid_cause)?);
    }
    Ok(values)
}

/// Read one bounded structural parameter without inventing missing identities.
fn json_parameter(values: &BTreeMap<String, String>, name: &str) -> Result<Value> {
    let json = values
        .get(name)
        .ok_or_else(|| Error::invalid("admin identity required"))?;
    let value: Value = serde_json::from_str(json).map_err(invalid_cause)?;
    stackpress_framework_schema::check_browser_json(&value).map_err(invalid_cause)?;
    Ok(value)
}

/// Delegate with owned input and the original authority headers; no raw native request escapes.
async fn call(
    request: &Request,
    invocation: &Invocation<'_>,
    module: &str,
    name: &str,
    input: Value,
) -> Result<Value> {
    let mut child = Request::new();
    child.headers = request.headers.clone();
    child.url = request.url.clone();
    child.data = Nest::from_object(input)?;
    let mut response = Response::new();
    if invocation
        .delegate(&format!("{module}.{name}"), &mut child, &mut response)
        .await?
        == Flow::Abort
    {
        return Err(Error::invalid("admin operation aborted"));
    }
    match response.body() {
        Body::Value(value) => value.to_json(),
        _ => Err(Error::invalid("admin operation returned no result")),
    }
}

/// Authorize and validate a compiled relation, then prove the parent exists through generated Get.
async fn parent(
    request: &Request,
    invocation: &Invocation<'_>,
    spec: &PageSpec,
    values: &BTreeMap<String, String>,
) -> Result<Option<ParentScope>> {
    if !values.contains_key("parent") {
        return Ok(None);
    }
    let raw = json_parameter(values, "parent")?;
    let scope: ParentScope = serde_json::from_value(raw).map_err(invalid_cause)?;
    let parent_model = model(&scope.module)?;
    let policy = invocation.service::<Policy>()?;
    stackpress_framework_sql::operations::authorize(policy, parent_model, Operation::Get, request)?;
    let mapping = spec
        .model
        .fields
        .iter()
        .filter_map(|field| field.relation.as_ref())
        .find(|relation| {
            relation.target == parent_model
                && relation
                    .mapping
                    .as_ref()
                    .is_some_and(|mapping| mapping.local == scope.field)
        })
        .and_then(|relation| relation.mapping.as_ref())
        .ok_or_else(|| Error::invalid("related scope unavailable"))?;
    if scope.key.get(&mapping.foreign) != Some(&scope.value) {
        return Err(Error::invalid("related scope identity mismatch"));
    }
    let _ = call(
        request,
        invocation,
        &scope.module,
        "get",
        json!({"key":scope.key}),
    )
    .await?;
    Ok(Some(scope))
}

/// Admit selected JSON API calls, checking policy and CSRF before parsing the public envelope.
pub async fn api(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let config = invocation.service::<Config>()?;
    let (module, name) = route(request, config, true)?;
    let selected = operation(name)?;
    let descriptor = spec(module)?;
    let policy = invocation.service::<Policy>()?;
    stackpress_framework_sql::operations::authorize(policy, model(module)?, selected, request)?;
    // Unsafe HTTP admission is separate from UI visibility and generated operation policy.
    if !matches!(
        selected,
        Operation::Get | Operation::Detail | Operation::Search
    ) && (request.headers.get("origin")
        != Some(invocation.service::<MutationOrigin>()?.0.as_str())
        || request.headers.get("x-admin-csrf") != Some(config.csrf.as_str()))
    {
        return Err(Error::new("admin CSRF refused")
            .with_code(403)
            .with_public_message("operation is forbidden"));
    }
    // Query parameters describe route scope; only the separately decoded POST object is the
    // generated operation envelope. Retaining aggregate query members would violate strict DTOs.
    request.data = request.post.clone();
    let values = parameters(request)?;
    if values.keys().any(|key| key != "parent") {
        return Err(Error::invalid("unused admin API parameter"));
    }
    let scope = parent(request, &invocation, &descriptor, &values).await?;
    if let Some(scope) = scope {
        let mut input = request.data.to_json()?;
        if selected == Operation::Create {
            let fields = input
                .get_mut("input")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| Error::invalid("related create input required"))?;
            if fields
                .get(&scope.field)
                .is_some_and(|value| value != &scope.value)
            {
                return Err(Error::invalid("related parent cannot be replaced"));
            }
            fields.insert(scope.field, scope.value);
        } else if selected == Operation::Search {
            let filters = input
                .get_mut("filters")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| Error::invalid("related query filters required"))?;
            filters.push(json!({"field":scope.field,"predicate":{"op":"eq","value":scope.value}}));
        } else {
            return Err(Error::invalid("related action unavailable"));
        }
        request.data = Nest::from_object(input)?;
    }
    invocation
        .delegate(&format!("{module}.{name}"), request, response)
        .await
}

/// Load actual generated data before synchronously rendering one request-local Leptos root.
pub async fn page(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let config = invocation.service::<Config>()?;
    let (module, name) = route(request, config, false)?;
    let descriptor = spec(module)?;
    let selected = action(name)?;
    let policy = invocation.service::<Policy>()?;
    authorize_action(policy, model(module)?, selected, request)?;
    let values = parameters(request)?;
    let scope = parent(request, &invocation, &descriptor, &values).await?;
    let mut query = values
        .get("state")
        .map(|json| serde_json::from_str::<SearchQuery>(json).map_err(invalid_cause))
        .transpose()?
        .unwrap_or_default();
    if let Some(parent) = &scope {
        query.filter(&parent.field, parent.value.clone());
    }
    let key = if matches!(
        selected,
        Action::Detail | Action::Update | Action::Copy | Action::Remove | Action::Restore
    ) {
        json_parameter(&values, "key")?
    } else {
        json!({})
    };
    let data = match selected {
        Action::Search => {
            call(
                request,
                &invocation,
                module,
                "search",
                query.operation(&descriptor.model).map_err(invalid_cause)?,
            )
            .await?
        }
        Action::Detail => call(request, &invocation, module, "detail", json!({"key":key})).await?,
        Action::Update | Action::Copy => {
            call(request, &invocation, module, "get", json!({"key":key})).await?
        }
        _ => json!({}),
    };
    // A copy seed itself excludes identity, independently of browser form behavior.
    let data = if selected == Action::Copy {
        stackpress_framework_admin::pages::input(&descriptor, Action::Copy, data)
    } else {
        data
    };
    if let Some(parent) = &scope {
        if matches!(selected, Action::Detail) && data.get(&parent.field) != Some(&parent.value) {
            return Err(stackpress_framework_sql::operations::not_found());
        }
        if !matches!(
            selected,
            Action::Search | Action::Create | Action::Detail | Action::Import | Action::Export
        ) {
            return Err(Error::invalid("related page action unavailable"));
        }
    }
    let allowed = [
        Action::Search,
        Action::Detail,
        Action::Create,
        Action::Update,
        Action::Copy,
        Action::Remove,
        Action::Restore,
        Action::Import,
        Action::Export,
    ]
    .into_iter()
    .filter(|action| descriptor.soft_remove || !matches!(action, Action::Remove | Action::Restore))
    .filter(|action| {
        scope.is_none()
            || matches!(
                action,
                Action::Search | Action::Create | Action::Detail | Action::Import | Action::Export
            )
    })
    .filter(|action| {
        authorize_action(
            policy,
            model(module).expect("selected model"),
            *action,
            request,
        )
        .is_ok()
    })
    .collect();
    let snapshot = Snapshot {
        transfer_limits: config.transfer_limits,
        spec: descriptor,
        action: selected,
        allowed,
        mount: config.mount.clone(),
        csrf: config.csrf.clone(),
        key: if selected == Action::Copy {
            json!({})
        } else {
            key
        },
        data,
        query,
        parent: scope,
    };
    response.html(crate::render::document(snapshot, config.csr).map_err(invalid_cause)?)?;
    response.headers_mut()?.set("cache-control", "no-store")?;
    Ok(Flow::Continue)
}

/// Build a related context from compiled public metadata, never a display template.
pub fn related(module: &str, key: Value, field: &str, value: Value) -> String {
    format!(
        "parent={}",
        encode(&json!({"module":module,"key":key,"field":field,"value":value}).to_string())
    )
}

/// A safe route/input context retains concrete parser/metadata causes privately.
fn invalid_cause(cause: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::invalid("invalid admin request").with_cause(cause)
}
