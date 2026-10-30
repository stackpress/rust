//! Delegated server events: source skip rules, supplied argv and explicit host IO.

use super::super::config::{Environment, ServerOptions, path, string};
use super::Bindings;
use serde_json::{Value, json};
use stackpress_lib::{
    Flow, Invocation, PublicError, Request, Response, Result, data::parse_arg_string,
};
use stackpress_server::ServerInvocationExt;

/// Whether an earlier source handler supplied a non-OK status.
pub(crate) fn skipped(res: &Response) -> bool {
    res.status()
        .is_some_and(|status| status.code != 0 && status.code != 200)
}

/// Stage the original event's intentionally public configuration/input error.
pub(crate) fn error(res: &mut Response, message: &str) -> Result<Flow> {
    res.set_error(PublicError {
        code: 500,
        status: "Internal Server Error".into(),
        error: message.into(),
        errors: json!({}),
    })?;
    Ok(Flow::Continue)
}

/// Invoke a host-owned operation with owned option data, retaining provider flow/errors.
/// # Errors
/// Propagates the existing missing binding, request construction and native provider failures.
pub async fn invoke(
    name: &str,
    options: Value,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Prepared options are a separate native call, not mutations of the source command input.
    let bindings = inv.shared_service::<Bindings>()?;
    let mut request = Request::builder().data(options).build()?;
    bindings.invoke(name, &mut request, res, inv).await
}

/// Source serve: skip a prior error; default host/port; execute only an explicit host binding.
/// # Errors
/// Retains typed native option and host-provider errors. Registration never listens.
pub async fn serve(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if skipped(res) {
        return Ok(Flow::Continue);
    }
    let config = &inv.config::<ServerOptions>()?.0;
    let host = string(path(config, "host"), "127.0.0.1")?;
    let port = path(config, "port").cloned().unwrap_or(json!(3000));
    let flow = invoke("serve", json!({"host":host,"port":port}), res, inv).await?;
    res.status_code(200, None)?;
    Ok(flow)
}

/// Source develop deliberately runs even after an earlier non-OK status.
/// # Errors
/// Propagates supervision-provider errors; this library starts no watcher or child implicitly.
pub async fn develop(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let config = &inv.config::<ServerOptions>()?.0;
    let configured = path(config, "develop").cloned().unwrap_or(json!({}));
    let env = inv.service::<Environment>().cloned().unwrap_or_default();
    // Nullish source defaults preserve false/zero/empty arrays where supplied.
    let option = |key: &str, default: Value| {
        configured
            .get(key)
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or(default)
    };
    let flow = invoke("develop",json!({"argv":env.argv,"cwd":env.cwd,"debounce":option("debounce",json!(100)),"extensions":option("extensions",json!([".ts"])),"ignore":option("ignore",json!([])),"timeout":option("timeout",json!(10000)),"watch":option("watch",json!(["."]))}),res,inv).await?;
    res.status_code(200, None)?;
    Ok(flow)
}

/// Source emit checks `argv[3]`, reparses its joined tail, then prints the raw nested response.
/// # Errors
/// Preserves parser, nested-event and explicit output-provider errors.
pub async fn emit(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if skipped(res) {
        return Ok(Flow::Continue);
    }
    let env = inv.service::<Environment>().cloned().unwrap_or_default();
    if env.argv.get(3).is_none() {
        return error(res, "Missing event name");
    }
    emit_with_skip(&env.argv, 3, res, inv).await?;
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}

/// Direct source emit script, whose default skip is two rather than the event wrapper's three.
/// # Errors
/// Preserves argument parsing, nested dispatch and explicit output-provider failures.
pub async fn emit_script(argv: &[String], res: &mut Response, inv: Invocation<'_>) -> Result<()> {
    emit_with_skip(argv, 2, res, inv).await
}

/// Direct source emit script with an explicit argv prefix length.
/// # Errors
/// Preserves parser/nested/provider errors; it does not stage an event-wrapper OK status.
pub async fn emit_with_skip(
    argv: &[String],
    skip: usize,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<()> {
    let event = argv.get(skip).map(String::as_str).unwrap_or_default();
    // Upstream Terminal joins arguments before its string parser; preserve that boundary.
    let data = parse_arg_string(
        &argv
            .get(skip.saturating_add(1)..)
            .unwrap_or_default()
            .join(" "),
        &Default::default(),
    )?;
    let mut child = Request::builder().data(data).build()?;
    let mut output = Response::new();
    let _ = inv.emit(event, &mut child, &mut output).await?;
    invoke(
        "output",
        json!({"text":serde_json::to_string_pretty(&output.try_to_json()?)?}),
        res,
        inv,
    )
    .await?;
    Ok(())
}
