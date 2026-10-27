//! Parent-owned SQLite reference file, with the same kill/reap policy as network cases.
use super::{TestResult, contract, watchdog};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Only successful create_new grants cleanup authority over this exact file and its sidecars.
struct FileFixture {
    /// Exact create_new-reserved path; never derived from an existing application database.
    path: PathBuf,
    /// Cleanup authority is armed only after create_new and cleared only after removal.
    armed: bool,
}
impl FileFixture {
    /// Reserve a unique file without adopting any pre-existing application database.
    fn create() -> TestResult<Self> {
        let path = std::env::temp_dir().join(format!(
            "stackpress_s7_m2_sqlite_{:x}_{:x}.sqlite",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        drop(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?,
        );
        println!("created owned SQLite file {}", path.display());
        Ok(Self { path, armed: true })
    }
    /// Reap precedes an independently bounded cleanup; no own worker can retain this file.
    fn cleanup(&mut self) -> TestResult<()> {
        if !self.armed {
            return Ok(());
        }
        let path = self.path.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let _ = sender.send(remove_owned(path));
        });
        let result = receiver
            .recv_timeout(Duration::from_secs(10))
            .map_err(|_| "SQLite file cleanup deadline exceeded")?;
        worker
            .join()
            .map_err(|_| "SQLite file cleanup worker panicked")?;
        result?;
        self.armed = false;
        println!(
            "removed owned SQLite file and sidecars {}",
            self.path.display()
        );
        Ok(())
    }
}
/// Remove only the main file and exact SQLite sidecars reserved by this parent.
fn remove_owned(path: PathBuf) -> TestResult<()> {
    for suffix in ["-wal", "-shm", ""] {
        let mut target = path.as_os_str().to_os_string();
        target.push(suffix);
        match std::fs::remove_file(PathBuf::from(target)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

impl Drop for FileFixture {
    /// Unwind cleanup failure remains visible to the test process exit status.
    fn drop(&mut self) {
        if self.armed && self.cleanup().is_err() {
            eprintln!("SQLite cleanup FAILED: {}", self.path.display());
            super::CLEANUP_FAILED.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
}
/// Execute the shared suite on the parent's reserved file; close is part of that suite.
pub(super) async fn child(path: PathBuf) -> TestResult<()> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid SQLite reference path")?;
    if path.parent() != Some(std::env::temp_dir().as_path())
        || !name.starts_with("stackpress_s7_m2_sqlite_")
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.'))
    {
        return Err("invalid SQLite reference path".into());
    }
    let mut options = SqliteOptions::file(path);
    options.max_connections = 1;
    options.max_in_flight = 8;
    // Values/transaction assertions are not acquisition-latency benchmarks. Use the
    // adapter's bounded public defaults; the parent still enforces 30s independently.
    contract::run_suite(Arc::new(SqliteDatabase::open(options).await?)).await?;
    Ok(())
}
/// Run offline even when the PostgreSQL gate is unset; a timeout is always a failed proof.
pub(super) fn parent() -> TestResult<()> {
    let mut fixture = FileFixture::create()?;
    let status = watchdog(
        Command::new(std::env::current_exe()?)
            .arg("--m2-sqlite")
            .arg(&fixture.path)
            .spawn()?,
        Duration::from_secs(30),
    );
    let cleanup = fixture.cleanup();
    if !status?.success() {
        return Err("SQLite reference contract failed".into());
    }
    cleanup
}
