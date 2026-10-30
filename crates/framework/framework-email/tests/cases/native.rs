//! Differential transport fixtures execute actual adapters with inert provider/process seams.

use serde_json::{Map, Value, json};
use stackpress_framework_email::{
    Content, EmailConfig, EmailResult, MailFuture, MessageOutput, NativeFactory,
    NativeSendMailOptions, SendMailOptions, SentMessageInfo, Service, Transport, TransportFactory,
};
use stackpress_framework_email::{
    backend::{
        NativeProcessBackend, ProcessBackend, ProviderFuture, SendmailRequest, SesClient,
        SesRequest, SmtpBackend, SmtpRequest,
    },
    options::TransportOptions,
};
use std::{
    io,
    sync::{Arc, Mutex},
};
use tokio::io::AsyncReadExt;

/// Immutable original-source outputs, generated with no live provider calls.
fn vectors() -> Value {
    serde_json::from_str(include_str!("../fixtures/source-transports.json")).unwrap()
}
/// Decode the source object/string union used by direct calls.
fn config(value: &Value) -> EmailConfig {
    serde_json::from_value(value.clone()).unwrap()
}
/// Borrow source-shaped message fields without introducing defaults in the fixture.
fn fields(value: &Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// Constructor selection, service/URL precedence, truthiness and pool defaults match source.
#[test]
fn source_constructor_option_vectors() {
    for case in vectors()["selection"].as_array().unwrap() {
        let resolved = TransportOptions::resolve(&config(&case["config"])).unwrap();
        let expected_kind = if case["kind"] == "SesTransport" {
            json!("Ses")
        } else {
            case["kind"].clone()
        };
        assert_eq!(
            serde_json::to_value(resolved.kind).unwrap(),
            expected_kind,
            "{}",
            case["config"]
        );
        assert_eq!(json!(resolved.fields), case["fields"], "{}", case["config"]);
        if !case["smtp"].is_null() {
            let smtp = resolved.smtp();
            assert_eq!(smtp.host, case["smtp"]["host"].as_str().unwrap());
            assert_eq!(smtp.port, case["smtp"]["port"].as_f64().unwrap());
            assert_eq!(smtp.secure, case["smtp"]["secure"].as_bool().unwrap());
            assert_eq!(smtp.secured, case["smtp"]["secured"].as_bool().unwrap());
        }
        if !case["rate"].is_null() {
            let rate = resolved.pool_rate();
            assert_eq!(rate.0, case["rate"][0].as_f64().unwrap());
            assert_eq!(rate.1, case["rate"][1].as_f64().unwrap());
        }
    }
}

/// Local JSON transport normalization/info is compared to the original implementation.
#[tokio::test]
async fn source_json_normalization_vectors() {
    for case in vectors()["jsonCases"].as_array().unwrap() {
        let service = Service::new(config(&case["config"]), Arc::new(NativeFactory::default()));
        let result = service.send(fields(&case["options"])).await.unwrap();
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            case["result"],
            "{} / {}",
            case["config"],
            case["options"]
        );
    }
}

/// Source buffer/readable results preserve exact raw bytes and expose native stream ownership.
#[tokio::test]
async fn source_stream_raw_buffer_and_reader_vectors() {
    for case in vectors()["streamCases"].as_array().unwrap() {
        let service = Service::new(config(&case["config"]), Arc::new(NativeFactory::default()));
        let mut result = service.send(fields(&case["options"])).await.unwrap();
        result.message = Some(match result.message.take().unwrap() {
            MessageOutput::Stream(mut reader) => {
                let mut bytes = Vec::new();
                reader.read_to_end(&mut bytes).await.unwrap();
                MessageOutput::Buffer(bytes)
            }
            buffer => buffer,
        });
        assert_eq!(serde_json::to_value(result).unwrap(), case["result"]);
    }
}

/// SMTP provider captures complete source settings/envelope/bytes and returns unfiltered info.
struct FakeSmtp {
    requests: Mutex<Vec<SmtpRequest>>,
    info: Value,
}
impl SmtpBackend for FakeSmtp {
    /// No connection occurs; the production adapter still owns compilation/result projection.
    fn send(&self, request: SmtpRequest) -> ProviderFuture<'_, SentMessageInfo> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            Ok(self.info.clone().into())
        })
    }
}

/// Source SMTP constructor/driver/result boundaries match the original fake-connection oracle.
#[tokio::test]
async fn source_smtp_fake_sdk_vectors() {
    for case in vectors()["smtpCases"].as_array().unwrap() {
        let backend = Arc::new(FakeSmtp {
            requests: Mutex::new(vec![]),
            info: json!({"accepted":["recipient@example.test"],"rejected":[],"response":"250 fixture","envelopeTime":1,"messageTime":2,"messageSize":3}),
        });
        let service = Service::new(
            config(&case["config"]),
            Arc::new(NativeFactory::default().with_smtp(backend.clone())),
        );
        let result = service.send(fields(&case["options"])).await.unwrap();
        assert_eq!(serde_json::to_value(result).unwrap(), case["result"]);
        let requests = backend.requests.lock().unwrap();
        let request = &requests[0];
        assert_eq!(json!(request.options.fields), case["captured"]["fields"]);
        assert_eq!(request.envelope, case["captured"]["envelope"]);
        assert_eq!(
            json!({"type":"Buffer","data":request.message}),
            case["captured"]["message"]
        );
    }
}

/// Every sendmail execution is inert and records the actual adapter's process invocation.
struct FakeProcess {
    requests: Mutex<Vec<SendmailRequest>>,
    code: i32,
}
impl ProcessBackend for FakeProcess {
    /// Preserve the source exit-code branch without invoking an executable.
    fn run(&self, request: SendmailRequest) -> ProviderFuture<'_, Option<i32>> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            Ok(Some(self.code))
        })
    }
}

/// Explicit/default arguments, raw Bcc-capable bytes and 0/127/other exit results match source.
#[tokio::test]
async fn source_sendmail_fake_process_vectors() {
    for case in vectors()["sendmailCases"].as_array().unwrap() {
        let code = if case["error"].is_null() {
            0
        } else if case["error"].as_str().unwrap().contains("127") {
            127
        } else {
            3
        };
        let process = Arc::new(FakeProcess {
            requests: Mutex::new(vec![]),
            code,
        });
        let service = Service::new(
            config(&case["config"]),
            Arc::new(NativeFactory::default().with_process(process.clone())),
        );
        match service.send(fields(&case["options"])).await {
            Ok(result) => assert_eq!(serde_json::to_value(result).unwrap(), case["result"]),
            Err(error) => assert_eq!(error.to_string(), case["error"].as_str().unwrap()),
        }
        let requests = process.requests.lock().unwrap();
        let request = &requests[0];
        assert_eq!(
            request.program,
            case["invocation"]["program"].as_str().unwrap()
        );
        assert_eq!(json!(request.args), case["invocation"]["args"]);
        assert_eq!(
            json!({"type":"Buffer","data":request.message}),
            case["invocation"]["message"]
        );
    }
}

/// Native SES callback captures full command input, including message.ses overrides.
struct FakeSes {
    requests: Mutex<Vec<SesRequest>>,
    region: Option<String>,
    id: String,
}
impl SesClient for FakeSes {
    /// Source region failure falls back without preventing the send callback.
    fn region(&self) -> ProviderFuture<'_, Option<String>> {
        Box::pin(async move {
            match &self.region {
                Some(value) => Ok(Some(value.clone())),
                None => Err(io::Error::other("region failed").into()),
            }
        })
    }
    /// Return source-like provider data, retaining its id for result mapping.
    fn send(&self, request: SesRequest) -> ProviderFuture<'_, Value> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            Ok(json!({"MessageId":self.id}))
        })
    }
}

/// SES region/id mapping, source overrides and raw result bytes match original fake-SDK traces.
#[tokio::test]
async fn source_ses_fake_sdk_vectors() {
    for case in vectors()["sesCases"].as_array().unwrap() {
        let client = Arc::new(FakeSes {
            requests: Mutex::new(vec![]),
            region: case["region"].as_str().map(str::to_owned),
            id: case["id"].as_str().unwrap().into(),
        });
        let service = Service::new(
            config(&json!({"SES":{}})),
            Arc::new(NativeFactory::default().with_ses(client.clone())),
        );
        let result = service.send(fields(&case["options"])).await.unwrap();
        assert_eq!(serde_json::to_value(result).unwrap(), case["result"]);
        let requests = client.requests.lock().unwrap();
        assert_eq!(requests[0].input, case["input"]);
    }
}

/// Full provider owns unsupported source options and native readers without prior consumption.
struct FullProvider;
struct FullTransport {
    config: EmailConfig,
}
impl TransportFactory for FullProvider {
    /// Reflect original source config, not a narrowed lettre DTO.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        Ok(Box::new(FullTransport {
            config: config.clone(),
        }))
    }
}

/// JSON-only custom transports accept ordinary native calls and reject owned content explicitly.
#[tokio::test]
async fn legacy_transport_native_content_contract_and_compiler_binding() {
    struct JsonOnly;
    impl Transport for JsonOnly {
        /// Existing custom implementations remain usable without adopting reader support.
        fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
            Box::pin(async move { Ok(json!({"options":options}).into()) })
        }
    }
    let mut transport = JsonOnly;
    let info = transport
        .send_native(NativeSendMailOptions::default())
        .await
        .unwrap();
    assert_eq!(info["options"], json!({}));
    let mut options = NativeSendMailOptions::default();
    options
        .content
        .insert("raw".into(), Content::Bytes(vec![1]));
    assert!(
        transport
            .send_native(options)
            .await
            .unwrap_err()
            .to_string()
            .contains("does not implement native content")
    );
    let conf = config(&json!({"jsonTransport":true}));
    let options =
        fields(&json!({"html":"<img src='data:image/png;base64,AQID'>","attachDataUrls":true}));
    let service = Service::new(conf.clone(), Arc::new(NativeFactory::default()));
    assert!(
        service
            .send(options.clone())
            .await
            .unwrap_err()
            .to_string()
            .contains("attachDataUrls")
    );
    let service = Service::new(
        conf,
        Arc::new(NativeFactory::default().with_provider(Arc::new(FullProvider))),
    );
    let result = service.send(options.clone()).await.unwrap();
    assert_eq!(result["options"], json!(options));
    let service = Service::new(
        config(&json!("legacy-relative-string")),
        Arc::new(NativeFactory::default().with_provider(Arc::new(FullProvider))),
    );
    let result = service.send(Default::default()).await.unwrap();
    assert_eq!(result["config"], "legacy-relative-string");
}

/// Generated SMTP bytes hide Bcc while retaining every envelope recipient and source info.
#[tokio::test]
async fn smtp_bcc_header_and_envelope_projection() {
    let smtp = Arc::new(FakeSmtp {
        requests: Mutex::new(vec![]),
        info: json!({"extension":17}),
    });
    let service = Service::new(
        config(&json!({"ignoreTLS":true})),
        Arc::new(NativeFactory::default().with_smtp(smtp.clone())),
    );
    let info=service.send(fields(&json!({"from":"sender@example.test","to":"to@example.test","bcc":"hidden@example.test","text":"hello"}))).await.unwrap();
    assert_eq!(info["extension"], 17);
    assert_eq!(
        info["envelope"]["to"],
        json!(["to@example.test", "hidden@example.test"])
    );
    let requests = smtp.requests.lock().unwrap();
    let bytes = String::from_utf8_lossy(&requests[0].message);
    assert!(!bytes.to_lowercase().contains("bcc:"));
    assert!(!bytes.contains("hidden@example.test"));
    assert!(bytes.contains("hello"));
}
impl Transport for FullTransport {
    /// Echo source fields to make lost extensions observable.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        Box::pin(async move { Ok(json!({"options":options,"config":self.config}).into()) })
    }
    /// Native callback consumes the owned payload only after the provider was selected.
    fn send_native(&mut self, options: NativeSendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            Ok(json!({"options":options.fields,"config":self.config,"nativeSlots":options.content.keys().collect::<Vec<_>>(),"extension":{"retained":true}}).into())
        })
    }
}

/// Missing support is an explicit error; bound providers receive all original option variants.
#[tokio::test]
async fn unsupported_capabilities_are_bound_or_reported() {
    for config_value in [
        json!({"lmtp":true,"extension":7}),
        json!({"pool":true,"maxMessages":2}),
        json!({"greetingTimeout":123}),
        json!({"auth":{"type":"custom","method":"fixture"}}),
        json!({"tls":{"ciphers":"fixture"}}),
    ] {
        let conf = config(&config_value);
        let unsupported = NativeFactory::default().create(&conf).err().unwrap();
        assert!(
            unsupported
                .to_string()
                .contains("native email provider required")
        );
        let service = Service::new(
            conf,
            Arc::new(NativeFactory::default().with_provider(Arc::new(FullProvider))),
        );
        let options = fields(&json!({"to":"to@example.test","extension":{"retained":true}}));
        let result = service.send(options.clone()).await.unwrap();
        assert_eq!(result["options"], json!(options));
        assert_eq!(result["config"], config_value);
    }
    let service = Service::new(
        config(&json!({"streamTransport":true})),
        Arc::new(NativeFactory::default().with_provider(Arc::new(FullProvider))),
    );
    let mut options = NativeSendMailOptions::from(fields(
        &json!({"dkim":{"selector":"fixture"},"to":"to@example.test"}),
    ));
    options
        .content
        .insert("text".into(), Content::Bytes(vec![1, 2, 3]));
    let result = service.send_native(options).await.unwrap();
    assert_eq!(result["nativeSlots"], json!(["text"]));
    assert_eq!(result["options"]["dkim"], json!({"selector":"fixture"}));
}

/// The native process driver is exercised with cat as a fake sink, never sendmail.
#[tokio::test]
async fn native_process_driver_closes_stdin_and_waits() {
    assert_eq!(
        NativeProcessBackend
            .run(SendmailRequest {
                program: "/bin/cat".into(),
                args: vec![],
                message: vec![b'x'; 4096]
            })
            .await
            .unwrap(),
        Some(0)
    );
    assert!(
        NativeProcessBackend
            .run(SendmailRequest {
                program: "/nonexistent/slice9b-fixture".into(),
                args: vec![],
                message: vec![]
            })
            .await
            .is_err()
    );
}

/// Collect native streams explicitly; serialization never consumes an owned reader.
async fn output_bytes(mut info: SentMessageInfo) -> Vec<u8> {
    match info.message.take().unwrap() {
        MessageOutput::Buffer(bytes) => bytes,
        MessageOutput::Stream(mut reader) => {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.unwrap();
            bytes
        }
    }
}

/// Original MIME results are compared by headers/tree roles; library folding/boundaries are native.
#[tokio::test]
async fn source_mime_semantics_and_option_choices() {
    for case in vectors()["mimeCases"].as_array().unwrap() {
        let service = Service::new(
            config(&json!({"streamTransport":true,"buffer":true})),
            Arc::new(NativeFactory::default().with_content(Arc::new(FakeContent::default()))),
        );
        let result = service.send(fields(&case["options"])).await.unwrap();
        assert_eq!(result["envelope"], case["envelope"]);
        assert_eq!(result["messageId"], case["messageId"]);
        let actual = String::from_utf8(output_bytes(result).await).unwrap();
        let original = case["message"].as_str().unwrap();
        for marker in [
            "text/html",
            "multipart/mixed",
            "multipart/alternative",
            "multipart/related",
            "Content-ID: <fixture>",
            "Content-Disposition: inline",
            "Content-Disposition: x-fixture",
            "Content-Transfer-Encoding: base64",
            "Content-Transfer-Encoding: quoted-printable",
            "Content-Transfer-Encoding: 8bit",
            "X-Priority: 1 (Highest)",
            "X-Priority: 5 (Lowest)",
            "Importance: High",
            "Importance: Low",
            "References: <one@example.test> <two@example.test>",
        ] {
            assert_eq!(
                actual.to_lowercase().contains(&marker.to_lowercase()),
                original.to_lowercase().contains(&marker.to_lowercase()),
                "marker {marker} / {}\n{actual}",
                case["options"]
            );
        }
        assert!(!actual.contains("X-Priority: bad"));
    }
}

/// An inert resolver captures complete source locations and headers before returning fixture bytes.
#[derive(Default)]
struct FakeContent {
    requests: Mutex<Vec<stackpress_framework_email::content::ContentRequest>>,
    fail: bool,
}
impl stackpress_framework_email::content::ContentResolver for FakeContent {
    /// Simulate both native IO success and the original asynchronous content error surface.
    fn read(
        &self,
        request: stackpress_framework_email::content::ContentRequest,
    ) -> stackpress_framework_email::content::ContentFuture<'_> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            if self.fail {
                Err(io::Error::other("fixture content failed").into())
            } else {
                Ok(b"fixture content".to_vec())
            }
        })
    }
}

/// MIME access flags apply before IO; JSON keeps the source normalization bypass.
#[tokio::test]
async fn content_locations_access_flags_and_transport_precedence() {
    for (path, flag) in [
        ("/fixture/file.txt", "disableFileAccess"),
        ("HTTPS://fixture.test/file.txt", "disableUrlAccess"),
    ] {
        let resolver = Arc::new(FakeContent::default());
        let options = fields(
            &json!({"text":{"path":path,"content":"ignored MIME literal","httpHeaders":{"X-Fixture":"retained"}},flag:true}),
        );
        let service = Service::new(
            config(&json!({"streamTransport":true,"buffer":true})),
            Arc::new(NativeFactory::default().with_content(resolver.clone())),
        );
        assert!(
            service
                .send(options.clone())
                .await
                .unwrap_err()
                .to_string()
                .contains("access rejected")
        );
        assert!(resolver.requests.lock().unwrap().is_empty());
        // An explicit transport false overrides message true, as in MailMessage's constructor.
        let service = Service::new(
            config(&json!({"streamTransport":true,"buffer":true,flag:false})),
            Arc::new(NativeFactory::default().with_content(resolver.clone())),
        );
        let body =
            String::from_utf8(output_bytes(service.send(options).await.unwrap()).await).unwrap();
        assert!(body.contains("fixture content"));
        let requests = resolver.requests.lock().unwrap();
        assert_eq!(requests[0].location, path);
        assert_eq!(requests[0].options["httpHeaders"]["X-Fixture"], "retained");
    }
    let resolver = Arc::new(FakeContent::default());
    let service = Service::new(
        config(&json!({"jsonTransport":true,"skipEncoding":true,"disableFileAccess":true})),
        Arc::new(NativeFactory::default().with_content(resolver.clone())),
    );
    let info = service
        .send(fields(&json!({"text":{"path":"/fixture/file.txt"}})))
        .await
        .unwrap();
    assert_eq!(info["message"]["text"], "fixture content");
    assert_eq!(resolver.requests.lock().unwrap().len(), 1);
}

/// A source non-buffer stream returns before content errors; no detached task runs on drop.
#[tokio::test]
async fn deferred_stream_errors_and_native_slot_ownership() {
    let resolver = Arc::new(FakeContent {
        fail: true,
        ..Default::default()
    });
    let service = Service::new(
        config(&json!({"streamTransport":true})),
        Arc::new(NativeFactory::default().with_content(resolver.clone())),
    );
    let mut result = service
        .send(fields(&json!({"text":{"path":"/fixture/fails"}})))
        .await
        .unwrap();
    assert!(resolver.requests.lock().unwrap().is_empty());
    assert!(
        serde_json::to_value(&result)
            .unwrap_err()
            .to_string()
            .contains("explicit consumption")
    );
    if let Some(MessageOutput::Stream(reader)) = &mut result.message {
        assert!(
            reader
                .read_to_end(&mut Vec::new())
                .await
                .unwrap_err()
                .to_string()
                .contains("fixture content failed")
        );
    } else {
        panic!("stream expected");
    }
    assert_eq!(resolver.requests.lock().unwrap().len(), 1);
    let result = service
        .send(fields(&json!({"text":{"path":"/fixture/unread"}})))
        .await
        .unwrap();
    drop(result);
    assert_eq!(resolver.requests.lock().unwrap().len(), 1);
    let service = Service::new(
        config(&json!({"jsonTransport":true,"skipEncoding":true})),
        Arc::new(NativeFactory::default()),
    );
    let mut mail = NativeSendMailOptions::from(fields(
        &json!({"attachments":[{"content":"replaced","filename":"blob.bin"}]}),
    ));
    mail.content.insert(
        "text.content".into(),
        Content::Reader(Box::pin(std::io::Cursor::new(b"native text".to_vec()))),
    );
    mail.content
        .insert("attachments.0.content".into(), Content::Bytes(vec![0, 255]));
    let result = service.send_native(mail).await.unwrap();
    assert_eq!(result["message"]["text"], "native text");
    assert_eq!(result["message"]["attachments"][0]["content"], "AP8=");
    assert_eq!(result["message"]["attachments"][0]["encoding"], "base64");
    let mut mail = NativeSendMailOptions::default();
    mail.content
        .insert("unknown".into(), Content::Bytes(vec![]));
    assert!(
        service
            .send_native(mail)
            .await
            .unwrap_err()
            .to_string()
            .contains("native content slot unknown")
    );
}

/// Native decoding matches Buffer encodings, including permissive hex/base64/data URI branches.
#[tokio::test]
async fn buffer_encoding_and_data_uri_vectors() {
    let service = Service::new(
        config(&json!({"streamTransport":true,"buffer":true})),
        Arc::new(NativeFactory::default()),
    );
    for (encoding, text, expected) in [
        ("hex", "6162zz63", b"ab".as_slice()),
        ("base64url", " YW Jj!!", b"abc"),
        ("latin1", "é", &[233]),
        ("ucs2", "A", &[65, 0]),
        ("utf8", "abc", b"abc"),
    ] {
        let result = service
            .send(fields(&json!({"raw":{"content":text,"encoding":encoding}})))
            .await
            .unwrap();
        assert_eq!(output_bytes(result).await, expected);
    }
    assert!(
        service
            .send(fields(
                &json!({"raw":{"content":"x","encoding":"not-an-encoding"}})
            ))
            .await
            .unwrap_err()
            .to_string()
            .contains("content encoding")
    );
    let resolver = stackpress_framework_email::content::NativeContentResolver;
    use stackpress_framework_email::content::{ContentRequest, ContentResolver};
    for (uri, expected) in [
        ("data:text/plain;base64,YWJj", b"abc".as_slice()),
        ("data:,a%20b", b"a b"),
        ("data:,bad%ZZ", b"bad%ZZ"),
        ("data:missing-comma", b""),
    ] {
        assert_eq!(
            resolver
                .read(ContentRequest {
                    location: uri.into(),
                    options: Default::default(),
                    http_headers: true
                })
                .await
                .unwrap(),
            expected
        );
    }
    let path = std::env::temp_dir().join(format!("slice9b-content-{}", std::process::id()));
    std::fs::write(&path, b"native file").unwrap();
    let bytes = resolver
        .read(ContentRequest {
            location: path.to_string_lossy().into(),
            options: Default::default(),
            http_headers: true,
        })
        .await
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(bytes, b"native file");
}

/// Full native bindings receive every unsupported source knob, with no transport IO first.
#[tokio::test]
async fn complete_provider_config_and_option_routing() {
    for value in [
        json!({"lmtp":true}),
        json!({"socket":{"native":"fixture"}}),
        json!({"getSocket":"callback"}),
        json!({"authMethod":"CRAM-MD5"}),
        json!({"auth":{"type":"OAuth2","user":"u"}}),
        json!({"dnsTimeout":10}),
        json!({"socketTimeout":10}),
        json!({"pool":true,"maxConnections":3,"maxMessages":9}),
        json!({"localAddress":"unrepresentable fixture"}),
        json!({"tls":{"servername":42}}),
        json!({"connectionTimeout":-1}),
    ] {
        let mut value = value;
        value["extension"] = json!({"retained":true});
        let conf = config(&value);
        assert!(NativeFactory::default().create(&conf).is_err());
        let service = Service::new(
            conf,
            Arc::new(NativeFactory::default().with_provider(Arc::new(FullProvider))),
        );
        let result = service
            .send(fields(
                &json!({"text":"message","extension":{"message":true}}),
            ))
            .await
            .unwrap();
        assert_eq!(result["config"], value);
        assert_eq!(result["options"]["extension"], json!({"message":true}));
    }
    for extra in [
        json!({"dkim":{"selector":"fixture"}}),
        json!({"textEncoding":"base64","subject":"café"}),
        json!({"watchHtml":"watch"}),
        json!({"amp":"amp"}),
        json!({"icalEvent":"event"}),
        json!({"dsn":{"notify":"SUCCESS"}}),
        json!({"list":{"help":"help@example.test"}}),
        json!({"normalizeHeaderKey":"native-callback"}),
        json!({"attachDataUrls":true}),
        json!({"baseBoundary":"fixture"}),
        json!({"envelope":{"size":10}}),
        json!({"headers":{"Content-Type":"text/x-fixture"}}),
        json!({"alternatives":[{"content":"x","filename":"x.txt"}]}),
        json!({"attachments":[{"raw":"raw part"}]}),
    ] {
        let conf = config(&json!({"streamTransport":true,"buffer":true}));
        let options = fields(&extra);
        let service = Service::new(conf.clone(), Arc::new(NativeFactory::default()));
        assert!(
            service
                .send(options.clone())
                .await
                .unwrap_err()
                .to_string()
                .contains("native email provider required")
        );
        let service = Service::new(
            conf,
            Arc::new(NativeFactory::default().with_provider(Arc::new(FullProvider))),
        );
        let mut mail = NativeSendMailOptions::from(options.clone());
        mail.content
            .insert("text".into(), Content::Bytes(vec![1, 2, 3]));
        let result = service.send_native(mail).await.unwrap();
        assert_eq!(result["options"], json!(options));
        assert_eq!(result["nativeSlots"], json!(["text"]));
    }
}

/// Source address limits are provider-bound; native outputs cannot silently erase non-object info.
#[tokio::test]
async fn native_metadata_and_output_error_boundaries() {
    let service = Service::new(
        config(&json!({"streamTransport":true})),
        Arc::new(NativeFactory::default().with_provider(Arc::new(FullProvider))),
    );
    let options = fields(&json!({"from":"source local address","extension":7}));
    let result = service.send(options.clone()).await.unwrap();
    assert_eq!(result["options"], json!(options));
    let service = Service::new(
        config(&json!({"sendmail":true})),
        Arc::new(NativeFactory::default()),
    );
    assert_eq!(
        service
            .send(fields(
                &json!({"envelope":{"from":"-flag@example.test","to":[]}})
            ))
            .await
            .unwrap_err()
            .to_string(),
        "Can not send mail. Invalid envelope addresses."
    );
    let mut info = SentMessageInfo::from(json!(null));
    info.message = Some(MessageOutput::Buffer(vec![1]));
    assert!(
        serde_json::to_value(&info)
            .unwrap_err()
            .to_string()
            .contains("require an object")
    );
    let plain = SentMessageInfo::from(json!([1, 2, 3]));
    assert_eq!(serde_json::to_value(&plain).unwrap(), json!([1, 2, 3]));
    assert_eq!(
        format!("{plain:?}"),
        "SentMessageInfo { native_message: false, native_raw: false, .. }"
    );
    let meta = stackpress_framework_email::compose::metadata(&fields(
        &json!({"envelope":{"from":"sender@custom.test","to":[]}}),
    ))
    .unwrap();
    assert!(meta.message_id.ends_with("@custom.test>"));
}

/// Invoke through the production event, normalizing only an explicitly consumed native stream.
#[cfg(feature = "server")]
async fn event_info(config: Value, options: Value, factory: NativeFactory) -> EmailResult<Value> {
    use stackpress_framework_email::events::{EmailSend, NativeReader};
    use stackpress_lib::{Request, Response, Value as Rich};
    let event = EmailSend::new(Some(config), Arc::new(factory));
    let mut request = Request::builder().data(options).build()?;
    let mut response = Response::new();
    event.handle(&mut request, &mut response).await?;
    let mut result = response.result().unwrap().clone();
    if let Rich::Object(fields) = &mut result
        && let Some(Rich::Shared(shared)) = fields.get("message")
    {
        let mut reader = shared.downcast_ref::<NativeReader>().unwrap().take()?;
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        fields.insert(
            "message".into(),
            Rich::try_from(json!({"type":"Buffer","data":bytes}))?,
        );
    }
    Ok(result.to_json()?)
}

/// Every original transport remains reachable through the same email-send process.
#[cfg(feature = "server")]
#[tokio::test]
async fn event_reuses_all_source_transport_option_and_result_vectors() {
    for key in [
        "jsonCases",
        "streamCases",
        "smtpCases",
        "sendmailCases",
        "sesCases",
    ] {
        for case in vectors()[key].as_array().unwrap() {
            let smtp = Arc::new(FakeSmtp {
                requests: Mutex::new(vec![]),
                info: json!({"accepted":["recipient@example.test"],"rejected":[],"response":"250 fixture","envelopeTime":1,"messageTime":2,"messageSize":3}),
            });
            let code = if case["error"].is_null() {
                0
            } else if case["error"].as_str().unwrap().contains("127") {
                127
            } else {
                3
            };
            let process = Arc::new(FakeProcess {
                requests: Mutex::new(vec![]),
                code,
            });
            let ses = Arc::new(FakeSes {
                requests: Mutex::new(vec![]),
                region: case["region"].as_str().map(str::to_owned),
                id: case["id"].as_str().unwrap_or("fixture").into(),
            });
            let factory = NativeFactory::default()
                .with_smtp(smtp)
                .with_process(process)
                .with_ses(ses);
            let config = if key == "sesCases" {
                json!({"SES":{}})
            } else {
                case["config"].clone()
            };
            let result = event_info(config, case["options"].clone(), factory).await;
            if case["error"].is_null() {
                assert_eq!(result.unwrap(), case["result"], "{key}: {}", case["config"]);
            } else {
                assert!(result.is_err(), "{key}");
            }
        }
    }
    // Pool/string selection and unsupported SDK settings must reach the full provider.
    for config in [
        json!({"pool":true,"maxMessages":3}),
        json!("smtp://fixture/?pool=true"),
        json!({"lmtp":true}),
    ] {
        let result = event_info(
            config.clone(),
            json!({"text":"fixture"}),
            NativeFactory::default().with_provider(Arc::new(FullProvider)),
        )
        .await
        .unwrap();
        assert_eq!(result["config"], config);
    }
}
