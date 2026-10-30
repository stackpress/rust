# Send and consume native content

Use the native carrier when a message contains owned bytes or a reader rather than JSON text. The transport consumes each content slot once, and a stream result remains available until its caller reads or drops it.

## Keep a reader out of JSON

This local stream example supplies exact raw bytes through an owned reader. It explicitly consumes the output; no network or external process is involved.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::{
    Content, EmailConfig, EmailResult, MessageOutput, NativeFactory, NativeSendMailOptions, Service,
};
use std::{io::Cursor, sync::Arc};
use tokio::io::AsyncReadExt;

pub async fn native_round_trip() -> EmailResult<()> {
    let config: EmailConfig = serde_json::from_value(json!({"streamTransport": true}))?;
    let service = Service::new(config, Arc::new(NativeFactory::default()));
    let mut mail = NativeSendMailOptions::default();
    mail.content.insert(
        "raw".into(),
        Content::Reader(Box::pin(Cursor::new(b"raw\r\n".to_vec()))),
    );
    let mut info = service.send_native(mail).await?;
    assert!(serde_json::to_value(&info).is_err());
    if let Some(MessageOutput::Stream(mut reader)) = info.message.take() {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        assert_eq!(bytes, b"raw\r\n");
    }
    Ok(())
}
```

Serialization returns an error while retaining the live reader. Taking `message` moves ownership to the caller; `read_to_end` is an explicit choice to buffer that output. Nonbuffer stream transport defers native MIME preparation, including content errors, until reading; dropping it unpolled performs no detached preparation.

## Choose a content slot

`Content::Bytes(Vec<u8>)` supplies Buffer data; `Content::Reader(MailReader)` supplies a Send, owned, one-shot AsyncRead. `MailReader` is a pinned boxed reader with a static lifetime and no Clone or Sync requirement.

| Slot | Example role |
| --- | --- |
| `text`, `html`, `raw` | Replace that original field or its explicit `.content` alias |
| `attachments.0.content` | Replace the first attachment content |
| `alternatives.0.content` | Replace the first alternative content |

Avoid supplying both a node and its alias for the same content: the native capability check routes conflicting/unsupported slots to a complete provider or reports the capability error before consumption. A JSON-only custom Transport can handle ordinary fields through its default `send_native`; native content requires an explicit implementation.

## Cross a server event boundary

With `server`, rich fields carry bytes or a `SharedNative` one-shot reader. The same carrier is returned at `results.message` for a stream output.

<!-- example: server -->
```rust
use stackpress_framework_email::{EmailResult, events::NativeReader};
use stackpress_lib::{Request, Result, SharedNative, Value};
use std::io::Cursor;
use tokio::io::AsyncReadExt;

pub async fn consume_reader(reader: &NativeReader) -> EmailResult<Vec<u8>> {
    let mut owned = reader.take()?;
    let mut bytes = Vec::new();
    owned.read_to_end(&mut bytes).await?;
    Ok(bytes)
}

pub fn request_with_reader() -> Result<Request> {
    let mut request = Request::new();
    let reader = NativeReader::new(Box::pin(Cursor::new(b"event raw".to_vec())));
    request
        .data
        .set("raw", Value::Shared(SharedNative::new(reader)))?;
    Ok(request)
}
```

`consume_reader` uses `NativeReader::take` to transfer the reader once, releasing its short lock before IO. Rich snapshot clones share that transfer slot, not a duplicated cursor; JSON conversion rejects a live native reader. Unknown native objects remain ordinary conversion errors instead of being silently discarded. See [event reference](../api/events.md) for response result shapes and error handling.
