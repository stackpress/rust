//! Explicit generated operation policy, registration ownership and safe transport boundaries.
//!
//! There are no inferred routes, maintenance commands, ambient transactions or universal
//! authorization on trusted direct store calls. The app supplies one Policy service and
//! retains one Registration marker for its selected generated listeners.
use crate::{DataError, DataErrorKind, DataResult};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use stackpress_framework_schema::{
    DatetimeValue, RecordSchema, ValueContext, ValueProviders, ValueType,
};
pub use stackpress_lib::{
    Body, DeliveryState, Error, ErrorKind, Flow, Invocation, ListenerId, Request, Response, Result,
};
pub use stackpress_server::{ServerInvocationExt, StartupContext};
use std::{collections::BTreeSet, error::Error as StdError, sync::Arc};

/// Explicit operation selection; importing a model registers none of these capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Base public key lookup.
    Get,
    /// Fixed app-default detail projection/includes.
    Detail,
    /// Bounded approved query and eager public projection.
    Search,
    /// One approved create input.
    Create,
    /// One nonempty approved patch.
    Update,
    /// One explicit native primary-key UPSERT.
    Upsert,
    /// Finite atomic create/update/upsert batch.
    Batch,
    /// Active-model soft removal only.
    Remove,
    /// Active-model restoration only.
    Restore,
    /// Independently selected hard deletion.
    Delete,
}
impl Operation {
    /// Stable operation suffix; app-owned prefixes choose the actual event names.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Detail => "detail",
            Self::Search => "search",
            Self::Create => "create",
            Self::Update => "update",
            Self::Upsert => "upsert",
            Self::Batch => "batch",
            Self::Remove => "remove",
            Self::Restore => "restore",
            Self::Delete => "delete",
        }
    }
}
/// Owned host inputs for one authorized invocation; no machine clock/provider is assumed.
pub struct HostInputs {
    /// One explicit canonical clock shared by every preparation in this operation.
    pub now: Option<DatetimeValue>,
    /// App-owned generators and host business validators.
    pub providers: Option<Arc<dyn ValueProviders>>,
    /// Private/nonwritable create fields; generated members use providers instead.
    pub create: Value,
    /// Private mutable patch fields; identities/generated fields remain forbidden.
    pub patch: Value,
}
impl Default for HostInputs {
    /// Simple public-only models need no implicit authority, clock or private assembly.
    fn default() -> Self {
        Self {
            now: None,
            providers: None,
            create: serde_json::json!({}),
            patch: serde_json::json!({}),
        }
    }
}
impl HostInputs {
    /// Borrow the exact once-per-invocation clock/provider set across generated awaits.
    pub fn context(&self) -> ValueContext<'_> {
        ValueContext {
            now: self.now.as_ref(),
            providers: self.providers.as_deref(),
        }
    }
}
/// Required app-owned policy service. A model, route, UI role or nested dispatch grants no access.
pub trait OperationPolicy: Send + Sync {
    /// Authorize the exact requested operation before input decoding or database work.
    fn authorize(&self, model: &'static str, operation: Operation, request: &Request)
    -> Result<()>;
    /// Assemble trusted host inputs only after authorization; the default supplies no private values.
    fn values(
        &self,
        _model: &'static str,
        _operation: Operation,
        _request: &Request,
    ) -> DataResult<HostInputs> {
        Ok(HostInputs::default())
    }
}
/// Sized service wrapper for the explicitly registered app policy implementation.
pub struct Policy(pub Arc<dyn OperationPolicy>);
/// App-owned registration marker. Retain one per host's generated operation registration phase.
#[derive(Default)]
pub struct Registration {
    /// Selected exact event identities; separate hosts use separate markers.
    selected: BTreeSet<String>,
}
impl Registration {
    /// Reject duplicate/invalid selection before registering any listener from this call.
    pub fn claim(&mut self, prefix: &str, operations: &[Operation]) -> Result<()> {
        if prefix.is_empty()
            || prefix.len() > 128
            || !prefix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
            || operations.len() > 10
        {
            return Err(Error::invalid("invalid generated operation selection"));
        }
        // Validate the whole selection before mutating the marker or publishing listeners.
        let events = operations
            .iter()
            .map(|operation| format!("{prefix}.{}", operation.as_str()))
            .collect::<BTreeSet<_>>();
        if events.len() != operations.len()
            || events.iter().any(|event| self.selected.contains(event))
        {
            return Err(Error::invalid("duplicate generated operation registration"));
        }
        self.selected.extend(events);
        Ok(())
    }
}
/// Respect deliberate earlier completion even when the dispatcher continues to another listener.
pub fn completed(response: &Response) -> bool {
    response.delivery_state() != DeliveryState::Open
        || response.status().is_some()
        || response.error().is_some()
        || !response.errors().is_empty()
        || !matches!(response.body(), Body::Empty)
}
/// Sanitize policy refusal independently of the policy's private diagnostics.
pub fn authorize(
    policy: &Policy,
    model: &'static str,
    operation: Operation,
    request: &Request,
) -> Result<()> {
    policy
        .0
        .authorize(model, operation, request)
        .map_err(|cause| {
            Error::new("generated operation policy refused")
                .with_kind(ErrorKind::Domain)
                .with_code(403)
                .with_public_message("operation is forbidden")
                .with_cause(cause)
        })
}
/// Strict operation-specific DTO decoding with finite shared JSON/browser budgets.
pub fn decode<T: DeserializeOwned>(request: &Request) -> Result<T> {
    // Preserve the structured private cause while all public decoding failures use fixed text.
    let value = request.data.to_json().map_err(|cause| {
        Error::new("operation input conversion failed")
            .with_kind(ErrorKind::InvalidInput)
            .with_code(400)
            .with_public_message("invalid operation input")
            .with_cause(cause)
    })?;
    stackpress_framework_schema::check_json(&value)
        .and_then(|()| stackpress_framework_schema::check_browser_json(&value))
        .map_err(|cause| {
            Error::new("operation input budget failed")
                .with_kind(ErrorKind::InvalidInput)
                .with_code(400)
                .with_public_message("invalid operation input")
                .with_cause(cause)
        })?;
    serde_json::from_value(value).map_err(|cause| {
        Error::new("operation input decoding failed")
            .with_kind(ErrorKind::InvalidInput)
            .with_code(400)
            .with_public_message("invalid operation input")
            .with_cause(cause)
    })
}
/// Keep host configuration/provider failures private; they are not caller validation errors.
pub fn host_error(cause: DataError) -> Error {
    Error::new("operation host inputs failed")
        .with_kind(ErrorKind::External)
        .with_code(500)
        .with_public_message("operation could not be completed")
        .with_cause(cause)
}
/// Explicit operation-specific absence, without schema values or private diagnostics.
pub fn not_found() -> Error {
    Error::new("generated operation row absent")
        .with_kind(ErrorKind::Domain)
        .with_code(404)
        .with_public_message("record was not found")
}
/// Check whether a validation issue describes an approved input path at every embedding segment.
fn public_path(policy: &RecordSchema, path: &[String]) -> bool {
    let mut model = &policy.model;
    for part in path {
        if part.parse::<usize>().is_ok() {
            continue;
        }
        let Some(field) = model
            .fields
            .iter()
            .find(|field| field.name == *part && field.access.writable && !field.generated)
        else {
            return false;
        };
        if let ValueType::Fieldset(name) = &field.value_type {
            let Some(nested) = policy.fieldsets.iter().find(|model| model.name == *name) else {
                return false;
            };
            model = nested;
        }
    }
    !path.is_empty()
}
/// Map only proven categories, retaining typed private causes and bounded approved field issues.
pub fn data_error(cause: DataError, policy: &RecordSchema) -> Error {
    let validation = cause.kind() == DataErrorKind::Validation;
    let unknown = cause.kind() == DataErrorKind::UnknownOutcome;
    let mut error = Error::new("generated data operation failed")
        .with_kind(if validation {
            ErrorKind::InvalidInput
        } else {
            ErrorKind::External
        })
        .with_code(if validation { 400 } else { 500 })
        .with_public_message(if validation {
            "input validation failed"
        } else if unknown {
            "write outcome is unknown; do not retry automatically"
        } else {
            "operation could not be completed"
        });
    // Only bounded, approved field paths survive the private source-chain traversal.
    if validation {
        let mut source: Option<&(dyn StdError + 'static)> = Some(&cause);
        for _ in 0..8 {
            let Some(current) = source else {
                break;
            };
            if let Some(issues) =
                current.downcast_ref::<stackpress_framework_schema::ValidationErrors>()
            {
                let issues = issues
                    .issues
                    .iter()
                    .filter(|issue| public_path(policy, &issue.path))
                    .take(64)
                    .map(|issue| stackpress_framework_schema::FieldIssue {
                        path: issue
                            .path
                            .iter()
                            .take(32)
                            .map(|part| part.chars().take(512).collect())
                            .collect(),
                        code: "invalid".into(),
                        message: "field validation failed".into(),
                    })
                    .collect::<Vec<_>>();
                error = error.with_errors(serde_json::json!({"fields":issues}));
                break;
            }
            source = current.source();
        }
    } else if unknown {
        // This is an acknowledgement category, never an invitation to retry automatically.
        error = error.with_errors(serde_json::json!({"outcome":"unknown"}));
    }
    error.with_cause(cause)
}
/// Merge private fields through the same record graph without overriding approved inputs.
/// Generated fields remain provider-owned; patches cannot write identities. Public list
/// embeddings may receive private members only at already supplied matching element indexes.
pub fn assemble<T: Serialize, U: DeserializeOwned>(
    input: &T,
    host: &Value,
    policy: &RecordSchema,
    patch: bool,
    context: &ValueContext<'_>,
) -> DataResult<U> {
    let mut value =
        stackpress_framework_schema::to_bounded_value(input).map_err(crate::stores::validation)?;
    stackpress_framework_schema::check_browser_json(&value).map_err(crate::stores::validation)?;
    // Host fields cannot turn a caller's empty patch into an authorized mutation.
    if patch && value.as_object().is_some_and(|value| value.is_empty()) {
        return Err(DataError::new("public patch must be nonempty"));
    }
    let mut state = Assembly {
        policy,
        context,
        budget: stackpress_framework_schema::PreparationBudget::default(),
    };
    // Charge both supplied trees before host values/defaults can be cloned into the result.
    state
        .budget
        .charge("public", &value)
        .map_err(crate::stores::validation)?;
    state
        .budget
        .charge("host", host)
        .map_err(crate::stores::validation)?;
    state.merge(&mut value, host, &policy.model, patch, 0)?;
    stackpress_framework_schema::check_json(&value).map_err(crate::stores::validation)?;
    serde_json::from_value(value).map_err(crate::codec_error)
}
/// One assembly-wide checked graph, explicit context and cumulative retention budget.
struct Assembly<'a, 'context> {
    /// Native record metadata; private defaults have already become named host providers.
    policy: &'a RecordSchema,
    /// Borrow the same app-owned preparation context used by the subsequent store call.
    context: &'a ValueContext<'context>,
    /// Charge before inserting defaults; nested fields never receive independent budgets.
    budget: stackpress_framework_schema::PreparationBudget,
}
impl Assembly<'_, '_> {
    /// Preserve a declared fieldset default when adding private members to an omitted create field.
    fn initial(
        &mut self,
        field: &stackpress_framework_schema::Field,
        patch: bool,
    ) -> DataResult<Value> {
        if !patch {
            if let Some(default) = &field.default {
                self.budget
                    .charge(&field.name, default)
                    .map_err(crate::stores::validation)?;
                return Ok(default.clone());
            }
            if let Some(rule) = &field.value_rule {
                let value = rule
                    .resolve_for(&field.value_type, self.context)
                    .map_err(crate::stores::validation)?;
                self.budget
                    .charge(&field.name, &value)
                    .map_err(crate::stores::validation)?;
                return Ok(value);
            }
        }
        Ok(serde_json::json!({}))
    }
    /// Recurse through only the checked acyclic fieldset graph and bounded supplied JSON tree.
    fn merge(
        &mut self,
        value: &mut Value,
        host: &Value,
        model: &stackpress_framework_schema::Model,
        patch: bool,
        depth: usize,
    ) -> DataResult<()> {
        if depth > 32 {
            return Err(DataError::new("host assembly depth exceeded"));
        }
        let object = value
            .as_object_mut()
            .ok_or_else(|| DataError::new("assembled input must be an object"))?;
        let host = host
            .as_object()
            .ok_or_else(|| DataError::new("host fields must be an object"))?;
        // Host authority is limited to the checked graph; relations/generated/immutable IDs stay out.
        for (name, private) in host {
            let field = model
                .fields
                .iter()
                .find(|field| {
                    field.name == *name
                        && !field.generated
                        && (!patch || !field.key)
                        && !matches!(field.value_type, ValueType::Model(_))
                })
                .ok_or_else(|| DataError::new("host field is not writable"))?;
            // A public fieldset may receive private descendants without replacing public values.
            if field.access.writable {
                let ValueType::Fieldset(target) = &field.value_type else {
                    return Err(DataError::new("host fields cannot replace approved input"));
                };
                let nested = self
                    .policy
                    .fieldsets
                    .iter()
                    .find(|model| model.name == *target)
                    .ok_or_else(|| DataError::new("host fieldset missing"))?;
                // Preserve create defaults when private additions cause an omitted fieldset to exist.
                if !object.contains_key(name) {
                    object.insert(name.clone(), self.initial(field, patch)?);
                }
                let public = object
                    .get_mut(name)
                    .ok_or_else(|| DataError::new("host fieldset assembly failed"))?;
                // Per-index private additions require exact cardinality; no nested records are invented.
                if field.multiple {
                    let public = public.as_array_mut().ok_or_else(|| {
                        DataError::new("host list must match supplied input elements")
                    })?;
                    let private = private
                        .as_array()
                        .filter(|private| private.len() == public.len())
                        .ok_or_else(|| {
                            DataError::new("host list length differs from supplied input")
                        })?;
                    for (public, private) in public.iter_mut().zip(private) {
                        self.merge(public, private, nested, false, depth + 1)?;
                    }
                } else {
                    self.merge(public, private, nested, false, depth + 1)?;
                }
            } else {
                // DTO decoding has excluded private fields, so a supplied member cannot be replaced.
                if object.contains_key(name) {
                    return Err(DataError::new("private field unexpectedly supplied"));
                }
                object.insert(name.clone(), private.clone());
            }
        }
        Ok(())
    }
}
