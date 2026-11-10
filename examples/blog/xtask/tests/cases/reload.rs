//! Tungstenite handshakes and framing use in-memory IO, never a bound socket.
use super::*;
use std::io::{self, Cursor};
use tungstenite::protocol::Role;

/// An idle nonblocking socket model with owned incoming and outgoing bytes.
#[derive(Default)]
struct Memory {
    input: Cursor<Vec<u8>>,
    output: Vec<u8>,
    fail_write: bool,
}
impl Read for Memory {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.input.position() == self.input.get_ref().len() as u64 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        self.input.read(out)
    }
}
impl Write for Memory {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail_write {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        self.output.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encode with the real library, including client masking, for a server-side frame test.
fn frames(messages: Vec<Message>, role: Role) -> Vec<u8> {
    let mut socket = WebSocket::from_raw_socket(Memory::default(), role, None);
    for message in messages {
        socket.send(message).unwrap();
    }
    socket.into_inner().output
}

/// The client checks these exact JSON keys and treats CSS as a relative stylesheet URL.
#[test]
fn matches_leptos_reload_protocol() {
    assert_eq!(Notice::All.text(), r#"{"all":true}"#);
    assert_eq!(
        Notice::Css("styles/global.css".into()).text(),
        r#"{"css":"styles/global.css"}"#
    );
    let encoded = frames(vec![Message::Text(Notice::All.text().into())], Role::Server);
    let mut client = WebSocket::from_raw_socket(
        Memory {
            input: Cursor::new(encoded),
            ..Default::default()
        },
        Role::Client,
        None,
    );
    assert_eq!(
        client.read().unwrap().into_text().unwrap(),
        Notice::All.text()
    );
    let encoded = message(&Notice::View("[]".into()), 7).into_text().unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["view"], "[]");
    assert_eq!(value["revision"], 7);
}

/// New pages replay patches in order; complete builds retire history and stale failure requests.
#[test]
fn retains_patch_history_and_coalesces_fallbacks() {
    let mut messages = Messages::default();
    messages.record(Notice::View("first".into()));
    messages.record(Notice::Css("styles/global.css".into()));
    messages.record(Notice::View("second".into()));
    assert_eq!(messages.history.len(), 2);
    assert_eq!(messages.revision, 2);
    assert!(!messages.fallback(false));
    assert!(messages.fallback(true));
    assert!(!messages.fallback(true));
    messages.record(Notice::All);
    assert!(messages.history.is_empty());
    assert_eq!(messages.revision, 3);
    assert!(messages.fallback(true));
    assert!(!stale_page(Some("build=3"), 3));
    assert!(stale_page(Some("build=2"), 3));
    assert!(stale_page(None, 3));
    let mut checkpoint = Messages::default();
    for _ in 0..63 {
        checkpoint.record(Notice::View("[]".into()));
    }
    assert!(!checkpoint.checkpoint_needed());
    checkpoint.record(Notice::View("[]".into()));
    assert!(checkpoint.checkpoint_needed());
    for (text, expected) in [
        (r#"{"reload":true,"revision":3}"#, true),
        (r#"{"reload":true,"revision":2}"#, false),
        (r#"{"reload":false,"revision":3}"#, false),
        ("bad JSON", false),
    ] {
        let input = frames(vec![Message::Text(text.into())], Role::Client);
        let mut socket = WebSocket::from_raw_socket(
            Memory {
                input: Cursor::new(input),
                ..Default::default()
            },
            Role::Server,
            None,
        );
        let mut fallback = false;
        assert!(alive(&mut socket, 3, &mut fallback));
        assert_eq!(fallback, expected);
    }
}

/// The real HTTP upgrade validates the RFC key and our exact reload path entirely in memory.
#[test]
fn handshakes_on_the_official_endpoint() {
    let request = "GET /live_reload HTTP/1.1\r\nHost: localhost:3001\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";
    let socket = tungstenite::accept_hdr(
        Memory {
            input: Cursor::new(request.as_bytes().to_vec()),
            ..Default::default()
        },
        endpoint,
    )
    .unwrap();
    let response = String::from_utf8(socket.into_inner().output).unwrap();
    assert!(response.starts_with("HTTP/1.1 101"));
    assert!(response.contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
    let bad = request.replace("/live_reload", "/wrong");
    assert!(
        tungstenite::accept_hdr(
            Memory {
                input: Cursor::new(bad.into_bytes()),
                ..Default::default()
            },
            endpoint
        )
        .is_err()
    );
    assert!(
        tungstenite::accept_hdr(
            Memory {
                input: Cursor::new(b"invalid\r\n\r\n".to_vec()),
                ..Default::default()
            },
            endpoint
        )
        .is_err()
    );
}

/// Idle, chatty, pinging, closing and malformed peers cannot block file polling.
#[test]
fn processes_control_frames_and_retires_failed_peers() {
    let mut idle = WebSocket::from_raw_socket(Memory::default(), Role::Server, None);
    assert!(alive(&mut idle, 0, &mut false));
    let input = frames(vec![Message::Ping(vec![1, 2].into())], Role::Client);
    let mut ping = WebSocket::from_raw_socket(
        Memory {
            input: Cursor::new(input),
            ..Default::default()
        },
        Role::Server,
        None,
    );
    assert!(alive(&mut ping, 0, &mut false));
    assert!(!ping.get_ref().output.is_empty());
    let input = frames(vec![Message::Close(None)], Role::Client);
    let mut close = WebSocket::from_raw_socket(
        Memory {
            input: Cursor::new(input),
            ..Default::default()
        },
        Role::Server,
        None,
    );
    assert!(!alive(&mut close, 0, &mut false));
    let mut malformed = WebSocket::from_raw_socket(
        Memory {
            input: Cursor::new(vec![255; 20]),
            ..Default::default()
        },
        Role::Server,
        None,
    );
    assert!(!alive(&mut malformed, 0, &mut false));
    let input = frames(vec![Message::Text("ignored".into()); 10], Role::Client);
    let mut chatty = WebSocket::from_raw_socket(
        Memory {
            input: Cursor::new(input),
            ..Default::default()
        },
        Role::Server,
        None,
    );
    assert!(alive(&mut chatty, 0, &mut false));
    assert!(alive(&mut chatty, 0, &mut false));
    let input = frames(vec![Message::Ping(vec![1].into())], Role::Client);
    let mut broken = WebSocket::from_raw_socket(
        Memory {
            input: Cursor::new(input),
            fail_write: true,
            ..Default::default()
        },
        Role::Server,
        None,
    );
    assert!(!alive(&mut broken, 0, &mut false));
}
