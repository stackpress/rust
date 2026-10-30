//! Source sendmail argument, process and exit-result mapping.

use super::NativeTransport;
use crate::{EmailResult, NativeSendMailOptions, SentMessageInfo};
use crate::{
    backend::{SendmailRequest, envelope_recipients},
    compose::{MessageMetadata, mime},
    options::{string, truthy},
};
use serde_json::{Value, json};
use std::io;

/// Execute one selected adapter after the dispatcher has resolved message metadata.
pub(super) async fn send(
    transport: &mut NativeTransport,
    mail: NativeSendMailOptions,
    meta: MessageMetadata,
) -> EmailResult<SentMessageInfo> {
    let from = meta.envelope.get("from").filter(|v| truthy(v)).map(string);
    let to = envelope_recipients(&meta.envelope);
    // This rejection is in the original sendmail adapter, not new hardening.
    if from.iter().chain(to.iter()).any(|v| v.starts_with('-')) {
        return Err(io::Error::other("Can not send mail. Invalid envelope addresses.").into());
    }
    // An explicit array, even empty, suppresses the source's default sender arguments.
    let mut args = vec!["-i".into()];
    if let Some(explicit) = transport
        .options
        .fields
        .get("args")
        .and_then(Value::as_array)
    {
        args.extend(explicit.iter().map(string));
    } else if let Some(from) = from {
        args.extend(["-f".into(), from]);
    }
    args.extend(to);
    // Resolve the executable before compiling bytes; process IO follows compilation.
    let program = transport
        .options
        .fields
        .get("path")
        .filter(|v| truthy(v))
        .map(string)
        .unwrap_or_else(|| "sendmail".into());
    let bytes = mime(mail, &meta, &transport.factory.resolver, true).await?;
    // Source treats signal termination like a zero exit; preserve its named 127 error.
    match transport.factory.process.run(SendmailRequest{program,args,message:bytes}).await? {
        None|Some(0)=>Ok(json!({"envelope":meta.envelope,"messageId":meta.message_id,"response":"Messages queued for delivery"}).into()),
        Some(127)=>Err(io::Error::other("Sendmail command not found, process exited with code 127").into()),
        Some(code)=>Err(io::Error::other(format!("Sendmail exited with code {code}")).into()),
    }
}
