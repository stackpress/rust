//! Source generated admin page actions, using API model events and the public CSRF mechanism.
use super::{
    management::{Management, ManagementPage, Ui},
    roles::{Choice, Relations},
};
use leptos::prelude::*;
use serde_json::{Value, json};
use stackpress_framework_admin::pages::Action;
use stackpress_framework_sql::source::{SessionModels, SourceActions, SourceModel, SourceStore};
use stackpress_lib::{
    Body, Error, Flow, Invocation, MethodPattern, Nest, Request, Response, Result, handler,
};
use stackpress_server::{RouteSpec, ServerBuilder, ServerInvocationExt};
use std::sync::Arc;

/// App-selected source presentation; registration opens no database and owns no identity.
#[derive(Clone, Debug)]
pub struct Config {
    /// Source admin name/base/menu and brand/view objects.
    pub presentation: Value,
    /// Reuse the session slice's independent CSRF service.
    pub csrf: stackpress_framework_csrf::Service,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            presentation: json!({"admin":{"name":"Admin","base":"/admin","menu":[]}}),
            csrf: Default::default(),
        }
    }
}
// One inventory serves immediate and source Route-phase contributions.
macro_rules! register_routes {
    ($target:expr, $ui:expr, $config:expr, $modules:expr, $family:ty) => {{
        let config = Arc::new($config);
        let base = config
            .presentation
            .pointer("/admin/base")
            .and_then(Value::as_str)
            .unwrap_or("/admin");
        for &module in $modules {
            let spec = ($ui.spec)(module).map_err(Error::from_source)?;
            for action in [
                Action::Search,
                Action::Detail,
                Action::Create,
                Action::Update,
                Action::Copy,
                Action::Remove,
                Action::Restore,
                Action::Import,
                Action::Export,
            ] {
                let keyed = matches!(
                    action,
                    Action::Detail
                        | Action::Update
                        | Action::Copy
                        | Action::Remove
                        | Action::Restore
                );
                if keyed && spec.model.key.is_empty() {
                    continue;
                }
                // Upstream routes.ts uses create/<keys> for copy. Retain the existing
                // Rust copy/<keys> spelling as an alias for callers already using it.
                let names = if action == Action::Copy {
                    &["create", "copy"][..]
                } else {
                    &[action.name()][..]
                };
                for name in names {
                    let mut path = format!("{base}/{module}/{name}");
                    if keyed {
                        for key in &spec.model.key {
                            path.push_str(&format!("/:{key}"));
                        }
                    }
                    let config = config.clone();
                    $target.route_with(
                        RouteSpec::new(MethodPattern::Any, &path),
                        handler(move |req, res, inv| {
                            let config = config.clone();
                            Box::pin(async move {
                                page::<$family>(req, res, inv, module, action, $ui, &config).await
                            })
                        }),
                    )?;
                }
            }
        }
    }};
}

/// Register the selected generated admin inventory through ordinary shared routes.
/// # Errors
/// Returns native route construction errors; no route conflict arbitration occurs.
pub fn register(app: &mut ServerBuilder, ui: Ui, config: Config) -> Result<()> {
    register_with::<stackpress_framework_models::Models>(
        app,
        ui,
        config,
        &["application", "session"],
    )
}

/// Register ordinary source admin routes for selected generated models and one actual family.
/// The application owns the module catalog and UI; this installs no Engine, schema or guard.
/// Item paths use all source key fields in declaration order. Copy uses `create/<keys>`;
/// the existing Rust `copy/<keys>` spelling remains an alias for the same handler.
/// # Errors
/// Preserves ordinary route errors and the same invocation-time action/render failures.
pub fn register_with<M: SessionModels>(
    app: &mut ServerBuilder,
    ui: Ui,
    config: Config,
    modules: &[&'static str],
) -> Result<()> {
    register_routes!(app, ui, config, modules, M);
    Ok(())
}

/// Queue generated source admin routes at the source Route phase; direct registration stays immediate.
/// No Engine, policy or schema setup is selected by this contribution.
/// # Errors
/// Returns ordinary startup errors; route failures surface while building the host.
pub fn register_queued_with<M: SessionModels>(
    app: &mut ServerBuilder,
    ui: Ui,
    config: Config,
    modules: &[&'static str],
) -> Result<()> {
    let modules = modules.to_vec();
    app.on_route(stackpress_lib::queue::task(
        move |start: &mut stackpress_server::StartupContext| {
            let modules = modules.clone();
            let config = config.clone();
            Box::pin(async move {
                register_routes!(start, ui, config, &modules, M);
                Ok(Flow::Continue)
            })
        },
    ))
}

/// Source prior-body/non-200 guard, shared event response and page-owned mutation sequencing.
async fn page<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
    module: &str,
    action: Action,
    ui: Ui,
    config: &Config,
) -> Result<Flow> {
    if super::super::pages::completed(res) {
        return Ok(Flow::Continue);
    }
    let original = req.data.to_json()?;
    let noview = config
        .presentation
        .pointer("/view/noview")
        .and_then(Value::as_str)
        .unwrap_or("json");
    let mut csrf = None;
    if matches!(
        action,
        Action::Create | Action::Update | Action::Copy | Action::Remove | Action::Restore
    ) {
        let token = config.csrf.generate(res)?;
        csrf = Some((config.csrf.name().to_owned(), token));
    }
    if matches!(action, Action::Detail | Action::Restore) {
        req.data
            .set_path(&stackpress_lib::DataPath::dot("eq.active")?, -1)?;
    }
    let confirmed = req
        .data
        .get("confirmed")
        .is_some_and(super::super::pages::truthy);
    let write = match action {
        Action::Create | Action::Copy | Action::Import => req.method.as_str() == "POST",
        Action::Update => matches!(req.method.as_str(), "POST" | "PUT"),
        Action::Remove | Action::Restore => confirmed,
        _ => false,
    };
    if write && action != Action::Import && !config.csrf.valid(req, res)? {
        if matches!(action, Action::Remove | Action::Restore) {
            res.session_mut()?.set("flash",json!({"type":"error","message":stackpress_framework_csrf::DEFAULT_ERROR,"close":2000}).to_string(),None)?;
            let base = config
                .presentation
                .pointer("/admin/base")
                .and_then(Value::as_str)
                .unwrap_or("/admin");
            let id = req
                .data
                .get("id")
                .and_then(stackpress_lib::Value::as_str)
                .unwrap_or("");
            res.redirect(&format!("{base}/{module}/{}/{id}", action.name()))?;
        }
        return Ok(Flow::Continue);
    }
    if action == Action::Search {
        // The generated source coerces numeric pagination before resolving its search event.
        for key in ["skip", "take"] {
            if let Some(value) = req
                .data
                .get(key)
                .and_then(stackpress_lib::Value::as_str)
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite())
            {
                req.data
                    .set(key, stackpress_lib::Value::try_from(json!(value))?)?;
            }
        }
        inv.emit(&format!("{module}-search"), req, res).await?;
        project_results(res)?;
    } else if action == Action::Export {
        let mut data = req.data.clone();
        data.set("take", 0)?;
        let found = crate::helpers::resolve(&inv, &format!("{module}-search"), data).await?;
        if found.status().map(|status| status.code) == Some(200) {
            let rows = found
                .result()
                .map(wire_json)
                .transpose()?
                .unwrap_or(json!([]));
            let csv = export(&rows)?;
            res.headers_mut()?.set(
                "Content-Disposition",
                &format!(
                    "attachment; filename={module}-{}.csv",
                    (crate::pages::Environment::default().now)()?
                ),
            )?;
            res.set("text/csv", Body::Text(csv))?;
        }
        return Ok(Flow::Continue);
    } else if write {
        if action == Action::Import
            && let Some(csv) = req.data.get("csv").and_then(stackpress_lib::Value::as_str)
        {
            let rows = super::management::import_csv(csv).map_err(Error::from_source)?;
            req.data
                .set("rows", stackpress_lib::Value::try_from(json!(rows))?)?;
        }
        let event = match action {
            Action::Copy => "create",
            Action::Import => "batch",
            _ => action.name(),
        };
        if action == Action::Copy {
            // The original copy page forwards only @field members, so keys/secrets regenerate.
            let spec = (ui.spec)(module).map_err(Error::from_source)?;
            let mut data = Nest::new();
            for field in spec.model.fields.iter().filter(|field| {
                field
                    .roles
                    .iter()
                    .any(|binding| binding.role == stackpress_framework_schema::Role::Field)
            }) {
                if let Some(value) = req.data.get(&field.wire_name) {
                    data.set(&field.wire_name, value.clone())?;
                }
            }
            crate::helpers::resolve_into(&inv, &format!("{module}-{event}"), data, res).await?;
        } else {
            inv.emit(&format!("{module}-{event}"), req, res).await?;
        }
        project_results(res)?;
        // The source rejects a non-object creation result even with noview. A
        // composite identity is a valid object; it need not contain a member named id.
        if matches!(action, Action::Create | Action::Copy)
            && res.status().map(|status| status.code) == Some(200)
            && res.result().and_then(|row| row.as_object()).is_none()
        {
            return Err(Error::invalid("Unknown creation response results"));
        }
        if res.status().map(|status| status.code) == Some(200)
            && !req.data.has(noview)
            && action != Action::Import
        {
            let base = config
                .presentation
                .pointer("/admin/base")
                .and_then(Value::as_str)
                .unwrap_or("/admin");
            let spec = (ui.spec)(module).map_err(Error::from_source)?;
            let creation = matches!(action, Action::Create | Action::Copy);
            if action == Action::Remove || (creation && spec.model.key.len() != 1) {
                res.redirect(&format!("{base}/{module}/search"))?;
            } else {
                // Create/copy interpolate the returned row; update/restore use the
                // request's complete declared identity, in source declaration order.
                let data = if creation {
                    res.result().map(wire_json).transpose()?.unwrap_or_default()
                } else {
                    // Update may add native Dates to unrelated request fields. Only
                    // interpolate the key members; do not JSON-project the whole request.
                    let mut keys = serde_json::Map::new();
                    for key in &spec.model.key {
                        if let Some(value) = req.data.get(key) {
                            keys.insert(key.clone(), wire_json(value)?);
                        }
                    }
                    Value::Object(keys)
                };
                let id = super::identity::path(&spec, &data, &Value::Null).unwrap_or_default();
                res.redirect(&format!("{base}/{module}/detail/{id}"))?;
            }
        }
    } else if !matches!(action, Action::Create | Action::Import) {
        inv.emit(&format!("{module}-detail"), req, res).await?;
        project_results(res)?;
    }
    if req.data.has(noview) {
        let json = res.try_to_json()?;
        let status = res.status().cloned();
        res.json(&json)?;
        if let Some(status) = status {
            res.status_code(status.code, Some(status.message.to_string()))?;
        }
        return Ok(Flow::Continue);
    }
    if res.redirected() {
        return Ok(Flow::Continue);
    }
    let data = res
        .result()
        .map(wire_json)
        .transpose()?
        .unwrap_or(json!({}));
    let relations = relations::<M>(&inv, &data).await?;
    let theme = super::host::presentation(req, &inv)?.theme;
    let page = Management {
        module: module.into(),
        action,
        data,
        query: original,
        relations,
        csrf,
        config: config.presentation.clone(),
        theme,
        error: res.error().map(str::to_owned),
    };
    let status = res.status().cloned();
    res.html(document(page, ui)?)?;
    if let Some(status) = status {
        res.status_code(status.code, Some(status.message.to_string()))?;
    }
    Ok(Flow::Continue)
}
/// Build labels from real joined results; choice lists are loaded by the app's public searches.
async fn relations<M: SessionModels>(inv: &Invocation<'_>, data: &Value) -> Result<Relations> {
    let mut relations = Relations::default();
    let rows = data
        .as_array()
        .map(|rows| rows.as_slice())
        .unwrap_or_else(|| std::slice::from_ref(data));
    for (field, relation) in [("profileId", "profile"), ("applicationId", "application")] {
        let mut choices = Vec::new();
        for row in rows {
            if let Some(joined) = row.get(relation)
                && let Some(id) = row.get(field).and_then(Value::as_str)
                && !choices.iter().any(|choice: &Choice| choice.id == id)
            {
                choices.push(Choice {
                    id: id.into(),
                    label: super::string(joined.get("name"), id),
                    row: joined.clone(),
                });
            }
        }
        relations.0.insert(field.into(), choices);
    }
    // Creation needs real selectable rows even before a particular foreign key is selected.
    if let Ok(engine) = inv.shared_service::<stackpress_sql::Engine>() {
        let profiles = M::Profile::actions(engine, String::new(), 0);
        let query = profiles
            .store()
            .query(&Default::default())
            .map_err(Error::from_source)?;
        let rows = profiles
            .find_all(&query)
            .await
            .map_err(Error::from_source)?;
        for row in rows {
            let row = serde_json::to_value(row)?;
            if let Some(id) = row.get("id").and_then(Value::as_str) {
                let choices = relations.0.entry("profileId".into()).or_default();
                if !choices.iter().any(|choice| choice.id == id) {
                    choices.push(Choice {
                        id: id.into(),
                        label: super::string(row.get("name"), id),
                        row: row.clone(),
                    });
                }
            }
        }
    }
    let apps = crate::helpers::resolve(inv, "application-search", Nest::new()).await?;
    if let Some(rows) = apps
        .result()
        .map(wire_json)
        .transpose()?
        .as_ref()
        .and_then(Value::as_array)
    {
        for row in rows {
            if let Some(id) = row.get("id").and_then(Value::as_str) {
                let choices = relations.0.entry("applicationId".into()).or_default();
                if !choices.iter().any(|choice| choice.id == id) {
                    choices.push(Choice {
                        id: id.into(),
                        label: super::string(row.get("name"), id),
                        row: row.clone(),
                    });
                }
            }
        }
    }
    Ok(relations)
}
/// Native Dates cross the JSON/HTML boundary here, after the nested source process finishes.
fn project_results(res: &mut Response) -> Result<()> {
    if res.status().map(|status| status.code) == Some(200)
        && let Some(value) = res.result()
    {
        let value = wire_json(value)?;
        let total = res.total();
        res.results(&value)?;
        res.set_total(total)?;
    }
    Ok(())
}
/// Source Date.toJSON at this host boundary; unsupported native payloads still fail.
fn wire_json(value: &stackpress_lib::Value) -> Result<Value> {
    use stackpress_lib::Value as Native;
    Ok(match value {
        Native::DateTime(time) => {
            let millis = time
                .seconds
                .checked_mul(1000)
                .and_then(|value| value.checked_add(i64::from(time.nanoseconds() / 1_000_000)))
                .ok_or_else(|| Error::invalid("source Date exceeds native milliseconds"))?;
            serde_json::to_value(stackpress_framework_schema::source::date(millis))?
        }
        Native::Array(values) => Value::Array(values.iter().map(wire_json).collect::<Result<_>>()?),
        Native::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), wire_json(value)?)))
                .collect::<Result<_>>()?,
        ),
        _ => value.to_json()?,
    })
}
/// Render locally and dispose the Leptos owner before yielding owned HTML to a Send future.
/// # Errors
/// Reports source seed or native render conversion errors.
pub fn document(page: Management, ui: Ui) -> Result<String> {
    super::document::render(|| {
        let title =
            view! {<title>{format!("{} {}",page.module,page.action.name())}</title>}.to_html();
        let body = view! {<ManagementPage page=page.clone() ui/>}.to_html();
        let seed = super::seed(&page)?;
        let marker = if page.theme.is_none() {
            " data-theme-unset"
        } else {
            ""
        };
        let bootstrap = super::document::bootstrap(page.theme)?;
        Ok(format!(
            "<!doctype html><html lang=\"en\" data-theme=\"{}\"{marker}><head>{bootstrap}<meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">{title}<link rel=\"stylesheet\" href=\"/styles/global.css\"></head><body><div id=\"app\">{body}</div><script id=\"api-management-seed\" type=\"application/json\">{seed}</script><script type=\"module\" src=\"/assets/api-checkpoint.js\"></script></body></html>",
            page.theme.map(|theme| theme.as_str()).unwrap_or("light")
        ))
    })
}
/// Source relation-export algorithm: JSON-stringified cells and dynamically discovered columns.
/// # Errors
/// Propagates JSON serialization errors without substituting spreadsheet escaping policy.
pub fn export(rows: &Value) -> Result<String> {
    let mut head: Vec<(String, String)> = Vec::new();
    let mut body = Vec::new();
    for row in rows.as_array().into_iter().flatten() {
        if let Some(row) = row.as_object() {
            for key in row
                .keys()
                .filter(|key| !matches!(key.as_str(), "profile" | "application"))
            {
                if !head.contains(&(String::new(), key.clone())) {
                    head.push((String::new(), key.clone()));
                }
            }
            for relation in ["profile", "application"] {
                if let Some(joined) = row.get(relation).and_then(Value::as_object) {
                    for key in joined.keys() {
                        if !head.contains(&(relation.into(), key.clone())) {
                            head.push((relation.into(), key.clone()));
                        }
                    }
                }
            }
            let cells = head
                .iter()
                .map(|(relation, key)| {
                    let value = if relation.is_empty() {
                        row.get(key)
                    } else {
                        row.get(relation).and_then(|row| row.get(key))
                    };
                    serde_json::to_string(
                        value
                            .filter(|value| !value.is_null())
                            .unwrap_or(&Value::String(String::new())),
                    )
                })
                .collect::<std::result::Result<Vec<_>, _>>()?;
            body.push(cells.join(","));
        }
    }
    let mut rows = vec![
        head.into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>()
            .join(","),
    ];
    rows.extend(body);
    Ok(rows.join("\n"))
}
