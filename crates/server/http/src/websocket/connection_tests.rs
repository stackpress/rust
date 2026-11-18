//! Queue pressure proves finite waits, reservation release and terminal visibility.
use super::*;

fn connection() -> (
    WsConnection,
    mpsc::Receiver<Queued>,
    mpsc::Sender<Queued>,
    watch::Receiver<Option<WsClose>>,
) {
    let (outbound, outgoing) = mpsc::channel(1);
    let (incoming, inbound) = mpsc::channel(1);
    let (close, closing) = watch::channel(None);
    (
        WsConnection {
            outbound,
            inbound,
            outbound_bytes: Arc::new(Semaphore::new(4)),
            close,
            terminal: Arc::new(Terminal {
                result: Default::default(),
                changed: Notify::new(),
                closing: AtomicBool::new(false),
            }),
            snapshot: Arc::new(
                WsRequestSnapshot::capture(&Request::new(), &Default::default()).unwrap(),
            ),
            max_message: 4,
            max_queue_bytes: 4,
            send_timeout: Duration::from_secs(1),
            ended: false,
        },
        outgoing,
        incoming,
        closing,
    )
}

#[tokio::test(start_paused = true)]
async fn bounded_send_wait_releases_byte_reservations_on_failure() {
    let (mut connection, mut outgoing, _incoming, mut closing) = connection();
    assert!(
        connection
            .send(WsMessage::Binary(vec![0; 5]))
            .await
            .is_err()
    );
    connection.send(WsMessage::Text("ab".into())).await.unwrap();
    let start = tokio::time::Instant::now();
    assert!(connection.send(WsMessage::Text("cd".into())).await.is_err());
    assert_eq!(start.elapsed(), Duration::from_secs(1));
    assert_eq!(connection.outbound_bytes.available_permits(), 2);
    drop(outgoing.recv().await.unwrap());
    assert_eq!(connection.outbound_bytes.available_permits(), 4);
    connection
        .send(WsMessage::Binary(vec![1; 4]))
        .await
        .unwrap();
    assert_eq!(connection.request().method(), "GET");
    assert!(connection.close_info().is_none());
    connection
        .close(WsClose::new(3000, "done").unwrap())
        .await
        .unwrap();
    closing.changed().await.unwrap();
    assert_eq!(closing.borrow().as_ref().unwrap().code(), 3000);
    assert!(
        connection
            .send(WsMessage::Text(String::new()))
            .await
            .is_err()
    );
    connection
        .close(WsClose::new(3001, "ignored").unwrap())
        .await
        .unwrap();
    assert_eq!(closing.borrow().as_ref().unwrap().code(), 3000);
}

#[tokio::test]
async fn full_inbound_queue_cannot_hide_terminal_failure_and_eof_is_fused() {
    let (mut connection, _outgoing, incoming, _closing) = connection();
    let bytes = Arc::new(Semaphore::new(2));
    assert!(
        incoming
            .try_send(Queued {
                message: WsMessage::Text("ok".into()),
                _bytes: bytes.clone().acquire_many_owned(2).await.unwrap()
            })
            .is_ok()
    );
    let info = WsCloseInfo {
        peer: None,
        clean: false,
    };
    connection
        .terminal
        .finish(info.clone(), Some(Arc::new(Error::new("lost"))));
    connection.terminal.finish(
        WsCloseInfo {
            peer: None,
            clean: true,
        },
        None,
    );
    assert_eq!(
        connection.recv().await.unwrap(),
        Some(WsMessage::Text("ok".into()))
    );
    assert_eq!(bytes.available_permits(), 2);
    assert!(connection.recv().await.is_err());
    assert_eq!(connection.recv().await.unwrap(), None);
    assert_eq!(connection.close_info(), Some(info));
}
