//! Authorized finite CSV boundary; generated contracts/stores/atomic operations own data semantics.
use super::*;
use stackpress_framework_admin::transfer::{self, Preview, Upload};
use stackpress_framework_sql::{DataSession, operations};

/// Export all filtered public matches in selected order, deliberately ignoring page selection.
/// # Errors
/// Refuses authorization, invalid scope/state, incomplete projection or any output cap excess.
pub async fn export(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let config = invocation.service::<Config>()?;
    let (module, name) = route(request, config, false)?;
    if name != "export" {
        return Err(Error::invalid("export route unavailable"));
    }
    let policy = invocation.service::<Policy>()?;
    authorize_action(policy, model(module)?, Action::Export, request)?;
    let descriptor = spec(module)?;
    let values = parameters(request)?;
    if values.contains_key("key") {
        return Err(Error::invalid("unused export identity"));
    }
    let scope = parent(request, &invocation, &descriptor, &values).await?;
    let mut query = values
        .get("state")
        .map(|raw| serde_json::from_str::<SearchQuery>(raw).map_err(invalid_cause))
        .transpose()?
        .unwrap_or_default();
    // Validate the incoming full URL state before overriding only pagination for the export.
    query.operation(&descriptor.model).map_err(invalid_cause)?;
    query.skip = 0;
    query.limit = config.transfer_limits.rows as u16;
    if let Some(scope) = scope {
        if query
            .filters
            .get(&scope.field)
            .is_some_and(|value| !value.is_null() && value != &scope.value)
        {
            return Err(Error::invalid("related export parent mismatch"));
        }
        query.filter(&scope.field, scope.value);
    }
    let input = pages::query(
        module,
        query.operation(&descriptor.model).map_err(invalid_cause)?,
    )
    .map_err(invalid_cause)?;
    let result = call(request, &invocation, module, "search", input)
        .await
        .map_err(|error| {
            // The generated operation also bounds its aggregate public result before CSV encoding.
            // Preserve that typed cause, while explaining how to reduce an oversized export.
            if error.kind() == stackpress_lib::ErrorKind::Limit {
                Error::limit("export search result limit exceeded")
                    .with_public_message("Export exceeds byte limit; narrow the filter")
                    .with_cause(error)
            } else {
                error
            }
        })?;
    let total = result
        .get("total")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::invalid("export total unavailable"))?;
    if total > config.transfer_limits.rows as u64 {
        return Err(Error::invalid("export row limit exceeded")
            .with_public_message("Export exceeds row limit; narrow the filter"));
    }
    let rows = result
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::invalid("export rows unavailable"))?;
    if rows.len() as u64 != total {
        return Err(Error::invalid("incomplete export result")
            .with_public_message("Export could not include every result; narrow the filter"));
    }
    let csv =
        transfer::export(&descriptor.model, rows, config.transfer_limits).map_err(csv_error)?;
    response.set("text/csv; charset=utf-8", Body::Text(csv))?;
    response.headers_mut()?.set(
        "content-disposition",
        &format!("attachment; filename=\"{module}-export.csv\""),
    )?;
    response.headers_mut()?.set("cache-control", "no-store")?;
    Ok(Flow::Continue)
}

/// Preview or explicitly confirm a create-only import; no upload survives the request on the host.
/// # Errors
/// Policy/CSRF precede body decoding; any invalid row prevents calling the actual batch handler.
pub async fn import(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let config = invocation.service::<Config>()?;
    let (module, name) = route(request, config, true)?;
    if !matches!(name, "preview" | "confirm") {
        return Err(Error::invalid("import route unavailable"));
    }
    let policy = invocation.service::<Policy>()?;
    authorize_action(policy, model(module)?, Action::Import, request)?;
    if request.headers.get("origin") != Some(invocation.service::<MutationOrigin>()?.0.as_str())
        || request.headers.get("x-admin-csrf") != Some(config.csrf.as_str())
    {
        return Err(Error::new("admin import CSRF refused")
            .with_code(403)
            .with_public_message("operation is forbidden"));
    }
    if name == "confirm" && request.headers.get("x-admin-confirmed") != Some("true") {
        return Err(Error::invalid("explicit import confirmation required"));
    }
    let descriptor = spec(module)?;
    let values = parameters(request)?;
    if values.keys().any(|name| name != "parent") {
        return Err(Error::invalid("unused import parameter"));
    }
    let scope = parent(request, &invocation, &descriptor, &values).await?;
    let upload = upload(request, config.transfer_limits)?;
    let mut preview =
        transfer::preview(&descriptor.model, &upload, config.transfer_limits).map_err(csv_error)?;
    let host = policy
        .0
        .values(model(module)?, Operation::Batch, request)
        .map_err(operations::host_error)?;
    let context = host.context();
    let engine = invocation.service::<stackpress_sql::Engine>()?;
    let mut db = DataSession::pooled(engine);
    check_rows(
        module,
        &descriptor,
        &mut preview,
        scope.as_ref(),
        &context,
        &mut db,
    )
    .await?;
    if name == "preview" || !preview.valid() {
        return preview_reply(response, preview, name == "confirm");
    }
    // Revalidation and conflict checks above are repeated on every explicit confirm.
    // The generated batch preflights every plan before BEGIN and owns the one transaction.
    let count = preview.rows.len();
    let items = preview
        .rows
        .into_iter()
        .map(|row| json!({"mode":"create","input":row.values}))
        .collect::<Vec<_>>();
    let result = call(
        request,
        &invocation,
        module,
        "batch",
        json!({"items":items}),
    )
    .await?;
    if result.as_array().is_none_or(|rows| rows.len() != count) {
        return Err(Error::new("import acknowledgement invalid")
            .with_code(500)
            .with_public_message("Write outcome is unknown")
            .with_errors(json!({"outcome":"unknown"})));
    }
    response.json(&json!({"code":200,"results":{"imported":count}}))?;
    response.headers_mut()?.set("cache-control", "no-store")?;
    Ok(Flow::Continue)
}

/// Validate all rows and identities without writing. A rule error must not hide an identity conflict.
async fn check_rows(
    module: &str,
    descriptor: &PageSpec,
    preview: &mut Preview,
    scope: Option<&stackpress_framework_admin::pages::ParentScope>,
    context: &stackpress_framework_schema::ValueContext<'_>,
    db: &mut DataSession<'_>,
) -> Result<()> {
    let mut identities = std::collections::BTreeSet::new();
    for row in &mut preview.rows {
        // Parent selection is authoritative even when its field was omitted from the file.
        if let Some(scope) = scope {
            if row
                .values
                .get(&scope.field)
                .is_some_and(|v| v != &scope.value)
            {
                row.errors.push(transfer::issue(
                    Some(&scope.field),
                    "parent",
                    "Related parent cannot be replaced",
                ));
            }
            row.values
                .as_object_mut()
                .expect("CSV object")
                .insert(scope.field.clone(), scope.value.clone());
        }
        let mut checked_contract = true;
        let issues = match pages::validate_import(module, row.values.clone(), context) {
            Ok(issues) => stackpress_framework_admin::presentation::submitted_issues(
                &descriptor.model,
                issues,
                &row.values,
            ),
            Err(_) => {
                checked_contract = false;
                vec![transfer::issue(
                    None,
                    "contract",
                    "Row does not match the generated create contract",
                )]
            }
        };
        row.errors.extend(issues);
        if !checked_contract {
            continue;
        }
        let Ok(key) = stackpress_framework_admin::pages::key(descriptor, &row.values) else {
            continue;
        };
        let repeated = !identities.insert(key.to_string());
        let existing = crate::generated::pages_host::import_exists(module, db, key)
            .await
            .map_err(operations::host_error)?;
        if repeated || existing {
            row.errors.push(transfer::issue(
                None,
                "conflict",
                "Identity already exists or is repeated in this file",
            ));
        }
    }
    Ok(())
}

/// The MIME is opaque to HTTP parsing. Admit raw file size before UTF-8 validation or copying.
fn upload(request: &Request, limits: transfer::Limits) -> Result<Upload> {
    let Body::Bytes(bytes) = request.body() else {
        return Err(Error::invalid("CSV upload required"));
    };
    if bytes.len() > limits.bytes {
        return Err(Error::invalid("CSV upload limit exceeded")
            .with_code(413)
            .with_public_message("CSV file exceeds byte limit"));
    }
    let guards = match request.headers.get("x-admin-decode-export-guards") {
        None | Some("0") => false,
        Some("1") => true,
        _ => return Err(Error::invalid("invalid export decoding option")),
    };
    let text = std::str::from_utf8(bytes).map_err(|cause| {
        Error::invalid("invalid CSV UTF-8")
            .with_public_message("CSV must be UTF-8")
            .with_cause(cause)
    })?;
    Ok(Upload {
        csv: text.into(),
        decode_export_guards: guards,
        confirm: false,
    })
}
/// Bounded preview serialization avoids turning a valid input budget into unbounded output.
fn preview_reply(response: &mut Response, preview: Preview, invalid: bool) -> Result<Flow> {
    let result = json!({"code":if invalid{400}else{200},"results":preview,"error":if invalid{Some("Import has errors; nothing was written")}else{None}});
    let mut writer = Limited { bytes: Vec::new() };
    serde_json::to_writer(&mut writer, &result).map_err(|cause| {
        Error::invalid("import preview output exceeded")
            .with_public_message("Preview is too large; use a smaller file")
            .with_cause(cause)
    })?;
    // Setting a body selects its normal success status; apply the rejection status afterwards.
    response.set("application/json", Body::Bytes(writer.bytes))?;
    response.status_code(if invalid { 400 } else { 200 }, None)?;
    response.headers_mut()?.set("cache-control", "no-store")?;
    Ok(Flow::Continue)
}
/// Public preview bytes have their own hard ceiling, independent of escaped input expansion.
struct Limited {
    /// Only charged bytes are retained; an oversized serialization is discarded in full.
    bytes: Vec<u8>,
}
impl std::io::Write for Limited {
    /// Charge before extending, so serialization cannot grow beyond the preview budget.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > 1_048_576usize.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("preview byte limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    /// The response owns an in-memory buffer with no external flush operation.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Only our static parser contexts are public; typed causes stay private under #11.
fn csv_error(error: stackpress_framework_schema::SchemaError) -> Error {
    Error::invalid("invalid CSV transfer")
        .with_public_message(error.to_string())
        .with_cause(error)
}
