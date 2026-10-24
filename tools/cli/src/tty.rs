//! Exclusive native terminal input, bounded polling and joined worker cleanup.
//!
//! TtySession owns raw-mode restoration; TtyInput owns that guard and one polling
//! worker. Only terminal acquisition uses the process-wide
//! lease; host services and async callbacks have no registry/global lock.

use std::io::{self, IsTerminal};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossterm::{event, terminal};
use tokio::sync::{mpsc, oneshot};

use crate::prompt::{PromptCancellation, PromptEvent, PromptInput};

/// The process has one native stdin terminal; concurrent framework readers are rejected.
static TERMINAL_LEASED: AtomicBool = AtomicBool::new(false);

/// One explicitly owned stdin terminal lease with prior raw-mode restoration.
///
/// Call [`Self::close`] to observe restoration errors. Drop attempts restoration on
/// cancellation/unwind, but cannot guarantee success after an operating-system I/O
/// failure. This is not a cancellable reader for arbitrary pipes or caller code.
pub struct TtySession {
    /// True only when this owner enabled raw mode over a previously non-raw terminal.
    restore_raw_mode: bool,
    /// Idempotent close/drop must release the process lease exactly once.
    active: bool,
}

impl TtySession {
    /// Acquire exclusive native terminal input, retaining the previous raw-mode policy.
    ///
    /// # Errors
    /// Returns immediately for non-TTY stdin or an existing framework lease; native
    /// query/setup errors release the provisional lease before returning.
    ///
    /// # Examples
    /// ```no_run
    /// use stackpress_cli::tty::TtySession;
    /// let mut session = TtySession::acquire()?;
    /// // A synchronous native poll belongs on an owned worker, not an async executor thread.
    /// let event = session.poll(std::time::Duration::from_millis(20))?;
    /// session.close()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn acquire() -> io::Result<Self> {
        // Never start an uncancellable blocking pipe reader under a terminal API.
        if !io::stdin().is_terminal() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "stdin is not a terminal",
            ));
        }
        // Compare/exchange grants the only framework reader lease without a poisonable mutex.
        if TERMINAL_LEASED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "terminal input is already owned",
            ));
        }
        // Own cleanup before the first fallible OS query/setup operation.
        let mut guard = Self {
            restore_raw_mode: false,
            active: true,
        };
        let was_raw = terminal::is_raw_mode_enabled()?;
        if !was_raw {
            // Restore only a transition owned by this guard; preserve an enclosing raw-mode owner.
            terminal::enable_raw_mode()?;
            guard.restore_raw_mode = true;
        }
        Ok(guard)
    }

    /// Poll one native event under this exclusive reader lease.
    ///
    /// # Errors
    /// Returns native polling/read errors or invalid closed-guard use. The supplied
    /// timeout applies to crossterm polling, not arbitrary competing application
    /// calls into crossterm's own process-wide internals. Call only from the owned
    /// input worker; no executor thread should block on this synchronous boundary.
    ///
    /// # Examples
    /// ```no_run
    /// use stackpress_cli::tty::TtySession;
    /// let mut session = TtySession::acquire()?;
    /// // A synchronous native poll belongs on an owned worker, not an async executor thread.
    /// let event = session.poll(std::time::Duration::from_millis(20))?;
    /// session.close()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn poll(&mut self, timeout: Duration) -> io::Result<Option<event::Event>> {
        // Closing the owner removes its authority to consume subsequent terminal input.
        if !self.active {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "terminal session is closed",
            ));
        }
        if event::poll(timeout)? {
            // Crossterm promises a following read is ready; no other framework reader has a lease.
            event::read().map(Some)
        } else {
            // No readiness means no read: idle workers can observe their stop signal next
            // iteration.
            Ok(None)
        }
    }

    /// Restore prior raw mode and release the exclusive lease without reading input.
    ///
    /// # Errors
    /// Returns an OS restoration failure; Drop makes a final best-effort attempt.
    ///
    /// # Examples
    /// ```no_run
    /// use stackpress_cli::tty::TtySession;
    /// let mut session = TtySession::acquire()?;
    /// // A synchronous native poll belongs on an owned worker, not an async executor thread.
    /// let event = session.poll(std::time::Duration::from_millis(20))?;
    /// session.close()?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn close(&mut self) -> io::Result<()> {
        // A successful repeated close has no additional terminal side effects.
        if !self.active {
            return Ok(());
        }
        if self.restore_raw_mode {
            // Keep active/restore flags set if the OS fails, allowing Drop's final retry.
            terminal::disable_raw_mode()?;
            self.restore_raw_mode = false;
        }
        // Release authority only after successful restoration, so a new owner cannot race cleanup.
        self.active = false;
        TERMINAL_LEASED.store(false, Ordering::Release);
        Ok(())
    }
}

impl Drop for TtySession {
    /// Restore raw mode on normal return, cancellation and unwind without panicking.
    fn drop(&mut self) {
        // Drop cannot report an OS error; callers needing evidence use explicit close.
        if self.close().is_err() {
            // Never advertise the terminal as available after failed restoration.
            TERMINAL_LEASED.store(true, Ordering::Release);
        }
    }
}

/// One native input worker and bounded event queue; non-Clone and exclusively leased.
///
/// Only crossterm polling/event mapping runs in the worker, never callbacks or writers.
/// Drop stops/joins it synchronously with a 20-ms polling/backpressure interval under
/// the exclusive-reader contract. Arbitrary competing crossterm callers and OS stalls
/// are outside this bound. Explicit [`Self::close`] awaits worker exit before joining.
pub struct TtyInput {
    /// Shared only with this worker, never with application callbacks.
    stop: Arc<AtomicBool>,
    /// The single exact worker to reap on all paths.
    worker: Option<JoinHandle<io::Result<()>>>,
    /// Worker completion notification is sent/dropped before joining, including unwind.
    done: Option<oneshot::Receiver<()>>,
    /// Backpressure is bounded to 32 small native key events.
    events: mpsc::Receiver<io::Result<PromptEvent>>,
}

impl TtyInput {
    /// Inspect buffered events without reading stdin or waiting on a worker.
    ///
    /// Diagnostic only: this snapshot is not a readiness promise. The owned worker can
    /// add another event immediately. The native queue capacity is 32 events.
    pub fn buffered_events(&self) -> usize {
        // Inspect only the async queue, never the OS input or the worker's terminal guard.
        self.events.len()
    }

    /// Acquire native stdin and start one controlled input worker.
    ///
    /// # Errors
    /// Rejects non-TTY input/existing ownership or OS/thread creation failure.
    /// A failed spawn drops the provisional guard, restoring raw mode immediately.
    ///
    /// # Examples
    /// ```no_run
    /// use stackpress_cli::prompt::{PromptCancellation, PromptInput};
    /// use stackpress_cli::tty::TtyInput;
    /// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
    /// let mut input = TtyInput::acquire(PromptCancellation::new())?;
    /// let event = input.next().await?;
    /// input.close().await?;
    /// # Ok::<(), std::io::Error>(()) })?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn acquire(cancellation: PromptCancellation) -> io::Result<Self> {
        // Acquire before spawning; a failed spawn drops this guard and restores provisional raw
        // mode.
        let session = TtySession::acquire()?;
        // Share a stop flag with exactly one worker; capacity bounds queued native events.
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let (sender, events) = mpsc::channel(32);
        let (completed, done) = oneshot::channel();
        let worker = thread::Builder::new()
            .name("stackpress-prompt".into())
            .spawn(move || {
                // Move the guard into the worker so every unwind/error path retains its destructor.
                let result = read_events(session, &worker_stop, &sender, &cancellation);
                // read_events drops/restores its session before normal completion is advertised.
                let _ = completed.send(());
                result
            })?;
        Ok(Self {
            stop,
            worker: Some(worker),
            done: Some(done),
            events,
        })
    }

    /// Stop without waiting for more input, await completion, then reap this exact worker.
    ///
    /// # Errors
    /// Reports worker panic, native input or raw-mode restoration failure.
    /// Cancellation of close is safe: Drop still stops/joins the owned worker.
    ///
    /// # Examples
    /// ```no_run
    /// use stackpress_cli::prompt::{PromptCancellation, PromptInput};
    /// use stackpress_cli::tty::TtyInput;
    /// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
    /// let mut input = TtyInput::acquire(PromptCancellation::new())?;
    /// let event = input.next().await?;
    /// input.close().await?;
    /// # Ok::<(), std::io::Error>(()) })?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub async fn close(&mut self) -> io::Result<()> {
        // Close backpressure and interrupt idle retry waits before awaiting terminal restoration.
        self.signal_stop();
        if let Some(done) = self.done.take() {
            // Channel closure also wakes this await if the worker unwinds.
            let _ = done.await;
        }
        // Completion precedes join, keeping normal cleanup from blocking on a live OS poll.
        self.join()
    }

    /// Release queue backpressure and wake an idle send-retry wait before joining.
    fn signal_stop(&mut self) {
        // Set stop before queue closure; both polling and a full send then have an exit path.
        self.stop.store(true, Ordering::Release);
        self.events.close();
        if let Some(worker) = &self.worker {
            // Wake a parked send retry immediately; an in-flight native poll has its finite
            // timeout.
            worker.thread().unpark();
        }
    }

    /// Reap once; all normal callers have observed the worker's completion first.
    fn join(&mut self) -> io::Result<()> {
        // Take the handle before joining, making repeated close/Drop cleanup idempotent.
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| io::Error::other("terminal input worker panicked"))?,
            None => Ok(()),
        }
    }
}

impl PromptInput for TtyInput {
    /// Receive one native event without blocking an async executor thread.
    async fn next(&mut self) -> io::Result<Option<PromptEvent>> {
        // recv is cancellation-safe; transpose preserves native I/O causes instead of fabricating
        // EOF.
        self.events.recv().await.transpose()
    }
}

impl Drop for TtyInput {
    /// Exceptional synchronous cleanup never detaches a blocked stdin reader.
    fn drop(&mut self) {
        // Exceptional cancellation/unwind cannot leave ownership behind in a detached reader.
        self.signal_stop();
        // Drop cannot return failures; TtySession retains the lease if raw restoration fails.
        let _ = self.join();
    }
}

/// Poll only under the exclusive guard, with finite input and send-retry intervals.
fn read_events(
    mut session: TtySession,
    stop: &AtomicBool,
    sender: &mpsc::Sender<io::Result<PromptEvent>>,
    cancellation: &PromptCancellation,
) -> io::Result<()> {
    // No callbacks or user-controlled waits occur here; stop/receiver closure bound the worker
    // loop.
    while !stop.load(Ordering::Acquire) && !sender.is_closed() {
        let next = match session.poll(Duration::from_millis(20)) {
            // Ignore unsupported native events without ever treating their bytes as answer text.
            Ok(Some(event)) => key_event(event),
            Ok(None) => None,
            Err(error) => {
                // Send the original I/O cause to the async driver before restoring ownership.
                send_event(sender, Err(error), stop);
                break;
            }
        };
        if let Some(event) = next {
            // Native cancellation/EOF stop this reader after forwarding their final event.
            let terminal = matches!(event, PromptEvent::Cancel | PromptEvent::Eof);
            if event == PromptEvent::Cancel {
                // Cancellation has its own wakeup path, independent of queue backpressure.
                cancellation.cancel();
            }
            send_event(sender, Ok(event), stop);
            if terminal {
                // Restore the raw guard before advertising completion to the async owner.
                break;
            }
        }
    }
    // Explicit close reports restoration errors; guard Drop retries if this result is an error.
    session.close()
}

/// Retain one bounded event while a full queue awaits consumption, with cancellable backpressure.
fn send_event(
    sender: &mpsc::Sender<io::Result<PromptEvent>>,
    mut event: io::Result<PromptEvent>,
    stop: &AtomicBool,
) {
    // Keep one unsent event locally rather than allocating an unbounded pending-input list.
    while !stop.load(Ordering::Acquire) {
        match sender.try_send(event) {
            // Consumption or queue closure removes the worker's need to retain this event.
            Ok(()) | Err(mpsc::error::TrySendError::Closed(_)) => return,
            // Preserve the exact event while backpressured; stop is rechecked before every retry.
            Err(mpsc::error::TrySendError::Full(pending)) => event = pending,
        }
        // Framework stop unparks this wait; no user code/OS read runs under queue backpressure.
        thread::park_timeout(Duration::from_millis(20));
    }
}

/// Map supported native keys without passing terminal control bytes as answer text.
fn key_event(event: event::Event) -> Option<PromptEvent> {
    use event::{KeyCode, KeyEventKind, KeyModifiers};
    // Resize/mouse/focus notifications are not answer input.
    let event::Event::Key(key) = event else {
        return None;
    };
    if key.kind == KeyEventKind::Release {
        // Release notifications must not duplicate the preceding press/repeat edit.
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        // Recognized control keys become typed events; all other controls remain outside answer
        // text.
        return match key.code {
            KeyCode::Char('c' | 'C') => Some(PromptEvent::Cancel),
            KeyCode::Char('d' | 'D') => Some(PromptEvent::Eof),
            _ => None,
        };
    }
    // Preserve printable scalar values, and expose editing/cancellation as explicit operations.
    match key.code {
        KeyCode::Enter => Some(PromptEvent::Enter),
        KeyCode::Tab => Some(PromptEvent::Tab),
        KeyCode::Backspace => Some(PromptEvent::Backspace),
        KeyCode::Esc => Some(PromptEvent::Cancel),
        KeyCode::Char(character) if !character.is_control() => {
            Some(PromptEvent::Text(character.to_string()))
        }
        _ => None,
    }
}
