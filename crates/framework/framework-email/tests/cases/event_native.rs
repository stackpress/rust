//! Reader transfer and cancellation through real event dispatch; all IO is in memory.

use serde_json::json;
use stackpress_framework_email::{
    NativeFactory,
    events::{EmailSend, NativeReader},
};
use stackpress_lib::{Body, Request, Response, SharedNative, Value};
use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

/// A deliberately non-Sync reader, proving the bridge imposes only Send ownership.
struct Reader {
    bytes: std::io::Cursor<Vec<u8>>,
    touched: std::cell::Cell<bool>,
    dropped: Arc<AtomicUsize>,
}
impl AsyncRead for Reader {
    /// Forward an in-memory source cursor and record entry without a shared IO lock.
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.touched.set(true);
        Pin::new(&mut self.bytes).poll_read(cx, buf)
    }
}
impl Drop for Reader {
    /// Track actual reader destruction, including an unread skipped request.
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

/// Source Readable input/output remains native; snapshots alias a one-shot transfer only.
#[tokio::test]
async fn native_reader_and_buffer_fields_survive_event_results() {
    let dropped = Arc::new(AtomicUsize::new(0));
    let native = SharedNative::new(NativeReader::new(Box::pin(Reader {
        bytes: std::io::Cursor::new(b"source\r\n".to_vec()),
        touched: std::cell::Cell::new(false),
        dropped: dropped.clone(),
    })));
    let mut request = Request::new();
    request
        .data
        .set("raw", Value::Shared(native.clone()))
        .unwrap();
    let event = EmailSend::new(
        Some(json!({"streamTransport":true})),
        Arc::new(NativeFactory::default()),
    );
    let mut response = Response::new();
    event.handle(&mut request, &mut response).await.unwrap();
    assert!(
        native
            .downcast_ref::<NativeReader>()
            .unwrap()
            .take()
            .is_err()
    );
    assert!(response.try_to_json().is_err());
    let Value::Object(fields) = response.result().unwrap() else {
        panic!("object info")
    };
    let reader = fields["message"]
        .as_shared()
        .unwrap()
        .downcast_ref::<NativeReader>()
        .unwrap();
    let mut owned = reader.take().unwrap();
    let mut bytes = vec![];
    owned.read_to_end(&mut bytes).await.unwrap();
    assert_eq!(bytes, b"source\r\n");
    assert!(reader.take().is_err());
    drop(owned);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    // Byte content at nested attachment/alternative paths follows the same native slot mapping.
    let mut request=Request::builder().data(json!({"text":"hello", "attachments":[{"filename":"a.bin"}], "alternatives":[{"contentType":"text/custom"}]})).build().unwrap();
    request
        .data
        .set_path("attachments.0.content", Value::Bytes(vec![0, 255]))
        .unwrap();
    request
        .data
        .set_path(
            "alternatives.0.content",
            Value::Bytes(b"alternative".to_vec()),
        )
        .unwrap();
    let event = EmailSend::new(
        Some(json!({"jsonTransport":true,"skipEncoding":true})),
        Arc::new(NativeFactory::default()),
    );
    let mut response = Response::new();
    event.handle(&mut request, &mut response).await.unwrap();
    let result = response.result().unwrap().to_json().unwrap();
    assert_eq!(
        result["message"]["attachments"][0]["content"],
        json!("AP8=")
    );
    assert_eq!(
        result["message"]["alternatives"][0]["content"],
        json!("YWx0ZXJuYXRpdmU=")
    );
}

/// Falsy bodies permit sends, but empty Buffer is truthy; all skips precede native transfer.
#[tokio::test]
async fn empty_buffer_body_and_missing_config_skip_without_consuming_reader() {
    for config in [None, Some(json!({"streamTransport":true}))] {
        let reader = SharedNative::new(NativeReader::new(Box::pin(std::io::Cursor::new(vec![
            1, 2, 3,
        ]))));
        let mut request = Request::new();
        request
            .data
            .set("raw", Value::Shared(reader.clone()))
            .unwrap();
        let mut response = Response::new();
        if config.is_some() {
            response
                .set("application/octet-stream", Body::Bytes(vec![]))
                .unwrap();
        }
        EmailSend::new(config, Arc::new(NativeFactory::default()))
            .handle(&mut request, &mut response)
            .await
            .unwrap();
        assert!(
            reader
                .downcast_ref::<NativeReader>()
                .unwrap()
                .take()
                .is_ok()
        );
    }
}

/// Unrepresentable rich input fails explicitly; it never becomes a silently missing field.
#[tokio::test]
async fn unknown_native_input_and_consumed_reader_fail_without_delivery() {
    let event = EmailSend::new(
        Some(json!({"jsonTransport":true})),
        Arc::new(NativeFactory::default()),
    );
    let mut request = Request::new();
    request
        .data
        .set("extension", Value::Shared(SharedNative::new(7u8)))
        .unwrap();
    assert!(
        event
            .handle(&mut request, &mut Response::new())
            .await
            .is_err()
    );
    let reader = SharedNative::new(NativeReader::new(Box::pin(std::io::Cursor::new(vec![1]))));
    drop(
        reader
            .downcast_ref::<NativeReader>()
            .unwrap()
            .take()
            .unwrap(),
    );
    let mut request = Request::new();
    request.data.set("text", Value::Shared(reader)).unwrap();
    assert!(
        event
            .handle(&mut request, &mut Response::new())
            .await
            .is_err()
    );
}

/// Existing caller-selected budgets remain effective through both event conversion directions.
#[tokio::test]
async fn event_preserves_deliberately_larger_request_and_response_budgets() {
    use stackpress_lib::ParseLimits;
    let text = "x".repeat(ParseLimits::default().max_bytes + 1);
    let limits = ParseLimits {
        max_bytes: 3 * 1024 * 1024,
        ..ParseLimits::default()
    };
    for config in [
        json!({"streamTransport":true}),
        json!({"jsonTransport":true,"skipEncoding":true}),
    ] {
        let mut request=Request::builder().limits(limits).data(json!({"raw":"fixture", "envelope":{"from":"sender@example.test", "to":[], "extension":text}})).build().unwrap();
        let mut response = Response::with_limits(limits);
        EmailSend::new(Some(config), Arc::new(NativeFactory::default()))
            .handle(&mut request, &mut response)
            .await
            .unwrap();
        assert!(response.status().is_some_and(|status| status.code == 200));
        let result = response.result().unwrap().as_object().unwrap();
        if let Some(message) = result.get("message").and_then(Value::as_shared) {
            // Stream metadata, including large source envelope extensions, stays intact.
            assert_eq!(
                result["envelope"].as_object().unwrap()["extension"]
                    .as_str()
                    .unwrap()
                    .len(),
                text.len()
            );
            drop(
                message
                    .downcast_ref::<NativeReader>()
                    .unwrap()
                    .take()
                    .unwrap(),
            );
        }
    }
}
