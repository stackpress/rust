//! Blog development orchestration; only the explicit `dev` command opens loopback listeners.

mod build;
mod change;
mod markup;
mod reload;

use crate::{BuildResult, OwnedChild, bounded_until};
use build::{Builder, server_command};
use change::{Debounce, Snapshot, Work, classify};
use markup::Markup;
use reload::{Notice, Reload};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

/// Explicit local ports for the app and the independently owned reload channel.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    /// Application HTTP port; defaults to 3000.
    pub port: u16,
    /// Reload WebSocket port; defaults to the application port plus one.
    pub reload_port: u16,
}

/// Run the explicitly requested loop until Ctrl-C, retaining the last good app after build errors.
///
/// Opens two IPv4 loopback listeners (app and reload); callers must explicitly authorize it.
/// # Errors
/// Invalid source/output paths, signal registration and reload listener failures are returned.
pub fn run(root: &Path, native: Option<&Path>, options: Options) -> BuildResult {
    let stop = Arc::new(AtomicBool::new(false));
    let signals = Signals::register(stop.clone())?;
    let channel = Reload::start(options.reload_port, stop.clone())?;
    let mut builder = Builder::new(root, native)?;
    let mut session = Session::default();
    let mut execute = |command: &mut Command, timeout| {
        bounded_until(command, timeout, || stop.load(Ordering::Relaxed))
    };
    // Observe before building: edits made during the initial compilation must schedule a batch.
    let mut snapshot = Snapshot::read(root)?;
    let initial = Work {
        generate: true,
        browser: true,
        native: true,
        restart: true,
        ..Default::default()
    };
    session.update(root, &options, &mut builder, &initial, &mut execute)?;
    channel.built(session.generation)?;
    let mut markup = Markup::new(root, &snapshot);
    println!(
        "dev http://localhost:{}/ (reload {}); Ctrl-C stops both processes",
        options.port, options.reload_port
    );
    let mut debounce = Debounce::default();
    while !stop.load(Ordering::Relaxed) {
        if channel.poll()? {
            // Missing markers, CSR pages or a failed patch require real rebuilt products.
            let work = Work {
                browser: true,
                native: true,
                restart: true,
                ..Default::default()
            };
            match session.update(root, &options, &mut builder, &work, &mut execute) {
                Ok(_) => {
                    channel.built(session.generation)?;
                    markup = Markup::new(root, &snapshot);
                }
                Err(error) => {
                    eprintln!("dev fallback build failed; last good server retained: {error}")
                }
            }
        }
        let next = Snapshot::read(root)?;
        debounce.observe(snapshot.changes(&next), Instant::now());
        snapshot = next;
        if let Some(paths) = debounce.ready(Instant::now(), Duration::from_millis(250)) {
            println!("changed: {paths:?}");
            if let Some(patches) = markup.patch(&paths, &snapshot) {
                channel.send(Notice::View(patches))?;
                continue;
            }
            let work = markup.rebuild_work(classify(&paths));
            match session.update(root, &options, &mut builder, &work, &mut execute) {
                Ok(notices) => {
                    for notice in notices {
                        if notice == Notice::All {
                            channel.built(session.generation)?;
                        } else {
                            channel.send(notice)?;
                        }
                    }
                    if work.browser || work.restart {
                        markup = Markup::new(root, &snapshot);
                    }
                }
                Err(error) => eprintln!("dev build failed; last good server retained: {error}"),
            }
        }
        session.check_exit()?;
        std::thread::sleep(Duration::from_millis(50));
    }
    // Drop the app while signal handlers and reload ownership still exist.
    drop(session);
    drop(signals);
    Ok(())
}

/// Retain and unregister only this command's signal-hook flag handlers.
struct Signals(Vec<signal_hook::SigId>);

impl Signals {
    /// Use safe signal-hook flag handlers; builds poll the same flag for prompt cancellation.
    fn register(stop: Arc<AtomicBool>) -> Result<Self, std::io::Error> {
        let mut signals = Self(Vec::new());
        for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
            signals
                .0
                .push(signal_hook::flag::register(signal, stop.clone())?);
        }
        Ok(signals)
    }
}

impl Drop for Signals {
    /// Remove only registrations created by this command, including partial registration failure.
    fn drop(&mut self) {
        for signal in self.0.drain(..) {
            signal_hook::low_level::unregister(signal);
        }
    }
}

/// A successful build is the only path to replacing the active owned process.
#[derive(Default)]
struct Session {
    /// The currently running executable's process group.
    child: Option<OwnedChild>,
    /// Immutable last-good executable, also reused for config-only restarts.
    executable: Option<PathBuf>,
    /// A restart generation lets the reload worker identify HTML loaded during compilation.
    generation: u64,
}

impl Session {
    /// Build all required products first; CSS-only batches do no compiling or restarting.
    fn update(
        &mut self,
        root: &Path,
        options: &Options,
        builder: &mut Builder,
        work: &Work,
        run: impl FnMut(&mut Command, Duration) -> BuildResult,
    ) -> Result<Vec<Notice>, Box<dyn std::error::Error>> {
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
            return Ok(vec![Notice::All]);
        }
        Ok(work.css.iter().cloned().map(Notice::Css).collect())
    }

    /// Stop/reap the previous group before spawning a replacement on the same app port.
    fn replace(&mut self, command: &mut Command, executable: PathBuf) -> BuildResult {
        if let Some(mut previous) = self.child.take() {
            previous.stop();
        }
        self.generation += 1;
        command.env("BLOG_DEV_BUILD_ID", self.generation.to_string());
        let child = OwnedChild::spawn(command)?;
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
    fn check_exit(&mut self) -> BuildResult {
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
        if let Some(child) = &mut self.child {
            child.stop();
        }
        if let Some(path) = &self.executable {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl Options {
    /// Parse paired CLI flags and reject ephemeral, conflicting or overflowing ports.
    /// # Errors
    /// Unknown flags, missing values, zero ports and conflicting ports are diagnostics.
    pub fn parse(args: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut port = 3000;
        let mut reload = None;
        let mut flags = args.chunks_exact(2);
        for pair in &mut flags {
            let value: u16 = pair[1].parse()?;
            if value == 0 {
                return Err("dev ports must be nonzero".into());
            }
            match pair[0].as_str() {
                "--port" => port = value,
                "--reload-port" => reload = Some(value),
                _ => return Err(format!("unknown dev flag: {}", pair[0]).into()),
            }
        }
        if !flags.remainder().is_empty() {
            return Err("dev flags require a port value".into());
        }
        let reload_port = reload
            .or_else(|| port.checked_add(1))
            .ok_or("reload port would overflow")?;
        if port == reload_port {
            return Err("app and reload ports must differ".into());
        }
        Ok(Self { port, reload_port })
    }
}

#[cfg(test)]
#[path = "../../tests/cases/dev.rs"]
mod tests;
