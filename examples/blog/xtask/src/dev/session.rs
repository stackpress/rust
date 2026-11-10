//! Own successful build publication, deferred CSR, and the replaceable app process group.
use super::{
    Options,
    build::{Builder, Csr, server_command},
    change::Work,
    reload::Notice,
};
use crate::{BuildResult, OwnedChild};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

/// A successful build is the only path to replacing the active owned process.
#[derive(Default)]
pub(super) struct Session {
    /// The currently running executable's process group.
    child: Option<OwnedChild>,
    /// Immutable last-good executable, also reused for config-only restarts.
    executable: Option<PathBuf>,
    /// A restart generation lets the reload worker identify HTML loaded during compilation.
    pub generation: u64,
    /// A superseding batch cancels and joins this worker before starting other compilers.
    csr: Option<Csr>,
    /// Remains true after a failed deferred build; the next successful edit retries it.
    csr_due: bool,
    /// Forward child stdout and join after the owned process group has closed its pipe.
    output: Option<std::thread::JoinHandle<()>>,
    /// Ctrl-C cancels readiness waits as well as compiler subprocesses.
    stop: Arc<AtomicBool>,
}

impl Session {
    /// Share Ctrl-C cancellation with the loop and its compiler/reload workers.
    pub fn new(stop: Arc<AtomicBool>) -> Self {
        let mut session = Self::default();
        session.stop.clone_from(&stop);
        session
    }

    /// Complete hydration/native before replacing the app; CSS-only batches never compile.
    pub fn update(
        &mut self,
        root: &Path,
        options: &Options,
        builder: &mut Builder,
        work: &Work,
        run: impl Fn(&mut Command, Duration) -> BuildResult + Sync,
    ) -> Result<Vec<Notice>, Box<dyn std::error::Error>> {
        if work.browser || work.restart {
            // Prioritize new edits: stop the old background group before two critical builds.
            drop(self.csr.take());
            // A canceled or failed CSR candidate remains due until a successful retry.
            self.csr_due |= work.browser;
        }
        let prepared = builder.prepare(work, run)?;
        prepared.publish(root)?;
        if work.restart {
            let executable = prepared
                .executable
                .as_ref()
                .or(self.executable.as_ref())
                .ok_or("no built server to restart")?
                .clone();
            self.replace(&mut server_command(root, &executable, options), executable)?;
        }
        if work.browser || work.restart {
            return Ok(vec![self.notice()]);
        }
        Ok(work.css.iter().cloned().map(Notice::Css).collect())
    }

    /// CSR pages keep their last bundle until the deferred build is complete.
    pub fn notice(&self) -> Notice {
        if self.csr_due {
            Notice::Hydrate
        } else {
            Notice::All
        }
    }

    /// Run CSR only after sending the hydration/native completion notice.
    pub fn defer_csr(&mut self, builder: &Builder, stop: Arc<AtomicBool>) {
        if self.csr_due && self.csr.is_none() {
            self.csr = Some(Csr::start(builder.clone(), stop));
        }
    }

    /// Publish a complete background bundle; failures leave the old CSR bundle available.
    pub fn finish_csr(&mut self, root: &Path) -> Option<Notice> {
        if !self.csr.as_ref().is_some_and(Csr::finished) {
            return None;
        }
        let result = self
            .csr
            .take()?
            .finish()
            .and_then(|prepared| prepared.publish(root));
        match result {
            Ok(()) => {
                self.csr_due = false;
                Some(Notice::Csr)
            }
            Err(error) => {
                eprintln!("dev CSR build failed; last good bundle retained: {error}");
                None
            }
        }
    }

    /// Stop/reap the previous group before spawning a replacement on the same app port.
    fn replace(&mut self, command: &mut Command, executable: PathBuf) -> BuildResult {
        if self.stop.load(Ordering::Relaxed) {
            return Err("dev restart canceled".into());
        }
        if let Some(mut previous) = self.child.take() {
            previous.stop();
        }
        if let Some(output) = self.output.take() {
            let _ = output.join();
        }
        self.generation += 1;
        command.env("BLOG_DEV_BUILD_ID", self.generation.to_string());
        // The app prints readiness after binding. Wait for that line, never probe a port.
        let ready = command
            .get_envs()
            .any(|(key, _)| key == "BLOG_DEV_RELOAD_PORT");
        if ready {
            command.stdout(std::process::Stdio::piped());
        }
        let mut child = OwnedChild::spawn(command)?;
        if ready {
            let stdout = child.child.stdout.take().ok_or("missing app stdout")?;
            let (send, receive) = std::sync::mpsc::channel();
            let output = std::thread::spawn(move || {
                use std::io::BufRead;
                for line in std::io::BufReader::new(stdout).lines() {
                    let Ok(line) = line else {
                        break;
                    };
                    println!("{line}");
                    if line.starts_with("ready http://localhost:") {
                        let _ = send.send(());
                    }
                }
            });
            let deadline = Instant::now() + Duration::from_secs(30);
            let result = loop {
                if self.stop.load(Ordering::Relaxed) || Instant::now() >= deadline {
                    break Err("canceled or readiness deadline elapsed".to_owned());
                }
                match receive.recv_timeout(Duration::from_millis(50)) {
                    Ok(()) => break Ok(()),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(error) => break Err(error.to_string()),
                }
            };
            if let Err(error) = result {
                child.terminate();
                let _ = output.join();
                return Err(format!("dev app did not become ready: {error}").into());
            }
            self.output = Some(output);
        }
        if let Some(previous) = self.executable.replace(executable) {
            // Config restarts reuse the same immutable candidate; never unlink that one.
            if self.executable.as_ref() != Some(&previous) {
                std::fs::remove_file(previous)?;
            }
        }
        self.child = Some(child);
        Ok(())
    }

    /// Surface unexpected app exits once; later edits can rebuild/restart it.
    pub fn check_exit(&mut self) -> BuildResult {
        if let Some(child) = &mut self.child
            && let Some(status) = child.child.try_wait()?
        {
            eprintln!("dev server exited: {status}; waiting for a source/config change");
            child.terminate();
            self.child = None;
        }
        Ok(())
    }
}

impl Drop for Session {
    /// Ctrl-C and all error/unwind paths stop and reap only this session's app group.
    fn drop(&mut self) {
        drop(self.csr.take());
        if let Some(child) = &mut self.child {
            child.stop();
        }
        if let Some(output) = self.output.take() {
            let _ = output.join();
        }
        if let Some(path) = &self.executable {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/cases/session.rs"]
mod tests;
