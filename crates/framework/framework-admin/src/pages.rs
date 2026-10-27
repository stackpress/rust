//! Public admin page contracts and honest mutation state, independent of host/SQL transports.
use crate::{Draft, search::SearchQuery};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult, UiModel};

/// Root-owned safe public issues, consumed by existing field frames without changing their ABI.
#[derive(Clone, Copy)]
pub struct FormIssues(pub RwSignal<Vec<stackpress_framework_schema::FieldIssue>>);

/// Host-selected parent fields may become read-only in every generated role at this root.
#[derive(Clone)]
pub struct LockedFields(pub Vec<String>);

/// Explicit public layout configuration. Action grants are supplied separately by host policy.
#[derive(Clone, Debug)]
pub struct AdminConfig {
    /// Validated same-application mount.
    pub mount: String,
    /// Explicit compiled page modules in navigation order.
    pub models: Vec<String>,
}
impl AdminConfig {
    /// Check mount and finite distinct portable navigation identities.
    /// # Errors
    /// Refuses empty/oversized selections, duplicate models and executable/path fragments.
    pub fn new(prefix: &str, models: Vec<String>) -> SchemaResult<Self> {
        let mut unique = std::collections::BTreeSet::new();
        if models.is_empty()
            || models.len() > 128
            || models.iter().any(|name| {
                name.is_empty()
                    || name.len() > 128
                    || !name.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
                    || !unique.insert(name)
            })
        {
            return Err(invalid("invalid admin model selection"));
        }
        Ok(Self {
            mount: mount(prefix)?,
            models,
        })
    }
}

/// Semantic layout slots over an explicit public host action projection; this authorizes no work.
#[component]
pub fn LayoutAdmin(
    /// Human page title, escaped as ordinary text.
    title: String,
    /// Checked selected models and mount.
    config: AdminConfig,
    /// Already authorized action labels/links, produced by the host-selected page controller.
    actions: Vec<(Action, String)>,
    /// Request/root-owned page content.
    children: Children,
) -> impl IntoView {
    let models = config
        .models
        .into_iter()
        .map(|module| {
            let href = format!("{}{module}/search", config.mount);
            view! {<a href=href>{module}</a>}
        })
        .collect_view();
    let actions = actions
        .into_iter()
        .map(|(action, href)| view! {<a data-admin-action=action.name() href=href>{action.name()}</a>})
        .collect_view();
    view! {<main class="sp-root"><h1>"Generated workspace admin"</h1><p>"Updates use last-write behavior."</p><nav aria-label="Models">{models}</nav><h2>{title}</h2><nav aria-label="Actions">{actions}</nav>{children()}</main>}
}

/// Explicit page actions; existence of a generated page grants no authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Bounded search/list.
    Search,
    /// Read detail.
    Detail,
    /// Create one approved public row.
    Create,
    /// Patch a complete identity.
    Update,
    /// Seed creation without identity.
    Copy,
    /// Explicit soft removal, where supported.
    Remove,
    /// Explicit restoration, where supported.
    Restore,
}
impl Action {
    /// Fixed route suffix, never a caller-provided event name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Detail => "detail",
            Self::Create => "create",
            Self::Update => "update",
            Self::Copy => "copy",
            Self::Remove => "remove",
            Self::Restore => "restore",
        }
    }
    /// Whether this action can submit a write; copy submits through Create.
    pub fn writes(self) -> bool {
        matches!(
            self,
            Self::Create | Self::Update | Self::Copy | Self::Remove | Self::Restore
        )
    }
}

/// One generated model's public page capability description, without an authority grant.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageSpec {
    /// Checked portable module/route identity.
    pub module: String,
    /// Actual revision-2 projected metadata.
    pub model: UiModel,
    /// Data-layer active field makes remove/restore available.
    pub soft_remove: bool,
}

/// One explicitly authorized parent relationship, selected by the compiled application route.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParentScope {
    /// Parent page module.
    pub module: String,
    /// Complete parent identity.
    pub key: Value,
    /// Child FK member selected by compiled relation metadata.
    pub field: String,
    /// Exact scalar FK value; never replaced by a supplied form value.
    pub value: Value,
}

/// Safe host-owned page seed; no identity provider, raw request or database resource enters it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    /// Explicit selected page specification.
    pub spec: PageSpec,
    /// Selected fixed action.
    pub action: Action,
    /// Authoritative action projection; empty means no visible grants.
    pub allowed: Vec<Action>,
    /// Validated application mount, ending with slash.
    pub mount: String,
    /// App-owned anti-forgery token; the finite example uses an explicit fixed proof token.
    pub csrf: String,
    /// Complete selected key, or empty object for search/create/copy.
    pub key: Value,
    /// Actual generated operation result, already publicly projected.
    pub data: Value,
    /// Validated URL search state.
    pub query: SearchQuery,
    /// Optional compiled related context.
    pub parent: Option<ParentScope>,
}

/// Native adapter response delivered back to its owning root.
#[derive(Clone, Debug)]
pub struct Reply {
    /// Actual HTTP status; a missing acknowledgement must not invent success.
    pub status: u16,
    /// Public response envelope, never private host diagnostics.
    pub envelope: Value,
}

/// One explicit client request. The platform adapter owns cancellation and delivery.
#[derive(Clone)]
pub struct Command {
    /// Fixed same-mount compiled API URL.
    pub endpoint: String,
    /// Strict public generated operation envelope.
    pub input: Value,
    /// CSRF token selected by the host snapshot.
    pub csrf: String,
    /// Distinguishes uncertain writes from failed reads on transport failure.
    pub write: bool,
    /// Completion remains inside the current root's lifetime.
    pub complete: Callback<Reply>,
}

/// Form rendering arguments for a generated model-specific component.
#[derive(Clone)]
pub struct Form {
    /// Creation/update/copy selects identity behavior.
    pub action: Action,
    /// Caller-owned exact draft.
    pub draft: Draft,
    /// Unique instance prefix.
    pub instance: String,
    /// FK field locked by related scope.
    pub locked: Option<String>,
}

/// Safe visible result category; server acknowledgement determines success.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteState {
    /// No submission yet.
    Idle,
    /// One request is in flight; duplicate submits are disabled.
    Pending,
    /// Confirmed successful acknowledgement.
    Saved,
    /// Public validation/authorization/operation failure, retaining the same draft.
    Failed(String),
    /// Submitted write outcome is unknown; there is no automatic retry.
    Unknown,
}
impl WriteState {
    /// Interpret only explicit public acknowledgement fields; no refetch-based inference.
    pub fn reply(reply: &Reply) -> Self {
        if reply
            .envelope
            .pointer("/errors/outcome")
            .and_then(Value::as_str)
            == Some("unknown")
        {
            return Self::Unknown;
        }
        if (200..300).contains(&reply.status)
            && reply
                .envelope
                .get("code")
                .and_then(Value::as_u64)
                .is_some_and(|code| (200..300).contains(&code))
        {
            Self::Saved
        } else {
            Self::Failed(
                reply
                    .envelope
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("Operation failed")
                    .into(),
            )
        }
    }
    /// Honest user-facing status, including the data layer's last-write behavior.
    pub fn message(&self) -> String {
        match self {Self::Idle=>String::new(),Self::Pending=>"Submitting…".into(),Self::Saved=>"Saved".into(),Self::Failed(message)=>message.clone(),Self::Unknown=>"Write outcome is unknown. Your draft is preserved. Check the record before deciding whether to submit again; no automatic retry was made.".into()}
    }
}

/// Extract a complete public identity without guessing a string ID or list position.
/// # Errors
/// Refuses missing/null key members and unbounded/browser-inexact data.
pub fn key(spec: &PageSpec, row: &Value) -> SchemaResult<Value> {
    let mut key = serde_json::Map::new();
    for name in &spec.model.key {
        let value = row
            .get(name)
            .filter(|value| !value.is_null())
            .ok_or_else(|| invalid("public identity is missing"))?;
        key.insert(name.clone(), value.clone());
    }
    if key.is_empty() {
        return Err(invalid("public identity is unavailable"));
    }
    let value = Value::Object(key);
    stackpress_framework_schema::check_browser_json(&value)?;
    Ok(value)
}

/// Prepare a copy or update draft without treating the stored identity as editable input.
pub fn input(spec: &PageSpec, action: Action, mut value: Value) -> Value {
    if matches!(action, Action::Copy | Action::Update)
        && let Some(object) = value.as_object_mut()
    {
        for key in &spec.model.key {
            object.remove(key);
        }
    }
    value
}

/// One checked mount; route assembly never accepts an origin, query or relative traversal.
/// # Errors
/// Refuses malformed mounts before compiling routes or public links.
pub fn mount(value: &str) -> SchemaResult<String> {
    if !value.starts_with('/')
        || !value.ends_with('/')
        || value.len() > 128
        || value.contains("//")
        || value
            .split('/')
            .filter(|part| !part.is_empty())
            .any(|part| {
                part == "."
                    || part == ".."
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
            })
    {
        return Err(invalid("invalid admin mount"));
    }
    Ok(value.into())
}

/// Safe typed context for metadata/route/input refusals.
fn invalid(context: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, context)
}
