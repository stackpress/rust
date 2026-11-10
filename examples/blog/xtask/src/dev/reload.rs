//! Leptos's exact reload message shapes over tungstenite, with bounded loopback clients.

use crate::BuildResult;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Duration;
use tungstenite::{Error, Message, WebSocket};

/// The official script tests `all`, `css` and a JSON-encoded string in `view`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Notice {
    /// A native-only build requires fresh pages in both browser modes.
    All,
    /// Native and hydration are ready; CSR tabs wait for their own complete bundle.
    Hydrate,
    /// Deferred CSR publication reloads only CSR tabs.
    Csr,
    /// Swap a matching stylesheet link without navigating or discarding form state.
    Css(String),
    /// The string contains serialized Leptos Patches, rather than an embedded JSON array.
    View(String),
}

impl Notice {
    /// Match `reload_script.js`: CSS is a relative URL and `all` is truthy.
    pub fn text(&self) -> String {
        match self {
            Self::Hydrate => serde_json::json!({"blog_reload":"hydrate"}).to_string(),
            Self::Csr => serde_json::json!({"blog_reload":"csr"}).to_string(),
            Self::All => serde_json::json!({"all":true}).to_string(),
            Self::Css(path) => serde_json::json!({"css":path}).to_string(),
            Self::View(patches) => serde_json::json!({"view":patches}).to_string(),
        }
    }
}

/// Own the reload listener independently from the replaceable application process.
pub(super) struct Channel {
    /// IPv4 loopback only, matching the blog server's bind address.
    listener: TcpListener,
    /// At most 32 active browser tabs, each with bounded reads and writes.
    clients: Vec<WebSocket<TcpStream>>,
    /// Ordered patches since the last native/browser build, replayed to newly opened pages.
    messages: Messages,
}

impl Channel {
    /// Open only when the explicitly selected `dev` entry calls this method.
    pub fn bind(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            clients: Vec::new(),
            messages: Messages::default(),
        })
    }

    /// Accept waiting browsers; handshake deadlines prevent a partial request blocking rebuilds.
    #[expect(
        clippy::result_large_err,
        reason = "tungstenite's handshake Callback requires its concrete HTTP ErrorResponse"
    )]
    pub fn poll(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        for _ in 0..16 {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if self.clients.len() >= 32 {
                        continue;
                    }
                    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
                    stream.set_write_timeout(Some(Duration::from_millis(100)))?;
                    // Use the library handshake, including version/key validation. The callback
                    // restricts the endpoint to the path hardcoded in Leptos's own script.
                    let mut stale = false;
                    match tungstenite::accept_hdr(
                        stream,
                        |request: &tungstenite::handshake::server::Request, response| {
                            stale = stale_page(request.uri().query(), self.messages.build);
                            endpoint(request, response)
                        },
                    ) {
                        Ok(mut socket) => {
                            socket.get_ref().set_nonblocking(true)?;
                            let replay = if stale {
                                vec![message(
                                    &self.messages.stale_notice(),
                                    self.messages.revision,
                                )]
                            } else {
                                self.messages.history.clone()
                            };
                            if replay
                                .iter()
                                .all(|message| socket.send(message.clone()).is_ok())
                            {
                                self.clients.push(socket);
                            }
                        }
                        Err(error) => eprintln!("reload handshake: {error}"),
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error.into()),
            }
        }
        // Control frames keep connections healthy; our guard adds only revision-bound failures.
        let mut fallback = self.messages.checkpoint_needed();
        self.clients
            .retain_mut(|socket| alive(socket, self.messages.revision, &mut fallback));
        Ok(self.messages.fallback(fallback))
    }

    /// A dead or slow browser must not make an otherwise successful build fail.
    pub fn send(&mut self, notice: Notice) {
        let message = self.messages.record(notice);
        self.clients
            .retain_mut(|client| client.send(message.clone()).is_ok());
    }
}

/// Protocol history is independent from socket IO and remains finite/testable.
#[derive(Default)]
struct Messages {
    /// Complete native build used by the current HTML document.
    build: u64,
    /// Latest patch/full-build revision, echoed in fallback requests.
    revision: u64,
    /// Sequential patches from the compiled baseline for new browser connections.
    history: Vec<Message>,
    /// Coalesce failures until another completed build makes the client state fresh.
    fallback_reported: bool,
    /// Stale CSR pages must wait until their renderer has been published.
    csr_pending: bool,
}

impl Messages {
    /// Reconnected CSR tabs wait for their bundle while hydration tabs catch up immediately.
    fn stale_notice(&self) -> Notice {
        if self.csr_pending {
            Notice::Hydrate
        } else {
            Notice::All
        }
    }

    /// Bound replay work; the next completed build becomes the new compiled baseline.
    fn checkpoint_needed(&self) -> bool {
        self.history.len() >= 64
    }

    /// Critical builds retire patches; CSR completion preserves the current native revision.
    fn record(&mut self, notice: Notice) -> Message {
        if matches!(notice, Notice::All | Notice::Hydrate | Notice::View(_)) {
            self.revision += 1;
        }
        if notice == Notice::Csr {
            self.csr_pending = false;
        }
        if matches!(notice, Notice::All | Notice::Hydrate) {
            self.csr_pending = notice == Notice::Hydrate;
            self.history.clear();
            self.fallback_reported = false;
        }
        let message = message(&notice, self.revision);
        if matches!(notice, Notice::View(_)) {
            self.history.push(message.clone());
        }
        message
    }

    /// A failed build waits for an edit rather than looping on the same stale browser request.
    fn fallback(&mut self, requested: bool) -> bool {
        if !requested || self.fallback_reported {
            return false;
        }
        self.fallback_reported = true;
        true
    }
}

/// Pages loaded from the last-good server during a build must reload after its replacement.
fn stale_page(query: Option<&str>, build: u64) -> bool {
    query != Some(format!("build={build}").as_str())
}

/// Critical completion includes the generation injected into the replacement server's HTML.
enum Delivery {
    Notice(Notice),
    Built(u64, Notice),
}

/// A dedicated worker keeps handshakes, close frames and patch feedback alive during Cargo.
pub(super) struct Reload {
    /// Ordered notices from the rebuild loop to the socket owner.
    send: mpsc::Sender<Delivery>,
    /// At least one current browser needs rebuilt products; consumed once per batch.
    fallback: Arc<AtomicBool>,
    /// Shared Ctrl-C flag also cancels compilation.
    stop: Arc<AtomicBool>,
    /// Joined on every exit before the listener is released.
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Reload {
    /// Only the explicit dev command opens this listener; tests exercise its pure state/IO.
    pub fn start(port: u16, stop: Arc<AtomicBool>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut channel = Channel::bind(port)?;
        let (send, receive) = mpsc::channel();
        let fallback = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let failed = fallback.clone();
        let worker = std::thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                for delivery in receive.try_iter() {
                    match delivery {
                        Delivery::Notice(notice) => channel.send(notice),
                        Delivery::Built(build, notice) => {
                            channel.messages.build = build;
                            channel.send(notice);
                        }
                    }
                }
                match channel.poll() {
                    Ok(true) => failed.store(true, Ordering::Relaxed),
                    Ok(false) => {}
                    Err(error) => {
                        eprintln!("reload channel failed: {error}");
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        Ok(Self {
            send,
            fallback,
            stop,
            worker: Some(worker),
        })
    }

    /// Report a worker failure and coalesce simultaneous current-page patch failures.
    pub fn poll(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if self
            .worker
            .as_ref()
            .is_none_or(|worker| worker.is_finished())
        {
            return Err("reload worker stopped".into());
        }
        Ok(self.fallback.swap(false, Ordering::Relaxed))
    }

    /// Send CSS/patch notices in order; a disconnected worker is an explicit loop failure.
    pub fn send(&self, notice: Notice) -> BuildResult {
        self.send.send(Delivery::Notice(notice))?;
        Ok(())
    }

    /// Record the replacement generation before accepting/reloading another browser.
    pub fn built(&self, build: u64, notice: Notice) -> BuildResult {
        self.send.send(Delivery::Built(build, notice))?;
        Ok(())
    }
}

impl Drop for Reload {
    /// No reload thread or listener can survive its owning xtask session.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Restrict upgrades to the exact endpoint used by `AutoReload`.
#[expect(
    clippy::result_large_err,
    reason = "tungstenite's handshake Callback requires its concrete HTTP ErrorResponse"
)]
fn endpoint(
    request: &tungstenite::handshake::server::Request,
    response: tungstenite::handshake::server::Response,
) -> Result<tungstenite::handshake::server::Response, tungstenite::handshake::server::ErrorResponse>
{
    if request.uri().path() == "/live_reload" {
        Ok(response)
    } else {
        let mut error = tungstenite::handshake::server::ErrorResponse::new(Some(
            "unknown reload endpoint".into(),
        ));
        *error.status_mut() = tungstenite::http::StatusCode::NOT_FOUND;
        Err(error)
    }
}

/// Keep an idle nonblocking peer; disconnect malformed/closed peers without spinning.
fn alive<S: Read + Write>(socket: &mut WebSocket<S>, revision: u64, fallback: &mut bool) -> bool {
    // Drain a bounded number of messages per tick so one tab cannot starve file polling.
    for _ in 0..8 {
        match socket.read() {
            Ok(Message::Close(_)) => {
                let _ = socket.flush();
                return false;
            }
            Ok(Message::Text(text)) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
                    && value["reload"] == true
                    && value["revision"].as_u64() == Some(revision)
                {
                    *fallback = true;
                }
            }
            Ok(_) => {
                if socket.flush().is_err() {
                    return false;
                }
            }
            Err(Error::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => return true,
            Err(_) => return false,
        }
    }
    true
}

/// Additional revision metadata is ignored by Leptos and echoed only by our failure guard.
fn message(notice: &Notice, revision: u64) -> Message {
    let text = match notice {
        Notice::View(patches) => {
            serde_json::json!({"view":patches,"revision":revision}).to_string()
        }
        _ => notice.text(),
    };
    Message::Text(text.into())
}

#[cfg(test)]
#[path = "../../tests/cases/reload.rs"]
mod tests;
