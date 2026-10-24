//! Exclusive native terminal ownership for the M0 prompt-guard proof.
//!
//! This module does not spawn input threads or start a prompt. A later driver owns
//! this guard and its polling worker. Only terminal acquisition uses the process-wide
//! lease; host services and async callbacks have no registry/global lock.

use std::io::{self, IsTerminal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossterm::{event, terminal};

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
    pub fn acquire() -> io::Result<Self> {
        // Never start an uncancellable blocking pipe reader under a terminal API.
        if !io::stdin().is_terminal() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "stdin is not a terminal",
            ));
        }
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
            Ok(None)
        }
    }

    /// Restore prior raw mode and release the exclusive lease without reading input.
    ///
    /// # Errors
    /// Returns an OS restoration failure; Drop makes a final best-effort attempt.
    pub fn close(&mut self) -> io::Result<()> {
        // A successful repeated close has no additional terminal side effects.
        if !self.active {
            return Ok(());
        }
        if self.restore_raw_mode {
            terminal::disable_raw_mode()?;
            self.restore_raw_mode = false;
        }
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
