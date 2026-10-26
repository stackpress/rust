//! Fixed local read operation with scoped admission, cancellation and one lifecycle.
//!
//! The registry owns only cancellation senders. No detached task owns a request,
//! response or server, and cancel means losing a result rather than rollback.

use stackpress_example_profile_ui_contracts::{
    HostError, MAX_REQUEST_BYTES, SearchInput, SearchReply,
};
use stackpress_lib::{Request, Response};
use stackpress_server::{HandleOptions, Server};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::watch;

/// Maximum simultaneous reads per trusted window.
const MAX_CALLS: usize = 32;
/// A read must finish or cancel within this deadline.
const DEADLINE: Duration = Duration::from_secs(5);

/// Safe adapter failures, independent of developer diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeError {
    /// Caller/window or command input is not admitted.
    Denied,
    /// Duplicate ID, closed host or finite capacity reached.
    Busy,
    /// Work was cancelled before producing a result.
    Cancelled,
    /// Read exceeded its finite deadline.
    Deadline,
    /// Native lifecycle, poisoned state or public projection failed.
    Unavailable,
}

impl std::fmt::Display for NativeError {
    /// Keep native causes out of IPC error strings.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Denied => "Search denied",
            Self::Busy => "Search busy",
            Self::Cancelled => "Search cancelled",
            Self::Deadline => "Search timed out",
            Self::Unavailable => "Search unavailable",
        })
    }
}
impl std::error::Error for NativeError {}

/// Mutable cancellation metadata only; the mutex is never held across await.
#[derive(Default)]
struct Registry {
    /// Shutdown prevents admission before signalling all admitted calls.
    closed: bool,
    /// IDs are qualified by the authoritative native window identity.
    calls: HashMap<(String, String), watch::Sender<bool>>,
}

/// Immutable native host plus explicitly shared cancellation state.
pub struct NativeHost {
    /// Same generated registration used by CLI, HTTP and the page loader.
    server: Server,
    /// No entry retains this host or a request; the ownership graph is acyclic.
    registry: Arc<Mutex<Registry>>,
}

impl NativeHost {
    /// Build the generated read-only application unchanged.
    ///
    /// # Errors
    /// Returns normal generated startup/configuration failures.
    pub async fn new() -> stackpress_server::BuildResult<Self> {
        let server =
            stackpress_example_generated_profile::build_application(Default::default()).await?;
        Ok(Self::from_server(server))
    }

    /// Attach an already-built application, preserving its lifecycle/plugin policies.
    pub fn from_server(server: Server) -> Self {
        Self {
            server,
            registry: Arc::new(Mutex::new(Registry::default())),
        }
    }

    /// Admit only a fixed typed search from the trusted native window.
    ///
    /// Input is checked before domain execution. Cancellation/deadline drops the
    /// borrowing lifecycle future; no retry or detached domain task is created.
    ///
    /// # Errors
    /// Rejects denied/malformed input, bounded admission and native execution failures.
    pub async fn search(
        &self,
        window: &str,
        call_id: &str,
        json: &str,
    ) -> Result<String, NativeError> {
        validate_identity(window, call_id)?;
        if json.len() > MAX_REQUEST_BYTES {
            return Err(NativeError::Denied);
        }
        let input: SearchInput = serde_json::from_str(json).map_err(|_| NativeError::Denied)?;
        input.validate().map_err(|_| NativeError::Denied)?;
        let mut ticket = self.admit(window, call_id)?;
        // Biased order makes a ready cancellation win over a simultaneously ready result.
        tokio::select! {
            biased;
            _ = ticket.cancelled.changed() => Err(NativeError::Cancelled),
            _ = tokio::time::sleep(DEADLINE) => Err(NativeError::Deadline),
            reply = self.run(input) => reply,
        }
    }

    /// Execute precisely one top-level lifecycle and project only public result data.
    async fn run(&self, input: SearchInput) -> Result<String, NativeError> {
        let mut request = Request::builder()
            .data(serde_json::json!({"id": input.id}))
            .build()
            .map_err(|_| NativeError::Unavailable)?;
        let mut response = Response::new();
        let report = self
            .server
            .handle_with_options(
                "profile-search",
                &mut request,
                &mut response,
                HandleOptions::default(),
            )
            .await
            .map_err(|_| NativeError::Unavailable)?;
        if !report.is_success(&response) {
            // A completed lifecycle rejection is public response data, not bridge failure.
            let envelope = response
                .try_to_json()
                .map_err(|_| NativeError::Unavailable)?;
            let error =
                HostError::from_envelope(&envelope).map_err(|_| NativeError::Unavailable)?;
            return serde_json::to_string(&error).map_err(|_| NativeError::Unavailable);
        }
        let reply = SearchReply::from_envelope(
            &response
                .try_to_json()
                .map_err(|_| NativeError::Unavailable)?,
        )
        .map_err(|_| NativeError::Unavailable)?;
        serde_json::to_string(&reply).map_err(|_| NativeError::Unavailable)
    }

    /// Insert one cancellation ticket after checking capacity and duplicate IDs.
    fn admit(&self, window: &str, call_id: &str) -> Result<Ticket, NativeError> {
        let mut registry = self.registry.lock().map_err(|_| NativeError::Unavailable)?;
        let key = (window.to_owned(), call_id.to_owned());
        if registry.closed || registry.calls.len() >= MAX_CALLS || registry.calls.contains_key(&key)
        {
            return Err(NativeError::Busy);
        }
        let (sender, cancelled) = watch::channel(false);
        registry.calls.insert(key.clone(), sender);
        Ok(Ticket {
            registry: Arc::clone(&self.registry),
            key,
            cancelled,
        })
    }

    /// Cancel only an ID in the originating trusted window; completed IDs are harmless.
    ///
    /// # Errors
    /// Rejects untrusted identities and poisoned state without running a handler.
    pub fn cancel(&self, window: &str, call_id: &str) -> Result<(), NativeError> {
        validate_identity(window, call_id)?;
        let registry = self.registry.lock().map_err(|_| NativeError::Unavailable)?;
        if let Some(sender) = registry.calls.get(&(window.to_owned(), call_id.to_owned())) {
            let _ = sender.send(true);
        }
        Ok(())
    }

    /// Close admission, then cancel all owned work; callers await their own futures.
    ///
    /// # Errors
    /// Returns a safe failure for poisoned state; poison is never silently cleared.
    pub fn close(&self) -> Result<(), NativeError> {
        let mut registry = self.registry.lock().map_err(|_| NativeError::Unavailable)?;
        registry.closed = true;
        for sender in registry.calls.values() {
            let _ = sender.send(true);
        }
        Ok(())
    }

    /// Inspect bounded in-flight metadata without exposing request contents.
    ///
    /// # Errors
    /// Returns a safe failure for poisoned state.
    pub fn in_flight(&self) -> Result<usize, NativeError> {
        Ok(self
            .registry
            .lock()
            .map_err(|_| NativeError::Unavailable)?
            .calls
            .len())
    }
}

/// Validate authoritative window identity and the caller-owned opaque ID.
fn validate_identity(window: &str, call_id: &str) -> Result<(), NativeError> {
    if window != "main"
        || call_id.is_empty()
        || call_id.len() > 64
        || !call_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(NativeError::Denied);
    }
    Ok(())
}

/// RAII removes metadata on completion, cancellation, timeout or future disposal.
struct Ticket {
    /// Shared metadata has no edge back to NativeHost.
    registry: Arc<Mutex<Registry>>,
    /// Admission key to remove exactly once.
    key: (String, String),
    /// Receiver is exclusively owned by the borrowing native search future.
    cancelled: watch::Receiver<bool>,
}

impl Drop for Ticket {
    /// Poison remains an error for later callers; cleanup never panics during unwind.
    fn drop(&mut self) {
        if let Ok(mut registry) = self.registry.lock() {
            registry.calls.remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests;
