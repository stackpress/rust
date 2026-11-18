//! Protocol decision proofs use finite duplex IO and simulated Tokio time.
use super::*;
use tokio::io::AsyncWriteExt;

fn snapshot() -> Arc<WsRequestSnapshot> {
    Arc::new(
        WsRequestSnapshot::capture(&stackpress_lib::Request::new(), &Default::default()).unwrap(),
    )
}
fn receiver(done: tokio::sync::oneshot::Sender<WsCloseInfo>) -> Callback {
    Box::new(move |mut connection| {
        Box::pin(async move {
            while connection.recv().await?.is_some() {}
            done.send(connection.close_info().expect("terminal close info"))
                .unwrap();
            Ok(())
        })
    })
}
fn masked(opcode: u8, fin: bool, data: &[u8]) -> Vec<u8> {
    assert!(data.len() < 126);
    let mut frame = vec![
        opcode | if fin { 128 } else { 0 },
        128 | data.len() as u8,
        1,
        2,
        3,
        4,
    ];
    frame.extend(
        data.iter()
            .enumerate()
            .map(|(i, b)| b ^ ([1, 2, 3, 4][i % 4])),
    );
    frame
}

#[tokio::test(start_paused = true)]
async fn explicit_close_preempts_queued_data_and_survives_callback_return() {
    for returns in [true, false] {
        let (server, client) = tokio::io::duplex(4096);
        let callback: Callback = Box::new(move |mut connection| {
            Box::pin(async move {
                // Fill the accepted queue before yielding to the protocol owner.
                for _ in 0..16 {
                    connection.send(WsMessage::Text("private".into())).await?;
                }
                connection
                    .close(WsClose::new(1008, "session expired")?)
                    .await?;
                if !returns {
                    while connection.recv().await?.is_some() {}
                }
                Ok(())
            })
        });
        let serving = run(
            server,
            callback,
            snapshot(),
            Default::default(),
            std::future::pending(),
            Duration::from_secs(1),
            Default::default(),
        );
        let peer = async {
            let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
            // An explicit close has priority even when every data-queue slot is occupied.
            let frame = client.next().await.unwrap().unwrap();
            let Message::Close(Some(close)) = frame else {
                panic!("unexpected frame: {frame:?}")
            };
            assert_eq!(u16::from(close.code), 1008);
            assert_eq!(close.reason.as_str(), "session expired");
            client.flush().await.unwrap();
        };
        let (outcome, ()) = tokio::join!(serving, peer);
        assert!(outcome.result.is_ok());
        assert!(!outcome.forced);
    }
}
async fn peer_close<I: AsyncRead + AsyncWrite + Unpin>(client: &mut WebSocketStream<I>) -> u16 {
    let message = client.next().await.unwrap().unwrap();
    let Message::Close(Some(close)) = message else {
        panic!("expected close, got {message:?}")
    };
    client.flush().await.unwrap();
    u16::from(close.code)
}

/// A fatal parse error fuses the protocol reader; the peer may already have dropped IO.
async fn malformed_close<I: AsyncRead + AsyncWrite + Unpin>(
    client: &mut WebSocketStream<I>,
    expected: u16,
) {
    let message = client.next().await.unwrap().unwrap();
    let Message::Close(Some(close)) = message else {
        panic!("expected error Close, got {message:?}")
    };
    assert_eq!(u16::from(close.code), expected);
    match client.flush().await {
        Ok(()) | Err(tungstenite::Error::ConnectionClosed) => {}
        Err(tungstenite::Error::Io(error)) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
        Err(error) => panic!("unexpected close acknowledgement error: {error}"),
    }
}

#[tokio::test]
async fn terminal_delivery_survives_peer_close_and_data_queue() {
    let (server, client) = tokio::io::duplex(4096);
    let (done, observed) = tokio::sync::oneshot::channel();
    let serving = run(
        server,
        receiver(done),
        snapshot(),
        Default::default(),
        std::future::pending(),
        Duration::from_secs(1),
        Default::default(),
    );
    let peer = async {
        let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
        client.send(Message::Text("one".into())).await.unwrap();
        client
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "bye".into(),
            })))
            .await
            .unwrap();
        assert_eq!(peer_close(&mut client).await, 1000);
    };
    let (outcome, ()) = tokio::join!(serving, peer);
    assert!(outcome.result.is_ok());
    assert!(!outcome.forced);
    assert_eq!(
        observed.await.unwrap(),
        WsCloseInfo {
            peer: Some(WsClose::new(1000, "bye").unwrap()),
            clean: true
        }
    );
}

#[tokio::test]
async fn fragmentation_reassembles_text_and_binary_without_exposing_controls() {
    let (server, mut client) = tokio::io::duplex(4096);
    let callback: Callback = Box::new(|mut connection| {
        Box::pin(async move {
            assert_eq!(connection.recv().await?, Some(WsMessage::Text("hé".into())));
            assert_eq!(
                connection.recv().await?,
                Some(WsMessage::Binary(vec![0, 255]))
            );
            connection.send(WsMessage::Text("received".into())).await
        })
    });
    let serving = run(
        server,
        callback,
        snapshot(),
        Default::default(),
        std::future::pending(),
        Duration::from_secs(1),
        Default::default(),
    );
    let peer = async {
        client
            .write_all(&masked(1, false, &[b'h', 0xc3]))
            .await
            .unwrap();
        client
            .write_all(&masked(9, true, b"control"))
            .await
            .unwrap();
        client.write_all(&masked(0, true, &[0xa9])).await.unwrap();
        client.write_all(&masked(2, true, &[0, 255])).await.unwrap();
        let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
        let mut pong = false;
        let mut text = false;
        for _ in 0..3 {
            match client.next().await.unwrap().unwrap() {
                Message::Pong(data) => {
                    assert_eq!(data.as_ref(), b"control");
                    pong = true;
                }
                Message::Text(data) => {
                    assert_eq!(data.as_str(), "received");
                    text = true;
                }
                Message::Close(close) => {
                    assert_eq!(u16::from(close.unwrap().code), 1000);
                    client.flush().await.unwrap();
                    break;
                }
                other => panic!("unexpected {other:?}"),
            }
        }
        assert!(pong && text);
    };
    let (outcome, ()) = tokio::join!(serving, peer);
    assert!(outcome.result.is_ok(), "{:?}", outcome.result);
}

#[tokio::test]
async fn wire_errors_choose_bounded_close_codes_and_fused_receive_error() {
    // Invalid UTF-8, unmasked text, fragmented control, and frame/message overflow.
    for (wire, limits, expected) in [
        (
            masked(1, true, &[255]),
            WebSocketHostLimits::default(),
            1007,
        ),
        (vec![0x81, 1, b'x'], WebSocketHostLimits::default(), 1002),
        (masked(9, false, b"x"), WebSocketHostLimits::default(), 1002),
        (
            masked(2, true, b"12345"),
            WebSocketHostLimits::default()
                .with_message_limits(4, 4)
                .unwrap(),
            1009,
        ),
        (
            [masked(1, false, b"123"), masked(0, true, b"45")].concat(),
            WebSocketHostLimits::default()
                .with_message_limits(4, 4)
                .unwrap(),
            1009,
        ),
    ] {
        let (server, mut client) = tokio::io::duplex(4096);
        let (seen, observed) = tokio::sync::oneshot::channel();
        let callback: Callback = Box::new(|mut connection| {
            Box::pin(async move {
                assert!(connection.recv().await.is_err());
                assert!(connection.recv().await?.is_none());
                assert!(connection.close_info().is_some());
                seen.send(()).unwrap();
                Ok(())
            })
        });
        let serving = run(
            server,
            callback,
            snapshot(),
            limits,
            std::future::pending(),
            Duration::from_secs(1),
            Default::default(),
        );
        let peer = async {
            client.write_all(&wire).await.unwrap();
            let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
            malformed_close(&mut client, expected).await;
        };
        let (outcome, ()) = tokio::join!(serving, peer);
        assert!(outcome.result.is_err());
        observed.await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn stop_sends_close_before_cancelling_an_uncooperative_callback() {
    let (server, client) = tokio::io::duplex(4096);
    let (open, opened) = tokio::sync::oneshot::channel();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let dropped = Arc::new(AtomicBool::new(false));
    struct Dropped(Arc<AtomicBool>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let owned = Arc::clone(&dropped);
    let callback: Callback = Box::new(move |connection| {
        Box::pin(async move {
            let _connection = connection;
            let _guard = Dropped(owned);
            open.send(()).unwrap();
            std::future::pending::<Result<()>>().await
        })
    });
    let stats = RealtimeStats::default();
    let serving = run(
        server,
        callback,
        snapshot(),
        Default::default(),
        async {
            let _ = stopped.await;
        },
        Duration::from_millis(50),
        stats.clone(),
    );
    let peer = async {
        opened.await.unwrap();
        stop.send(()).unwrap();
        let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
        assert_eq!(peer_close(&mut client).await, 1001);
        assert!(!dropped.load(Ordering::SeqCst));
    };
    let start = Instant::now();
    let (outcome, ()) = tokio::join!(serving, peer);
    assert!(outcome.stopped && outcome.forced);
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(start.elapsed(), Duration::from_millis(50));
    assert_eq!(stats.forced(), 1);
}

#[tokio::test(start_paused = true)]
async fn matching_pong_clock_and_non_receiving_application() {
    let (server, client) = tokio::io::duplex(4096);
    let callback: Callback = Box::new(|connection| {
        Box::pin(async move {
            let _connection = connection;
            std::future::pending().await
        })
    });
    let serving = run(
        server,
        callback,
        snapshot(),
        Default::default(),
        std::future::pending(),
        Duration::from_secs(1),
        Default::default(),
    );
    let peer = async {
        let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
        let Message::Ping(first) = client.next().await.unwrap().unwrap() else {
            panic!("ping")
        };
        // The native client queues a matching Pong automatically; overwrite it with an unmatched
        // Pong, proving unrelated inbound traffic cannot cancel the outstanding deadline.
        client.send(Message::Pong(vec![42].into())).await.unwrap();
        assert!(!first.is_empty());
        assert_eq!(peer_close(&mut client).await, 1001);
    };
    let start = Instant::now();
    let (outcome, ()) = tokio::join!(serving, peer);
    assert!(outcome.result.is_err());
    assert_eq!(start.elapsed(), Duration::from_secs(35));
}

#[tokio::test]
async fn already_stopped_handoff_never_invokes_callback_even_for_terminal_delivery() {
    let (server, client) = tokio::io::duplex(4096);
    let callback: Callback = Box::new(|_| panic!("open after observed stop"));
    let serving = run(
        server,
        callback,
        snapshot(),
        Default::default(),
        async {},
        Duration::from_secs(1),
        Default::default(),
    );
    let peer = async {
        let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
        assert_eq!(peer_close(&mut client).await, 1001);
    };
    let (outcome, ()) = tokio::join!(serving, peer);
    assert!(outcome.stopped);
    assert!(outcome.result.is_ok());
    assert!(!outcome.forced);
}

#[tokio::test(start_paused = true)]
async fn inbound_overload_age_and_application_error_choose_distinct_closes() {
    for (mode, expected) in [("overload", 1013), ("age", 1001), ("application", 1011)] {
        let (server, client) = tokio::io::duplex(4096);
        let callback: Callback = Box::new(move |connection| {
            Box::pin(async move {
                let _connection = connection;
                if mode == "application" {
                    return Err(Error::new("private app error"));
                }
                std::future::pending().await
            })
        });
        let limits = WebSocketHostLimits::default()
            .with_queue_limits(1, 4)
            .unwrap()
            .with_max_age(Duration::from_secs(2))
            .unwrap();
        let serving = run(
            server,
            callback,
            snapshot(),
            limits,
            std::future::pending(),
            Duration::from_secs(1),
            Default::default(),
        );
        let peer = async {
            let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
            if mode == "overload" {
                client.send(Message::Text("data".into())).await.unwrap();
                client.send(Message::Text("more".into())).await.unwrap();
            }
            assert_eq!(peer_close(&mut client).await, expected);
        };
        let start = Instant::now();
        let (outcome, ()) = tokio::join!(serving, peer);
        assert_eq!(outcome.forced, mode != "application");
        assert_eq!(
            start.elapsed(),
            Duration::from_secs(match mode {
                "age" => 7,
                "overload" => 5,
                _ => 0,
            })
        );
    }
}

#[tokio::test(start_paused = true)]
async fn matching_pong_rearms_ping_without_application_recv() {
    let (server, client) = tokio::io::duplex(4096);
    let callback: Callback = Box::new(|connection| {
        Box::pin(async move {
            let _connection = connection;
            std::future::pending().await
        })
    });
    let (stop, stopping) = tokio::sync::oneshot::channel();
    let serving = run(
        server,
        callback,
        snapshot(),
        Default::default(),
        async {
            let _ = stopping.await;
        },
        Duration::from_millis(1),
        Default::default(),
    );
    let peer = async {
        let mut client = WebSocketStream::from_raw_socket(client, Role::Client, None).await;
        for _ in 0..2 {
            assert!(matches!(
                client.next().await.unwrap().unwrap(),
                Message::Ping(_)
            ));
            client.flush().await.unwrap();
        }
        stop.send(()).unwrap();
        assert_eq!(peer_close(&mut client).await, 1001);
    };
    let start = Instant::now();
    let (outcome, ()) = tokio::join!(serving, peer);
    assert!(outcome.stopped && outcome.forced);
    assert_eq!(start.elapsed(), Duration::from_millis(40001));
}
