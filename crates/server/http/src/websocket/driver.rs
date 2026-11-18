//! One supervised protocol owner: IO and callback are polled inside the serving future.
use super::{
    Callback,
    connection::{
        Queued, Terminal, WsClose, WsCloseInfo, WsConnection, WsMessage, WsRequestSnapshot,
    },
};
use crate::{RealtimeStats, WebSocketHostLimits};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use stackpress_lib::{Error, Result};
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{Notify, Semaphore, mpsc, watch},
    time::Instant,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{
    Message,
    protocol::{CloseFrame, Role, WebSocketConfig, frame::coding::CloseCode},
};

/// Final supervision outcome, including stop/grace facts for the host report.
pub(crate) struct Outcome {
    pub(crate) result: Result<()>,
    pub(crate) stopped: bool,
    pub(crate) forced: bool,
}
/// The read side requests bounded control writes without blocking its next protocol read.
enum Control {
    Flush,
    Ping(Vec<u8>),
}
/// Validate dependency assertion conditions before constructing its private configuration.
fn config(limits: &WebSocketHostLimits) -> Result<WebSocketConfig> {
    limits.validate()?;
    Ok(WebSocketConfig::default()
        .read_buffer_size(limits.read)
        .write_buffer_size(limits.write)
        .max_write_buffer_size(limits.write_max)
        .max_message_size(Some(limits.message))
        .max_frame_size(Some(limits.frame))
        .accept_unmasked_frames(false))
}
/// Drive protocol IO and one callback as scoped futures; dropping this future drops them all.
pub(super) async fn run<I, F>(
    io: I,
    callback: Callback,
    snapshot: Arc<WsRequestSnapshot>,
    limits: WebSocketHostLimits,
    stop: F,
    grace: Duration,
    stats: RealtimeStats,
) -> Outcome
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    F: Future<Output = ()> + Send,
{
    let configuration = match config(&limits) {
        Ok(config) => config,
        Err(error) => {
            return Outcome {
                result: Err(error),
                stopped: false,
                forced: false,
            };
        }
    };
    let socket = WebSocketStream::from_raw_socket(io, Role::Server, Some(configuration)).await;
    let (mut sink, mut stream) = socket.split();
    let (outbound, mut outgoing) = mpsc::channel(limits.queue);
    let (incoming, inbound) = mpsc::channel(limits.queue);
    let outbound_bytes = Arc::new(Semaphore::new(limits.queue_bytes));
    let incoming_bytes = Arc::new(Semaphore::new(limits.queue_bytes));
    let (close, mut closing) = watch::channel(None);
    let terminal = Arc::new(Terminal {
        result: Default::default(),
        changed: Notify::new(),
        closing: AtomicBool::new(false),
    });
    let handle = WsConnection {
        outbound,
        inbound,
        outbound_bytes,
        close,
        terminal: Arc::clone(&terminal),
        snapshot,
        max_message: limits.message,
        max_queue_bytes: limits.queue_bytes,
        send_timeout: limits.send,
        ended: false,
    };
    // Invocation is deferred until this future is polled below the stop branch, after handoff.
    let invoked = AtomicBool::new(false);
    let invocation = &invoked;
    let callback = async move {
        invocation.store(true, Ordering::Release);
        callback(handle).await
    };
    tokio::pin!(callback);
    tokio::pin!(stop);
    let age = Instant::now() + limits.age;
    let mut idle = Instant::now() + limits.idle;
    let mut ping = Instant::now() + limits.ping;
    let mut outstanding: Option<(Vec<u8>, Option<Instant>)> = None;
    let mut sequence = 0u64;
    let mut failure = None;
    let mut peer = None;
    let mut stopped = false;
    let mut peer_closed = false;
    let mut callback_done = false;
    let requested;
    {
        // Two control slots allow one outstanding Ping plus one coalesced flush request.
        let (controls, control_rx) = mpsc::channel(2);
        let (sent, mut sent_rx) = mpsc::channel(1);
        let writer = write(
            &mut sink,
            &mut outgoing,
            control_rx,
            closing.clone(),
            sent,
            &limits,
            &stats,
        );
        tokio::pin!(writer);
        requested = loop {
            tokio::select! {
                biased;
                // Biased stop wins before a not-yet-invoked application callback.
                () = &mut stop => {
                    stopped = true;
                    break WsClose::library(1001, "server stopping");
                }
                () = tokio::time::sleep_until(age) => {
                    break WsClose::library(1001, "session expired");
                }
                changed=closing.changed(),if !callback_done=>{
                    // Explicit close takes priority over polling a ready data writer.
                    if let Some(close) = closing.borrow_and_update().clone() {
                        break close;
                    }
                    if changed.is_err() {
                        break WsClose::library(1000, "");
                    }
                }
                result=&mut callback,if !callback_done=>{
                    callback_done=true;
                    if let Err(error) = result {
                        failure = Some(Arc::new(error));
                        break WsClose::library(1011, "application failed");
                    }
                    // The callback may request close and finish in this same poll. Preserve
                    // that request before normal completion begins draining accepted data.
                    if let Some(close) = closing.borrow().clone() {
                        break close;
                    }
                    // Dropping its handle closes the outbound sender. The writer drains all
                    // already accepted messages before normal callback completion sends Close.
                }
                result=&mut writer=>{
                    if let Err(error) = result {
                        failure = Some(Arc::new(error));
                        break WsClose::library(1001, "write failed");
                    }
                    // Another application task can request close during this writer poll.
                    break closing.borrow().clone().unwrap_or_else(|| WsClose::library(1000,""));
                }
                sent=sent_rx.recv(),if outstanding.as_ref().is_some_and(|(_,deadline)|deadline.is_none())=>{
                    // Start the Pong budget only after the writer has actually flushed this Ping.
                    if sent.is_some() && let Some((_, deadline)) = &mut outstanding {
                        *deadline = Some(Instant::now() + limits.pong);
                    }
                }
                ()=tokio::time::sleep_until(outstanding.as_ref().and_then(|(_,deadline)|*deadline).unwrap_or(age)),
                    if outstanding.as_ref().is_some_and(|(_,deadline)|deadline.is_some())=>{
                    failure=Some(Arc::new(Error::new("WebSocket matching Pong deadline exceeded")));
                    break WsClose::library(1001,"pong timeout");
                }
                ()=tokio::time::sleep_until(idle)=>{
                    failure=Some(Arc::new(Error::new("WebSocket inbound idle exceeded")));
                    break WsClose::library(1001,"idle timeout");
                }
                ()=tokio::time::sleep_until(ping),if outstanding.is_none()=>{
                    let Some(next)=sequence.checked_add(1) else {break WsClose::library(1011,"ping counter exhausted");};
                    sequence=next;
                    let payload=sequence.to_be_bytes().to_vec();
                    if controls.try_send(Control::Ping(payload.clone())).is_err() {
                        failure=Some(Arc::new(Error::new("WebSocket control queue unavailable")));
                        break WsClose::library(1011,"control unavailable");
                    }
                    outstanding=Some((payload,None));
                }
                item=stream.next()=>{
                    match item {
                        Some(Ok(message))=>{
                            idle=Instant::now()+limits.idle;
                            match message {
                                Message::Text(text)=>{
                                    // A full application queue never suspends protocol control reads.
                                    if enqueue(&incoming, &incoming_bytes, WsMessage::Text(text.to_string())).is_err() {
                                        break WsClose::library(1013, "inbound capacity");
                                    }
                                }
                                Message::Binary(bytes)=>{
                                    if enqueue(&incoming, &incoming_bytes, WsMessage::Binary(bytes.to_vec())).is_err() {
                                        break WsClose::library(1013, "inbound capacity");
                                    }
                                }
                                Message::Pong(payload)=>{
                                    if outstanding.as_ref().is_some_and(|(expected,_)|expected.as_slice()==payload.as_ref()) {
                                        outstanding=None;ping=Instant::now()+limits.ping;
                                    }
                                }
                                Message::Ping(_)=>{
                                    // Tungstenite owns the automatic Pong. Coalesce wakeups while
                                    // allowing the split reader to keep processing Close/Pong.
                                    if controls.capacity()==2 {let _=controls.try_send(Control::Flush);}
                                }
                                Message::Close(frame)=>{
                                    peer=frame.map(|f|WsClose::library(u16::from(f.code),f.reason.as_str()));
                                    peer_closed=true;break WsClose::library(1000,"");
                                }
                                Message::Frame(_)=>{},
                            }
                        }
                        Some(Err(error))=>{
                            // Preserve the RFC failure category without sending internal diagnostics.
                            let code = match &error {
                                tungstenite::Error::Capacity(_) => 1009,
                                tungstenite::Error::Utf8(_) => 1007,
                                tungstenite::Error::Protocol(_) => 1002,
                                _ => 1001,
                            };
                            failure=Some(Arc::new(crate::external("WebSocket read failed",error)));
                            break WsClose::library(code,"invalid input");
                        }
                        None => {
                            failure = Some(Arc::new(Error::new("WebSocket disconnected without Close")));
                            break WsClose::library(1001, "");
                        }
                    }
                }
            }
        };
        // Cancellation here releases the split writer before the close protocol takes its sink.
    }
    terminal.closing.store(true, Ordering::Release);
    outgoing.close();
    // Free both byte and count reservations; terminal outcome storage never shares this queue.
    while outgoing.try_recv().is_ok() {}
    let mut deadline = Instant::now() + limits.close;
    if stopped {
        deadline = deadline.min(Instant::now() + grace);
    }
    let close_result;
    {
        let close = close_protocol(&mut sink, &mut stream, requested, peer_closed, &mut peer);
        tokio::pin!(close);
        close_result = loop {
            tokio::select! {biased;
                ()=&mut stop,if !stopped=>{stopped=true;deadline=deadline.min(Instant::now()+grace);}
                ()=tokio::time::sleep_until(deadline)=>break Err(Error::new("WebSocket close deadline exceeded")),
                result=&mut close=>break result,
            }
        };
    }
    let clean = close_result.is_ok();
    let mut forced = !clean && Instant::now() >= deadline;
    // Preserve the original protocol/application failure when close also fails.
    if let Err(error) = close_result
        && failure.is_none()
    {
        failure = Some(Arc::new(error));
    }
    terminal.finish(WsCloseInfo { peer, clean }, failure.clone());
    drop(incoming);
    // Give recv/close_info their terminal observation without ever extending close or grace.
    while invoked.load(Ordering::Acquire) && !callback_done {
        tokio::select! {biased;
            ()=&mut stop,if !stopped=>{stopped=true;deadline=deadline.min(Instant::now()+grace);}
            result=&mut callback=>{
                callback_done=true;
                if let Err(error) = result && failure.is_none() {
                    failure = Some(Arc::new(error));
                }
            }
            ()=tokio::time::sleep_until(deadline)=>{forced=true;break;}
        }
    }
    if forced {
        stats.force();
    }
    Outcome {
        result: failure.map_or(Ok(()), |error| {
            Err(Error::new("WebSocket session failed").with_cause(error))
        }),
        stopped,
        forced,
    }
}
/// Write accepted data concurrently with protocol reads; each native wait has a finite deadline.
async fn write<I: AsyncRead + AsyncWrite + Unpin>(
    sink: &mut SplitSink<WebSocketStream<I>, Message>,
    outgoing: &mut mpsc::Receiver<Queued>,
    mut controls: mpsc::Receiver<Control>,
    mut closing: watch::Receiver<Option<WsClose>>,
    sent: mpsc::Sender<()>,
    limits: &WebSocketHostLimits,
    stats: &RealtimeStats,
) -> Result<()> {
    let mut draining = false;
    loop {
        tokio::select! {biased;
            _ = closing.changed(), if !draining => {
                // Stop before consuming queued data when the wakeup is an explicit close.
                if closing.borrow().is_some() {
                    return Ok(());
                }
                // Sender drop drains accepted messages; no later enqueue can extend this phase.
                draining = true;
                outgoing.close();
            }
            Some(control)=controls.recv()=>{
                let ping=matches!(control,Control::Ping(_));
                let operation = async {
                    match control {
                        Control::Flush => sink.flush().await,
                        Control::Ping(payload) => sink.send(Message::Ping(payload.into())).await,
                    }
                };
                timed_write(operation,limits.send).await?;
                if ping {let _=sent.try_send(());}
            }
            queued=outgoing.recv()=>{
                let Some(queued)=queued else {return Ok(());};
                let bytes=queued.message.len();
                let message = match queued.message {
                    WsMessage::Text(text) => Message::Text(text.into()),
                    WsMessage::Binary(bytes) => Message::Binary(bytes.into()),
                };
                timed_write(sink.send(message),limits.send).await?;
                stats.add_bytes(bytes);
            }
        }
    }
}
/// Separate control/data writes share the same finite transport timeout and retained cause.
async fn timed_write<F: Future<Output = std::result::Result<(), tungstenite::Error>>>(
    operation: F,
    budget: Duration,
) -> Result<()> {
    tokio::time::timeout(budget, operation)
        .await
        .map_err(|e| crate::external("WebSocket write deadline exceeded", e))?
        .map_err(|e| crate::external("WebSocket write failed", e))
}
/// Exchange Close after the data writer has relinquished IO; the supervisor owns its deadline.
async fn close_protocol<I: AsyncRead + AsyncWrite + Unpin>(
    sink: &mut SplitSink<WebSocketStream<I>, Message>,
    stream: &mut SplitStream<WebSocketStream<I>>,
    close: WsClose,
    peer_closed: bool,
    peer: &mut Option<WsClose>,
) -> Result<()> {
    if peer_closed {
        return sink
            .flush()
            .await
            .map_err(|e| crate::external("WebSocket close acknowledgement failed", e));
    }
    sink.send(Message::Close(Some(CloseFrame {
        code: CloseCode::from(close.code),
        reason: close.reason.into(),
    })))
    .await
    .map_err(|e| crate::external("WebSocket close send failed", e))?;
    while let Some(item) = stream.next().await {
        match item {
            Ok(Message::Close(frame)) => {
                *peer = frame.map(|f| WsClose::library(u16::from(f.code), f.reason.as_str()));
                return sink
                    .flush()
                    .await
                    .map_err(|e| crate::external("WebSocket close flush failed", e));
            }
            Ok(_) => {}
            Err(error) => return Err(crate::external("WebSocket close read failed", error)),
        }
    }
    Err(Error::new("WebSocket peer omitted Close"))
}
/// Reserve both count and bytes without waiting, so overload never blocks control processing.
fn enqueue(
    sender: &mpsc::Sender<Queued>,
    bytes: &Arc<Semaphore>,
    message: WsMessage,
) -> Result<()> {
    let count = u32::try_from(message.len())
        .map_err(|_| Error::limit("WebSocket inbound cost overflow"))?;
    let permit = Arc::clone(bytes)
        .try_acquire_many_owned(count)
        .map_err(|_| Error::limit("WebSocket inbound byte capacity"))?;
    sender
        .try_send(Queued {
            message,
            _bytes: permit,
        })
        .map_err(|_| Error::limit("WebSocket inbound message capacity"))
}

#[cfg(test)]
mod tests;
