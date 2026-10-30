//! Real lettre protocol parsing over an in-memory duplex, never TCP or live SMTP.

use super::*;
use lettre::transport::smtp::client::AsyncTokioStream;
use std::{
    net::SocketAddr,
    task::{Context, Poll},
};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader, DuplexStream, ReadBuf};

/// A byte-level fake socket whose peer address is metadata only; it never binds a port.
#[derive(Debug)]
struct MemorySocket(DuplexStream);
impl AsyncRead for MemorySocket {
    /// Delegate reads to Tokio's bounded memory channel.
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}
impl AsyncWrite for MemorySocket {
    /// Delegate writes without creating a network resource.
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    /// Flush the in-memory channel only.
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    /// Close the producer half when the SDK drops its connection.
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
impl AsyncTokioStream for MemorySocket {
    /// A constant address satisfies the SDK metadata contract without a socket.
    fn peer_addr(&self) -> io::Result<SocketAddr> {
        Ok(([127, 0, 0, 1], 25).into())
    }
}

/// Read one exact SDK command so protocol-order regressions fail at their cause.
async fn command(reader: &mut BufReader<tokio::io::ReadHalf<DuplexStream>>, expected: &str) {
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    assert_eq!(line, expected);
}

/// In-memory server script for success, partial rejection and all-rejected paths.
async fn server(stream: DuplexStream, all_rejected: bool, auth: bool) -> Vec<u8> {
    let (read, mut write) = tokio::io::split(stream);
    let mut read = BufReader::new(read);
    write.write_all(b"220 fixture ESMTP\r\n").await.unwrap();
    command(&mut read, "EHLO fixture.test\r\n").await;
    write
        .write_all(if auth {
            b"250-fixture\r\n250 AUTH PLAIN LOGIN\r\n"
        } else {
            b"250 fixture\r\n"
        })
        .await
        .unwrap();
    if auth {
        let mut line = String::new();
        read.read_line(&mut line).await.unwrap();
        assert!(line.starts_with("AUTH PLAIN "));
        write
            .write_all(b"235 fixture authenticated\r\n")
            .await
            .unwrap();
    }
    command(&mut read, "MAIL FROM:<from@example.test>\r\n").await;
    write.write_all(b"250 sender accepted\r\n").await.unwrap();
    command(&mut read, "RCPT TO:<one@example.test>\r\n").await;
    write
        .write_all(if all_rejected {
            b"550 first rejected\r\n"
        } else {
            b"250 first accepted\r\n"
        })
        .await
        .unwrap();
    command(&mut read, "RCPT TO:<two@example.test>\r\n").await;
    write.write_all(b"550 second rejected\r\n").await.unwrap();
    if all_rejected {
        return vec![];
    }
    command(&mut read, "DATA\r\n").await;
    write.write_all(b"354 continue\r\n").await.unwrap();
    let mut bytes = Vec::new();
    loop {
        let mut line = Vec::new();
        read.read_until(b'\n', &mut line).await.unwrap();
        if line == b".\r\n" {
            break;
        }
        assert!(!line.is_empty());
        bytes.extend(line);
    }
    write.write_all(b"250 fixture queued\r\n").await.unwrap();
    bytes
}

/// Source accepts successful recipients even when another RCPT is rejected; no AUTH is required.
#[tokio::test]
async fn sdk_protocol_partial_rejection_and_optional_auth() {
    for auth in [false, true] {
        let (client, peer) = tokio::io::duplex(8192);
        let peer = tokio::spawn(server(peer, false, auth));
        let mut conn = AsyncSmtpConnection::connect_with_transport(
            Box::new(MemorySocket(client)),
            &ClientId::Domain("fixture.test".into()),
        )
        .await
        .unwrap();
        let config = crate::options::TransportOptions::resolve(&crate::EmailConfig::Options(
            serde_json::from_value(
                json!({"ignoreTLS":true,"auth":{"user":"fixture","pass":"fixture"}}),
            )
            .unwrap(),
        ))
        .unwrap();
        let settings = ConnectionSettings::new(&config.smtp()).unwrap();
        authenticate(&mut conn, &settings).await.unwrap();
        let body = b"Subject: fixture\r\n\r\n.body\r\n";
        let info = send_connection(
            &mut conn,
            &json!({"from":"from@example.test","to":["one@example.test","two@example.test"]}),
            body,
        )
        .await
        .unwrap();
        assert_eq!(info["accepted"], json!(["one@example.test"]));
        assert_eq!(info["rejected"], json!(["two@example.test"]));
        assert_eq!(info["response"], "250 fixture queued");
        assert_eq!(info["rejectedErrors"][0]["responseCode"], 550);
        assert_eq!(info["rejectedErrors"][0]["code"], "EENVELOPE");
        assert_eq!(info["rejectedErrors"][0]["response"], "550 second rejected");
        assert!(info["envelopeTime"].is_number() && info["messageTime"].is_number());
        // The SDK performs source-compatible SMTP transparency on a leading dot.
        assert_eq!(peer.await.unwrap(), b"Subject: fixture\r\n\r\n..body\r\n");
    }
}

/// No DATA is sent after every recipient rejects; the transport error remains visible.
#[tokio::test]
async fn sdk_protocol_all_recipients_rejected() {
    let (client, peer) = tokio::io::duplex(8192);
    let peer = tokio::spawn(server(peer, true, false));
    let mut conn = AsyncSmtpConnection::connect_with_transport(
        Box::new(MemorySocket(client)),
        &ClientId::Domain("fixture.test".into()),
    )
    .await
    .unwrap();
    let error = send_connection(
        &mut conn,
        &json!({"from":"from@example.test","to":["one@example.test","two@example.test"]}),
        b"body",
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Can't send mail - all recipients were rejected: 550 second rejected"
    );
    let source = error.downcast_ref::<SmtpError>().unwrap();
    assert_eq!(source.fields["code"], "EENVELOPE");
    assert_eq!(
        source.fields["rejected"],
        json!(["one@example.test", "two@example.test"])
    );
    assert_eq!(
        source.fields["rejectedErrors"][1]["response"],
        "550 second rejected"
    );
    assert!(std::error::Error::source(source).is_some());
    assert!(peer.await.unwrap().is_empty());
}

/// Native SDK mapping preserves configured TLS/auth choices without opening a connection.
#[test]
fn connection_mapping_tls_auth_and_mechanics_errors() {
    let options = |value: Value| {
        crate::options::TransportOptions::resolve(&serde_json::from_value(value).unwrap())
            .unwrap()
            .smtp()
    };
    let settings=ConnectionSettings::new(&options(json!({"secure":true,"host":"relay.example.test","port":465,"connectionTimeout":345,"name":"fixture.test","auth":{"user":"u","pass":"p","method":"LOGIN"},"tls":{"servername":"tls.example.test","rejectUnauthorized":false}}))).unwrap();
    assert_eq!(settings.port, 465);
    assert_eq!(settings.timeout, Duration::from_millis(345));
    assert!(matches!(settings.hello,ClientId::Domain(ref value) if value=="fixture.test"));
    assert!(matches!(settings.tls,Tls::Wrapper(ref value) if value.domain()=="tls.example.test"));
    assert!(matches!(settings.auth,Some((_,ref methods)) if methods==&[Mechanism::Login]));
    let settings = ConnectionSettings::new(&options(json!({"ignoreTLS":true,"auth":{}}))).unwrap();
    assert!(matches!(settings.tls, Tls::None));
    assert!(settings.auth.is_none());
    // Test-only certificate material is public fixture data; no TLS handshake occurs.
    let cert = include_str!("../tests/fixtures/tls-cert.pem");
    let key = include_str!("../tests/fixtures/tls-key.pem");
    let settings=ConnectionSettings::new(&options(json!({"localAddress":"127.0.0.2","secure":true,"tls":{"minVersion":"TLSv1.3","cert":cert,"key":key,"ca":[cert]}}))).unwrap();
    assert_eq!(settings.local_address, Some("127.0.0.2".parse().unwrap()));
    assert!(matches!(settings.tls, Tls::Wrapper(_)));
    let config = serde_json::from_value(
        json!({"tls":{"minVersion":"TLSv1.2","ca":{"type":"Buffer","data":cert.as_bytes()}}}),
    )
    .unwrap();
    let factory = crate::NativeFactory::default();
    let _transport = crate::TransportFactory::create(&factory, &config).unwrap();
    let settings = ConnectionSettings::new(&options(
        json!({"requireTLS":true,"auth":{"type":"OAuth2","user":"u","accessToken":"token"}}),
    ))
    .unwrap();
    assert!(matches!(settings.tls, Tls::Required(_)));
    assert!(matches!(settings.auth,Some((_,ref methods)) if methods==&[Mechanism::Xoauth2]));
    let settings = ConnectionSettings::new(&options(json!({"auth":{"type":"OAuth2"}}))).unwrap();
    assert!(matches!(settings.tls, Tls::Opportunistic(_)));
    assert!(settings.auth.is_none());
    for value in [
        json!({"port":-1}),
        json!({"port":70000}),
        json!({"port":1.5}),
        json!({"connectionTimeout":-1}),
        json!({"auth":{"user":"u","method":"unsupported"}}),
    ] {
        assert!(
            ConnectionSettings::new(&options(value.clone())).is_err(),
            "{value}"
        );
    }
}

/// Each SMTP failure preserves its source phase fields and native cause without a server.
#[tokio::test]
async fn sdk_command_error_fields_and_no_recipient_order() {
    for phase in ["MAIL", "API", "DATA", "MESSAGE"] {
        let (client, peer) = tokio::io::duplex(8192);
        let peer = tokio::spawn(async move {
            let (read, mut write) = tokio::io::split(peer);
            let mut read = BufReader::new(read);
            write.write_all(b"220 fixture\r\n").await.unwrap();
            command(&mut read, "EHLO fixture.test\r\n").await;
            write.write_all(b"250 fixture\r\n").await.unwrap();
            command(&mut read, "MAIL FROM:<from@example.test>\r\n").await;
            if phase == "MAIL" {
                write.write_all(b"550 sender denied\r\n").await.unwrap();
                return;
            }
            write.write_all(b"250 sender accepted\r\n").await.unwrap();
            if phase == "API" {
                return;
            }
            command(&mut read, "RCPT TO:<one@example.test>\r\n").await;
            write.write_all(b"250 accepted\r\n").await.unwrap();
            command(&mut read, "DATA\r\n").await;
            if phase == "DATA" {
                write.write_all(b"550 data denied\r\n").await.unwrap();
                return;
            }
            write.write_all(b"354 continue\r\n").await.unwrap();
            loop {
                let mut line = String::new();
                read.read_line(&mut line).await.unwrap();
                if line == ".\r\n" {
                    break;
                }
                assert!(!line.is_empty());
            }
            write.write_all(b"451 message deferred\r\n").await.unwrap();
        });
        let mut conn = AsyncSmtpConnection::connect_with_transport(
            Box::new(MemorySocket(client)),
            &ClientId::Domain("fixture.test".into()),
        )
        .await
        .unwrap();
        let to = if phase == "API" {
            json!([])
        } else {
            json!(["one@example.test"])
        };
        let error = send_connection(
            &mut conn,
            &json!({"from":"from@example.test","to":to}),
            b"body",
        )
        .await
        .unwrap_err();
        let error = error.downcast_ref::<SmtpError>().unwrap();
        assert_eq!(
            error.fields["code"],
            if phase == "MESSAGE" {
                "EMESSAGE"
            } else {
                "EENVELOPE"
            }
        );
        assert_eq!(
            error.fields["command"],
            match phase {
                "MAIL" => "MAIL FROM",
                "API" => "API",
                _ => "DATA",
            }
        );
        if phase == "API" {
            assert_eq!(error.to_string(), "Can't send mail - no recipients defined");
            assert!(std::error::Error::source(error).is_none());
        } else {
            assert!(error.fields["responseCode"].is_number());
            assert!(std::error::Error::source(error).is_some());
        }
        peer.await.unwrap();
    }
}
