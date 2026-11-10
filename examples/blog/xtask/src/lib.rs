//! Browser builds and a live development loop, isolated from the application and generator.
//!
//! Finite compiler/bindgen children have deadlines. The dev server and reload worker live
//! until Ctrl-C; every exit path retires their owned process groups and thread.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

pub mod dev;

/// Build or development-loop result, preserving filesystem and child-process diagnostics.
pub type BuildResult = Result<(), Box<dyn std::error::Error>>;

/// Compile both browser modes and emit `.build/browser/{hydrate,csr}` with `wasm-bindgen`.
///
/// `native_target` follows Cargo's `CARGO_TARGET_DIR`; its siblings hold each renderer.
/// With no override, targets live under the blog's `target/` directory.
///
/// # Errors
/// Returns the first filesystem, missing-command, unsuccessful-child or deadline error.
pub fn build_browser(root: &Path, native_target: Option<&Path>) -> BuildResult {
    build_with(root, native_target, bounded)
}

/// Plan each mode in order; an unsuccessful compile must never publish its bindgen output.
fn build_with(
    root: &Path,
    native_target: Option<&Path>,
    mut run: impl FnMut(&mut Command, Duration) -> BuildResult,
) -> BuildResult {
    build_into(root, native_target, &root.join(".build/browser"), &mut run)
}

/// Stage browser output separately during development; publication waits for every build.
fn build_into(
    root: &Path,
    native_target: Option<&Path>,
    output: &Path,
    mut run: impl FnMut(&mut Command, Duration) -> BuildResult,
) -> BuildResult {
    let targets = target_parent(root, native_target)?;
    for mode in ["hydrate", "csr"] {
        let target = targets.join(mode);
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(root)
            .env("CARGO_TARGET_DIR", &target)
            .args(["build", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .args([
                "-p",
                "blog-client",
                "--target",
                "wasm32-unknown-unknown",
                "--no-default-features",
                "--features",
                mode,
                "--locked",
                "--offline",
            ]);
        run(&mut cargo, Duration::from_secs(600))?;
        // Publish glue only after the matching mode has produced its WASM artifact.
        let output = output.join(mode);
        std::fs::create_dir_all(&output)?;
        let mut bindgen = Command::new("wasm-bindgen");
        bindgen
            .current_dir(root)
            .args(["--target", "web", "--out-dir"])
            .arg(&output)
            .args(["--out-name", "blog"])
            .arg(target.join("wasm32-unknown-unknown/debug/blog_client.wasm"));
        run(&mut bindgen, Duration::from_secs(600))?;
    }
    Ok(())
}

/// Resolve relative Cargo target overrides from the caller's blog directory.
fn target_parent(
    root: &Path,
    native: Option<&Path>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let native = match native {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => root.join(path),
        None => root.join("target/native"),
    };
    Ok(native
        .parent()
        .ok_or("native target has no parent")?
        .to_path_buf())
}

/// Own a child until it exits or is killed; unwinding also retires its process group.
struct OwnedChild {
    /// The direct child whose group contains all nested compilers and helpers.
    child: Child,
    /// A completed child has already been reaped by `try_wait`.
    reaped: bool,
}

impl OwnedChild {
    /// Spawn into a fresh group so replacing the app cannot affect xtask or another command.
    fn spawn(command: &mut Command) -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Ok(Self {
            child: command.spawn()?,
            reaped: false,
        })
    }

    /// Give the app's Ctrl-C shutdown a bounded chance to drain before killing its group.
    fn stop(&mut self) {
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .args(["-INT", "--", &format!("-{}", self.child.id())])
                .status();
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(2) {
                if self.child.try_wait().ok().flatten().is_some() {
                    // Even if the leader exited, retire any descendants left in its group.
                    self.terminate();
                    return;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        self.terminate();
    }

    /// Terminate only this command's group, with a direct-child fallback if the OS tool fails.
    fn terminate(&mut self) {
        #[cfg(unix)]
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", self.child.id())])
            .status();
        #[cfg(windows)]
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &self.child.id().to_string()])
            .status();
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.reaped = true;
    }
}

impl Drop for OwnedChild {
    /// A failed wait, deadline, or panic cannot leave this owned build running.
    fn drop(&mut self) {
        if !self.reaped {
            self.terminate();
        }
    }
}

/// Run one command with inherited output and an explicit wall-clock deadline.
///
/// Unix children receive a fresh process group; Windows cleanup uses the owned process tree.
/// # Errors
/// Reports spawn/wait failures, nonzero exit codes and elapsed deadlines.
pub fn bounded(command: &mut Command, timeout: Duration) -> BuildResult {
    bounded_until(command, timeout, || false)
}

/// Cancel a build promptly when dev receives Ctrl-C, including nested compiler processes.
fn bounded_until(
    command: &mut Command,
    timeout: Duration,
    cancelled: impl Fn() -> bool,
) -> BuildResult {
    let description = format!("{command:?}");
    let mut owned = OwnedChild::spawn(command)?;
    let start = Instant::now();
    loop {
        if let Some(status) = owned.child.try_wait()? {
            owned.reaped = true;
            return successful(status, &description);
        }
        if cancelled() {
            return Err("build cancelled".into());
        }
        if start.elapsed() >= timeout {
            owned.terminate();
            return Err(format!("build deadline exceeded: {description}").into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Keep unsuccessful child status visible rather than continuing with stale output.
fn successful(status: ExitStatus, command: &str) -> BuildResult {
    if !status.success() {
        return Err(format!("build command failed ({status}): {command}").into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/cases/build.rs"]
mod tests;
