//! Blog development orchestration; only the explicit `dev` command opens loopback listeners.

mod build;
mod change;
mod markup;
mod reload;
mod session;

use crate::{BuildResult, bounded_until};
use build::Builder;
use change::{Debounce, Snapshot, Work, classify};
use markup::Markup;
use reload::{Notice, Reload};
use session::Session;
use std::path::Path;
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
    let mut session = Session::new(stop.clone());
    let execute = |command: &mut Command, timeout| {
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
    session.update(root, &options, &mut builder, &initial, execute)?;
    channel.built(session.generation, session.notice())?;
    session.defer_csr(&builder, stop.clone());
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
            match session.update(root, &options, &mut builder, &work, execute) {
                Ok(_) => {
                    channel.built(session.generation, session.notice())?;
                    session.defer_csr(&builder, stop.clone());
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
            if paths
                .iter()
                .any(|path| path.starts_with("plugins/app/src/views"))
            {
                let reason = markup.reason(&paths, &snapshot);
                eprintln!("{reason} → rebuilding");
            }
            let work = markup.rebuild_work(classify(&paths));
            match session.update(root, &options, &mut builder, &work, execute) {
                Ok(notices) => {
                    for notice in notices {
                        if matches!(notice, Notice::All | Notice::Hydrate) {
                            channel.built(session.generation, session.notice())?;
                        } else {
                            channel.send(notice)?;
                        }
                    }
                    session.defer_csr(&builder, stop.clone());
                    if work.browser || work.restart {
                        markup = Markup::new(root, &snapshot);
                    }
                }
                Err(error) => eprintln!("dev build failed; last good server retained: {error}"),
            }
        }
        if let Some(notice) = session.finish_csr(root) {
            channel.send(notice)?;
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

#[cfg(test)]
#[path = "../../tests/cases/timing.rs"]
mod timing;
