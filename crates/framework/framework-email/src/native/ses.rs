//! Source SES command, region and result mapping.

use super::{NativeTransport, unsupported};
use crate::{EmailResult, NativeSendMailOptions, SentMessageInfo};
use crate::{
    backend::SesRequest,
    compose::{MessageMetadata, addresses, mime},
    message::buffer_json,
    options::string,
};
use serde_json::{Value, json};

/// Execute one selected adapter after the dispatcher has resolved message metadata.
pub(super) async fn send(
    transport: &mut NativeTransport,
    mail: NativeSendMailOptions,
    meta: MessageMetadata,
) -> EmailResult<SentMessageInfo> {
    // Preserve source message.ses overrides for the final command-object merge.
    let overrides = mail
        .fields
        .get("ses")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let from = match mail.fields.get("from") {
        Some(value) => json!(
            addresses(value)?
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => meta.envelope.get("from").cloned().unwrap_or(Value::Null),
    };
    // Compile before region lookup/send, then apply overrides after generated command fields.
    let bytes = mime(mail, &meta, &transport.factory.resolver, false).await?;
    let mut input = json!({"Content":{"Raw":{"Data":buffer_json(&bytes)}},"FromEmailAddress":from,"Destination":{"ToAddresses":meta.envelope["to"]}});
    if let Some(fields) = input.as_object_mut() {
        fields.extend(overrides);
    }
    let client = transport
        .factory
        .ses
        .as_ref()
        .ok_or_else(|| unsupported("SES native client"))?;
    // Source uses us-east-1 for failed, missing or empty regions without failing the send.
    let region = client
        .region()
        .await
        .ok()
        .flatten()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "us-east-1".into());
    let response = client
        .send(SesRequest {
            input,
            raw: bytes.clone(),
        })
        .await?;
    // Keep the provider's response value while formatting only the public Message-ID.
    let id = response.get("MessageId");
    let id_text = id.map(string).unwrap_or_else(|| "undefined".into());
    let region = if region == "us-east-1" {
        "email"
    } else {
        &region
    };
    let id_text = if id_text.contains('@') {
        format!("<{id_text}>")
    } else {
        format!("<{id_text}@{region}.amazonses.com>")
    };
    let mut result = SentMessageInfo::from(json!({"envelope":meta.envelope,"messageId":id_text}));
    if let Some(id) = id {
        result.fields["response"] = id.clone();
    }
    result.raw = Some(bytes);
    Ok(result)
}
