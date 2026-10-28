//! Fixed manual cancellation fixture, selected only by the debug review launcher.
//!
//! This adds no command, permission, mutation or admin operation. The existing
//! read deadline and cancellation registry own the suspended request as usual.

use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

/// Exact public prefix documented beside the manual review controls.
pub const PREFIX: &str = "review-wait";

/// Only the manual launcher opts into a delayed read; ordinary startup is unchanged.
pub fn selected() -> bool {
    std::env::var("PROFILE_UI_MANUAL_REVIEW").as_deref() == Ok("1")
}

/// Register one finite delay after generated registration, before the normal query.
///
/// # Errors
/// Returns the builder's registration failure before any host is constructed.
pub fn register(builder: &mut ServerBuilder) -> Result<()> {
    builder.on("profile-search", delay)?;
    Ok(())
}

/// A deliberately slow, read-only fixture leaves cancellation to the real host.
async fn delay(req: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Other prefixes preserve every generated read and existing finite proof.
    if req.data.path_as::<String>("id")?.as_deref() == Some(PREFIX) {
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    }
    Ok(Flow::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NativeError, NativeHost, register_application};

    /// Observe actual admission, cancel its ticket, then prove ordinary reads still work.
    #[tokio::test]
    async fn fixed_delay_uses_real_cancellation_and_preserves_reads() {
        let mut builder = ServerBuilder::new();
        register_application(&mut builder).unwrap();
        register(&mut builder).unwrap();
        let host = NativeHost::from_server(builder.build().await.unwrap());
        // The manual denial button requires this exact safe error, not any IPC failure.
        assert_eq!(
            host.search(
                "main",
                "review-denial",
                r#"{"revision":1,"id":"abc","event":"shell"}"#
            )
            .await,
            Err(NativeError::Denied)
        );
        assert_eq!(host.in_flight().unwrap(), 0);
        let read = host.search(
            "main",
            "manual-check",
            r#"{"revision":1,"id":"review-wait"}"#,
        );
        tokio::pin!(read);
        // Poll into the real lifecycle: completion before cancellation would fail.
        tokio::select! {
            result = &mut read => panic!("delay completed early: {result:?}"),
            _ = tokio::time::sleep(std::time::Duration::from_millis(25)) => {}
        }
        assert_eq!(host.in_flight().unwrap(), 1);
        host.cancel("main", "manual-check").unwrap();
        assert_eq!(read.await, Err(NativeError::Cancelled));
        assert_eq!(host.in_flight().unwrap(), 0);
        assert!(
            host.search("main", "normal", r#"{"revision":1,"id":"abc"}"#)
                .await
                .unwrap()
                .contains("Ada")
        );
        assert_eq!(
            host.search("ungranted", "denied", r#"{"revision":1,"id":"abc"}"#)
                .await,
            Err(NativeError::Denied)
        );
        host.shutdown().await.unwrap();
    }
}
