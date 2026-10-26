//! Shared admission/shutdown loop; injected accepts let tests use only duplex streams.

use crate::Gateway;
use std::{future::Future, io, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{Semaphore, watch},
    task::JoinSet,
    time::timeout,
};

impl Gateway {
    /// Own accepted connections until completion, admission rejection or bounded shutdown.
    ///
    /// The executable supplies TCP accepts; tests supply ready duplex streams.
    /// End-of-input, accept failure and the stop future all stop new admissions,
    /// signal graceful HTTP shutdown, then abort/reap any work past its drain limit.
    ///
    /// # Errors
    /// Preserves accept/stop errors after cleaning up already admitted connections.
    pub async fn run<A, F, I, S>(self: Arc<Self>, mut accept: A, stop: S) -> io::Result<()>
    where
        A: FnMut() -> F,
        F: Future<Output = io::Result<Option<I>>>,
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        S: Future<Output = io::Result<()>>,
    {
        let slots = Arc::new(Semaphore::new(self.limits.connections));
        let (shutdown, signal) = watch::channel(false);
        let mut tasks = JoinSet::new();
        tokio::pin!(stop);
        let result = loop {
            tokio::select! {
                result=&mut stop=>break result,
                Some(_)=tasks.join_next(),if !tasks.is_empty()=>{},
                accepted=accept()=>match accepted {
                    Ok(Some(io))=>{
                        // Keep the permit in the owned task; excess IO is dropped immediately.
                        let Ok(slot)=Arc::clone(&slots).try_acquire_owned() else {drop(io);continue;};
                        let host=Arc::clone(&self);let signal=signal.clone();
                        tasks.spawn(async move {let _slot=slot;host.serve(io,signal).await});
                    }
                    Ok(None)=>break Ok(()),
                    Err(error)=>break Err(error),
                }
            }
        };
        let _ = shutdown.send(true);
        let drained = async { while tasks.join_next().await.is_some() {} };
        if timeout(
            self.limits.drain_timeout + std::time::Duration::from_secs(1),
            drained,
        )
        .await
        .is_err()
        {
            // JoinSet shutdown aborts and reaps every task; no detached work remains.
            tasks.shutdown().await;
        }
        debug_assert_eq!(self.active_connections(), 0);
        result
    }
}
