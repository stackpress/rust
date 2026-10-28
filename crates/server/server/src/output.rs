//! Adapter-owned output authority, independent of replaceable handler Response values.

use std::future::Future;

use stackpress_lib::{DeliveryState, Error, Response, ResponseView, Result};

/// One native output authority retained privately by an adapter, never given to handlers.
///
/// A handler can replace its entire Response with a fresh value; that cannot reset
/// this separate guard. Arbitrary caller-owned writers are outside this guarantee.
pub struct OutputGuard {
    /// Attempt state survives Response replacement and cancelled delivery futures.
    state: DeliveryState,
}

impl Default for OutputGuard {
    /// Start with one unused native output authority.
    fn default() -> Self {
        // No native handle or global output lock is acquired during construction.
        Self {
            state: DeliveryState::Open,
        }
    }
}

impl OutputGuard {
    /// Inspect adapter history, independently of the current Response's delivery state.
    pub fn state(&self) -> DeliveryState {
        // Returning a copy cannot grant permission to reset or repeat native output.
        self.state
    }

    /// Attempt one adapter-owned writer, or honor explicit response suppression.
    ///
    /// # Errors
    /// Rejects repeated/failed/incomplete authority or a Response already finalized
    /// elsewhere. Writer failure may follow partial output and is never retried.
    pub async fn dispatch<F, Fut>(&mut self, response: &mut Response, writer: F) -> Result<()>
    where
        F: FnOnce(ResponseView) -> Fut + Send,
        Fut: Future<Output = Result<()>> + Send,
    {
        // Check adapter history first; a fresh Response cannot grant a second write.
        if self.state != DeliveryState::Open {
            return Err(Error::new(
                "native output authority has already been consumed",
            ));
        }
        match response.delivery_state() {
            // Only unused value-level delivery may proceed to this adapter's native attempt.
            DeliveryState::Open => {}
            DeliveryState::Stopped => {
                // Suppression consumes this adapter's output opportunity without entering writer.
                self.state = DeliveryState::Stopped;
                return Ok(());
            }
            _ => {
                // A terminal adapter-state failure cannot be repaired by replacing Response.
                self.state = DeliveryState::Failed;
                return Err(Error::new(
                    "response finalized outside this native output authority",
                ));
            }
        }
        // Consume before any await; dropping a pending future cannot reopen authority.
        self.state = DeliveryState::Attempted;
        // The sole writer owns this await; cancellation preserves Attempted in both owners.
        let result = response.dispatch_with(writer).await;
        // Record completion without granting a retry after partial failure.
        self.state = if result.is_ok() {
            DeliveryState::Sent
        } else {
            DeliveryState::Failed
        };
        result
    }
}
