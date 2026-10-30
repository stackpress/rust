//! Explicit channels establish ordering; no proof relies on sleeps or scheduler load.

use stackpress_framework_email::tasks::TaskSupervisor;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{runtime::Handle, sync::oneshot};

/// Mark future destruction, including cancellation after its first poll.
struct DropProbe(Arc<AtomicUsize>);
impl Drop for DropProbe {
    /// Increment only when owned future state is actually released.
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Parent completion does not await the hook; the executor still polls it.
#[tokio::test]
async fn nonawaited_hook_executes_with_independent_owned_payload() {
    let mut tasks = TaskSupervisor::new();
    let (entered, observed) = oneshot::channel();
    let (release, wait) = oneshot::channel();
    let parent = String::from("original");
    let mut child = parent.clone();
    tasks.spawn_on(&Handle::current(), async move {
        child.push_str(" child");
        entered.send(child).unwrap();
        wait.await?;
        Ok(())
    });
    assert_eq!(parent, "original");
    assert!(!tasks.is_empty());
    assert_eq!(observed.await.unwrap(), "original child");
    release.send(()).unwrap();
    tasks.join_next().await.unwrap().unwrap().unwrap();
    assert!(tasks.is_empty());
    assert!(tasks.join_next().await.is_none());
}

/// Background failures remain visible to the runtime owner without changing parent output.
#[tokio::test]
async fn error_and_panic_are_retained_for_supervisor() {
    let mut tasks = TaskSupervisor::new();
    tasks.spawn_on(&Handle::current(), async {
        Err(io::Error::other("hook failed").into())
    });
    assert_eq!(
        tasks
            .join_next()
            .await
            .unwrap()
            .unwrap()
            .unwrap_err()
            .to_string(),
        "hook failed"
    );
    tasks.spawn_on(&Handle::current(), async { panic!("fixture panic") });
    assert!(tasks.join_next().await.unwrap().unwrap_err().is_panic());
}

/// Shutdown aborts and joins every future before returning to the lifecycle owner.
#[tokio::test]
async fn shutdown_reaps_started_and_unpolled_jobs() {
    let mut tasks = TaskSupervisor::new();
    let dropped = Arc::new(AtomicUsize::new(0));
    let (entered, observed) = oneshot::channel();
    let probe = DropProbe(dropped.clone());
    tasks.spawn_on(&Handle::current(), async move {
        let _probe = probe;
        entered.send(()).unwrap();
        std::future::pending::<()>().await;
        Ok(())
    });
    observed.await.unwrap();
    let probe = DropProbe(dropped.clone());
    tasks.spawn_on(&Handle::current(), async move {
        let _probe = probe;
        std::future::pending::<()>().await;
        Ok(())
    });
    tasks.shutdown().await;
    assert_eq!(dropped.load(Ordering::SeqCst), 2);
    assert!(tasks.is_empty());
}

/// Dropping the owner requests abort instead of detaching unfinished work.
#[tokio::test]
async fn drop_aborts_and_releases_owned_future() {
    let mut tasks = TaskSupervisor::new();
    let (entered, observed) = oneshot::channel();
    let (destroyed, done) = oneshot::channel();
    struct NotifyDrop(Option<oneshot::Sender<()>>);
    impl Drop for NotifyDrop {
        /// A destruction channel gives an explicit completion barrier after owner drop.
        fn drop(&mut self) {
            if let Some(tx) = self.0.take() {
                let _ = tx.send(());
            }
        }
    }
    tasks.spawn_on(&Handle::current(), async move {
        let _probe = NotifyDrop(Some(destroyed));
        entered.send(()).unwrap();
        std::future::pending::<()>().await;
        Ok(())
    });
    observed.await.unwrap();
    drop(tasks);
    done.await.unwrap();
}
