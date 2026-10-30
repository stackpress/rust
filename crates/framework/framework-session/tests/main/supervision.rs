//! Process-external watchdog: a blocked runtime cannot block the parent's deadline.
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

#[test]
fn deadlock_class_proofs_are_externally_bounded() {
    run_child("real", Duration::from_secs(15), false);
    // Prove the supervisor detects and kills a permanently blocked child itself.
    run_child("blocked", Duration::from_millis(200), true);
}
/// Own the child and always kill/reap before reporting a timeout or missing entry ack.
fn run_child(mode: &str, deadline: Duration, expect_timeout: bool) {
    // The absolute deadline includes startup/entry acknowledgement, not just runtime work.
    let until = Instant::now() + deadline;
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "supervision::child", "--ignored", "--nocapture"])
        .env("S9_CHILD", mode)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if line.unwrap().contains("S9_ENTERED") {
                let _ = tx.send(());
            }
        }
    });
    if rx.recv_timeout(Duration::from_secs(2)).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        reader.join().unwrap();
        panic!("child failed to acknowledge entry");
    }
    let timed_out = loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break false;
        }
        if Instant::now() >= until {
            child.kill().unwrap();
            child.wait().unwrap();
            break true;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    reader.join().unwrap();
    assert_eq!(timed_out, expect_timeout);
}
#[test]
#[ignore = "entered only by the externally supervised parent"]
fn child() {
    println!("S9_ENTERED");
    std::io::stdout().flush().unwrap();
    if std::env::var("S9_CHILD").as_deref() == Ok("blocked") {
        loop {
            std::thread::park();
        }
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        #[cfg(feature = "sql")]
        super::actions::transaction_child().await;
        #[cfg(feature = "auth-server")]
        super::auth_flows::concurrency_child().await;
        use stackpress_framework_session::Service;
        use std::sync::Arc;
        // Reentrant host code must run after settings locks have been released.
        struct Reentrant(std::sync::Mutex<Option<Service>>);
        impl stackpress_framework_session::Clock for Reentrant {
            fn now(&self) -> f64 {
                if let Some(service) = self.0.lock().unwrap().as_ref() {
                    service.configure(service.config());
                }
                1_700_000_000.0
            }
        }
        let clock = Arc::new(Reentrant(std::sync::Mutex::new(None)));
        let reentrant = Service::with_clock(Default::default(), clock.clone());
        *clock.0.lock().unwrap() = Some(reentrant.clone());
        reentrant.create(Default::default()).await.unwrap();
        // Break the deliberate test ownership cycle before child shutdown.
        clock.0.lock().unwrap().take();
        let service = Service::default();
        let token = service.create(Default::default()).await.unwrap();
        let barrier = Arc::new(tokio::sync::Barrier::new(16));
        let identity = Arc::new(service.load(token));
        let mut jobs = Vec::new();
        for _ in 0..16 {
            let barrier = barrier.clone();
            let identity = identity.clone();
            jobs.push(tokio::spawn(async move {
                barrier.wait().await;
                assert!(!identity.guest().await);
            }));
        }
        for job in jobs {
            job.await.unwrap();
        }
    });
}
