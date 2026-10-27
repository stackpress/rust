//! Fixed local read operation with scoped admission, cancellation and one lifecycle.
//!
//! The registry owns only cancellation senders. No detached task owns a request,
//! response or server, and cancel means losing a result rather than rollback.

use stackpress_example_profile_ui_contracts::{
    HostError, MAX_REQUEST_BYTES, SearchInput, SearchReply,
};
use stackpress_lib::{DeliveryState, Request, Response};
use stackpress_server::{HandleOptions, HandleReport, OutputGuard, Server};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{Notify, watch};

// Compile the same optional plugin source used by the direct/CLI/HTTP/SSR hosts.
#[cfg(feature = "demo-customization")]
#[path = "../../customization.rs"]
mod customization;

/// Register generated operations followed by the optional shared labels plugin.
///
/// # Errors
/// Returns registration failures without building a partial application.
pub fn register_application(
    builder: &mut stackpress_server::ServerBuilder,
) -> stackpress_lib::Result<()> {
    stackpress_example_generated_profile::generated::register(builder)?;
    #[cfg(feature = "demo-customization")]
    builder.plugin("profile-ui-labels", customization::register)?;
    Ok(())
}

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
    /// The lifecycle deliberately returned no typed search content.
    NoContent,
    /// The lifecycle explicitly suppressed native output.
    Suppressed,
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
            Self::NoContent => "Search returned no content",
            Self::Suppressed => "Search output suppressed",
            Self::Unavailable => "Search unavailable",
        })
    }
}
impl std::error::Error for NativeError {}

impl NativeError {
    /// Stable safe bridge token; native diagnostics are never used as IPC errors.
    pub fn code(self) -> &'static str {
        match self {
            Self::Denied => "denied",
            Self::Busy => "busy",
            Self::Cancelled => "cancelled",
            Self::Deadline => "deadline",
            Self::NoContent => "no-content",
            Self::Suppressed => "suppressed",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Check native WebView metadata before input can enter the selected operation.
///
/// # Errors
/// Rejects other windows, origins and schemes. These values must come from the
/// authoritative native WebView, never from fields supplied to invoke().
pub fn validate_caller(window: &str, scheme: &str, host: Option<&str>) -> Result<(), NativeError> {
    if window == "main" && scheme == "tauri" && host == Some("localhost") {
        Ok(())
    } else {
        Err(NativeError::Denied)
    }
}

/// Authoritative adapter metadata inherited by raw nested dispatch, never caller data.
#[derive(Debug)]
pub struct NativeCallContext {
    /// Checked native window label.
    pub window: String,
    /// Checked opaque ID belonging to that native window.
    pub call_id: String,
}

/// Mutable cancellation metadata only; the mutex is never held across await.
#[derive(Default)]
struct Registry {
    /// Shutdown prevents admission before signalling all admitted calls.
    closed: bool,
    /// IDs are qualified by the authoritative native window identity.
    calls: HashMap<(String, String), watch::Sender<bool>>,
}

/// Immutable native host plus explicitly shared cancellation state.
#[derive(Clone)]
pub struct NativeHost {
    /// Same generated registration used by CLI, HTTP and the page loader.
    server: Arc<Server>,
    /// No entry retains this host or a request; the ownership graph is acyclic.
    registry: Arc<Mutex<Registry>>,
    /// Ticket disposal wakes shutdown after the actual borrowing future is dropped.
    drained: Arc<Notify>,
    /// Trusted finite execution policy; production uses the five-second deadline.
    deadline: Duration,
}

impl NativeHost {
    /// Build the generated read-only application with the selected shared customization.
    ///
    /// # Errors
    /// Returns normal generated startup/configuration failures.
    pub async fn new() -> stackpress_server::BuildResult<Self> {
        let mut builder = stackpress_server::ServerBuilder::new();
        register_application(&mut builder)?;
        let server = builder.build().await?;
        Ok(Self::from_server(server))
    }

    /// Attach an already-built application, preserving its lifecycle/plugin policies.
    pub fn from_server(server: Server) -> Self {
        Self {
            server: Arc::new(server),
            registry: Arc::new(Mutex::new(Registry::default())),
            drained: Arc::new(Notify::new()),
            deadline: DEADLINE,
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
            _ = tokio::time::sleep(self.deadline) => Err(NativeError::Deadline),
            reply = self.run(input, NativeCallContext { window: window.into(), call_id: call_id.into() }) => reply,
        }
    }

    /// Execute precisely one top-level lifecycle and project only public result data.
    async fn run(
        &self,
        input: SearchInput,
        context: NativeCallContext,
    ) -> Result<String, NativeError> {
        let mut request = Request::builder()
            .data(serde_json::json!({"id": input.id}))
            .build()
            .map_err(|_| NativeError::Unavailable)?;
        let mut response = Response::new();
        let mut report = self
            .server
            .handle_with_options(
                "profile-search",
                &mut request,
                &mut response,
                HandleOptions {
                    adapter_context: Some(Arc::new(context)),
                },
            )
            .await
            .map_err(|_| NativeError::Unavailable)?;
        // Adapter authority is independent of a handler replacing its entire Response.
        let mut guard = OutputGuard::default();
        if response.delivery_state() != DeliveryState::Open {
            guard
                .dispatch(&mut response, |_| async { Ok(()) })
                .await
                .map_err(|_| NativeError::Unavailable)?;
            report.record_output(&response);
            return Err(NativeError::Suppressed);
        }
        let text = project(&response, &report)?;
        let mut published = None;
        guard
            .dispatch(&mut response, |_| async {
                published = Some(text);
                Ok(())
            })
            .await
            .map_err(|_| NativeError::Unavailable)?;
        report.record_output(&response);
        published.ok_or(NativeError::Suppressed)
    }

    /// Close admission, cancel all calls and await actual ticket/future disposal.
    ///
    /// # Errors
    /// Preserves poisoned-state failures or a bounded shutdown deadline. No success
    /// is reported while registry entries remain, and no detached domain task exists.
    pub async fn shutdown(&self) -> Result<(), NativeError> {
        self.close()?;
        let deadline = tokio::time::Instant::now() + DEADLINE;
        loop {
            // Register before inspecting the map so a last-ticket wake cannot be lost.
            let notified = self.drained.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.in_flight()? == 0 {
                return Ok(());
            }
            tokio::time::timeout_at(deadline, notified)
                .await
                .map_err(|_| NativeError::Deadline)?;
        }
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
            drained: Arc::clone(&self.drained),
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

    /// Close admission before signalling all owned calls. Use shutdown to await disposal.
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
}

/// Project the chosen final public response; dispatch outcome alone cannot decide success.
fn project(response: &Response, report: &HandleReport) -> Result<String, NativeError> {
    if response.status().is_some_and(|status| status.code == 204) {
        return Err(NativeError::NoContent);
    }
    if !report.is_success(response) {
        // A completed lifecycle rejection is public response data, not bridge failure.
        let envelope = response
            .try_to_json()
            .map_err(|_| NativeError::Unavailable)?;
        let error = HostError::from_envelope(&envelope).map_err(|_| NativeError::Unavailable)?;
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
    /// Shared completion notification, with no ownership edge back to the host.
    drained: Arc<Notify>,
}

impl Drop for Ticket {
    /// Poison remains an error for later callers; cleanup never panics during unwind.
    fn drop(&mut self) {
        // Even during unwind, discard this ticket's metadata without clearing poison.
        // Operational APIs still return Unavailable rather than retrying poisoned state.
        let mut registry = self
            .registry
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        registry.calls.remove(&self.key);
        drop(registry);
        self.drained.notify_waiters();
    }
}

#[cfg(all(unix, any(feature = "app", test)))]
mod preferences;

// Platform app APIs stay optional; native host/controller tests do not load Tauri.
#[cfg(feature = "app")]
mod app;
#[cfg(feature = "app")]
pub use app::run;

#[cfg(test)]
mod tests;
