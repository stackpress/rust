//! Compare actual reference envelopes and preserve exact Rust byte-prefix behavior.

use serde_json::{Value, json};
use stackpress_cli::{CliSinks, Terminal};
use stackpress_example_sql_profile::build_application;
use stackpress_lib::ParseLimits;
use stackpress_server::http::{BufferedRequest, HttpAdapter, HttpOptions, HttpOutput};
use stackpress_server::{ConfigLayers, Server};
use stackpress_sql::{Insert, SqlValue};
use tempfile::TempDir;

use super::{close, layers, writable};

/// Serialize the complete public direct-call envelope, without fabricating an expected surrogate.
async fn direct(app: &Server, data: Value) -> Vec<u8> {
    serde_json::to_vec(
        &app.call("profile-search", data)
            .await
            .unwrap()
            .response
            .try_to_json()
            .unwrap(),
    )
    .unwrap()
}

/// Use the same real CLI runner/parser as generated-profile's parity proof, retaining its newline.
async fn cli(app: &Server, id: &str) -> (Vec<u8>, Vec<u8>) {
    let args = ["emit", "profile-search", "--id", id].map(String::from);
    let terminal = Terminal::parse(&args, ParseLimits::default()).unwrap();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    stackpress_cli::run(app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
        .await
        .unwrap();
    (stdout, stderr)
}

/// Encode a path segment by its UTF-8 bytes; operators and percent text cannot change the URI shape.
fn segment(id: &str) -> String {
    id.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

/// Buffer a real HTTP request without creating any native socket or listener.
async fn http(app: &Server, uri: &str) -> (u16, Vec<u8>) {
    let mut request = BufferedRequest::new(Vec::new());
    *request.uri_mut() = uri.parse().unwrap();
    let reply = HttpAdapter::new(HttpOptions::default())
        .unwrap()
        .handle(app, request)
        .await
        .unwrap();
    let HttpOutput::Response(response) = reply.output else {
        panic!("expected HTTP envelope");
    };
    (response.status().as_u16(), response.body().clone())
}

/// Direct, CLI and buffered HTTP envelope bytes agree with the actual original application.
pub(crate) async fn parity() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("profiles.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap();
    for limit in [1, 3, 100] {
        let app = build_application(layers(&path, limit)).await.unwrap();
        let mut original_layers = ConfigLayers::new();
        original_layers
            .value("test", json!({"profiles":{"limit":limit}}))
            .unwrap();
        let reference = stackpress_example_capability_host::build_application(original_layers)
            .await
            .unwrap();
        for id in [
            "abc123",
            "abc",
            "abc4",
            "unknown",
            "ABC",
            "abc123-extra",
            "%",
            "_",
            "\\",
            "'",
            "λ",
            "abc%",
            "abc_",
            "abc\\",
            "abc' OR 1=1 --",
        ] {
            let input = json!({"id":id});
            let actual = direct(&app, input.clone()).await;
            assert_eq!(
                actual,
                direct(&reference, input).await,
                "direct {id:?}, limit {limit}"
            );
            let cli = cli(&app, id).await;
            assert_eq!(cli, self::cli(&reference, id).await, "CLI {id:?}");
            let mut framed = actual.clone();
            framed.push(b'\n');
            assert_eq!(cli, (framed, Vec::new()));
            let uri = format!("/profiles/{}", segment(id));
            let response = http(&app, &uri).await;
            assert_eq!(response, http(&reference, &uri).await, "HTTP {id:?}");
            assert_eq!(response, (200, actual));
        }
        for input in [
            json!({}),
            json!({"id":""}),
            json!({"id":3}),
            json!({"id":null}),
            json!({"id":{}}),
            json!({"id":"abc\u{0}"}),
            json!({"id":"abc","limit":100,"sqlite":{"path":"wrong.sqlite"},"sql":"DROP TABLE profile"}),
        ] {
            assert_eq!(
                direct(&app, input.clone()).await,
                direct(&reference, input).await
            );
        }
        // Retain the existing host's path spelling/capture policy, including percent escapes.
        // The actual reference determines HTTP behavior; the SQL plugin adds no URL decoder.
        // Route selection stays independent from SQL service registration.
        for uri in [
            "/profiles",
            "/sql?event=profile-search&id=abc",
            "/profiles/abc%00",
            "/profiles/%61bc",
            "/profiles/%25",
            "/profiles/%5C",
        ] {
            assert_eq!(http(&app, uri).await, http(&reference, uri).await);
        }
        let actual = app
            .call("sql-query", json!({"sql":"SELECT * FROM profile"}))
            .await
            .unwrap();
        let expected = reference
            .call("sql-query", json!({"sql":"SELECT * FROM profile"}))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_vec(&actual.response.try_to_json().unwrap()).unwrap(),
            serde_json::to_vec(&expected.response.try_to_json().unwrap()).unwrap()
        );
        close(app).await;
    }
    // A later code layer still wins over a file-shaped layer and request-supplied limit.
    let mut config = layers(&path, 2);
    config
        .value("last override", json!({"profiles":{"limit":1}}))
        .unwrap();
    let app = build_application(config).await.unwrap();
    let value: Value =
        serde_json::from_slice(&direct(&app, json!({"id":"abc","limit":100})).await).unwrap();
    assert_eq!(value["total"], 1);
    assert_eq!(value["results"][0]["name"], "Ada Lovelace");
    close(app).await;
}

/// Positive literal matches prevent wildcard/case/escaping bugs from hiding behind empty results.
pub(crate) async fn literals() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("profiles.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap();
    let extra = [
        "abcZ",
        "abcA",
        "%percent",
        "_underscore",
        "\\slash",
        "'quote",
        "λunicode",
        "ABCupper",
        "abc%literal",
        "abc_literal",
        "abc\\literal",
        "abc'quote",
        "abcλ",
        "abc\0suffix",
    ];
    let database = writable(&path).await;
    for (index, id) in extra.iter().enumerate() {
        database
            .execute(
                &Insert::new("profile")
                    .columns(["ordinal", "id", "name"])
                    .values([
                        SqlValue::Integer(index as i64 + 3),
                        (*id).into(),
                        format!("name {index}").into(),
                    ]),
            )
            .await
            .unwrap();
    }
    database.close().await.unwrap();
    let app = build_application(layers(&path, 100)).await.unwrap();
    let all = ["abc123", "abc456", "abc789"]
        .into_iter()
        .chain(extra)
        .collect::<Vec<_>>();
    for prefix in [
        "abc",
        "%",
        "_",
        "\\",
        "'",
        "λ",
        "ABC",
        "abc%",
        "abc_",
        "abc\\",
        "abc'",
        "abcλ",
        "abc\0",
        "abc\0suffix",
        "abc' OR 1=1 --",
    ] {
        let value: Value =
            serde_json::from_slice(&direct(&app, json!({"id":prefix})).await).unwrap();
        let expected = all
            .iter()
            .copied()
            .filter(|id| id.starts_with(prefix))
            .collect::<Vec<_>>();
        let actual = value["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "prefix {prefix:?}");
        assert_eq!(value["total"], expected.len());
        if !prefix.contains('\0') {
            // Positive hostile matches pass through the shared CLI parser/runner too.
            let mut framed = serde_json::to_vec(&value).unwrap();
            framed.push(b'\n');
            assert_eq!(cli(&app, prefix).await, (framed, Vec::new()));
        }
    }
    close(app).await;
}
