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

/// Run Cargo's compiler wrapper with incremental blog crates and cached framework dependencies.
///
/// Cargo has already supplied the incremental compiler flags. Its environment switch is removed
/// only in this subprocess, because sccache rejects that switch before inspecting rustc flags.
/// Path dependencies outside the watched blog keep nonincremental, cacheable compiler arguments.
/// # Errors
/// Returns missing compiler arguments, spawn failures, or unsuccessful compiler status.
pub fn compiler(wrapper: &std::ffi::OsStr, args: &[std::ffi::OsString]) -> BuildResult {
    let local = std::env::var_os("BLOG_DEV_WORKSPACE_ROOT")
        .zip(std::env::var_os("CARGO_MANIFEST_DIR"))
        .is_some_and(|(root, package)| Path::new(&package).starts_with(root));
    let mut command = compiler_command(wrapper, args, local)?;
    let status = command.status()?;
    if !status.success() {
        return Err(format!("compiler exited: {status}").into());
    }
    Ok(())
}

/// Incremental local crates bypass the cache; unchanged nonincremental dependencies use it.
fn compiler_command(
    wrapper: &std::ffi::OsStr,
    args: &[std::ffi::OsString],
    local: bool,
) -> Result<Command, Box<dyn std::error::Error>> {
    let rustc = args.first().ok_or("compiler wrapper requires rustc")?;
    // CARGO_INCREMENTAL=1 also affects framework path dependencies. Their sources are not
    // watched by this loop: keep them nonincremental and cacheable, as in Cargo's defaults.
    let mut filtered = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_string_lossy();
        // Accept Cargo's split -C form and the equivalent compact rustc option.
        if !local
            && arg == "-C"
            && args
                .get(index + 1)
                .is_some_and(|s| s.to_string_lossy().starts_with("incremental="))
        {
            index += 2;
        } else if !local && arg.starts_with("-Cincremental=") {
            index += 1;
        } else {
            filtered.push(args[index].clone());
            index += 1;
        }
    }
    let incremental = filtered.iter().any(|arg| {
        let arg = arg.to_string_lossy();
        arg.starts_with("incremental=") || arg.starts_with("-Cincremental=")
    });
    let mut command = if incremental {
        let mut command = Command::new(rustc);
        command.args(&filtered[1..]);
        command
    } else {
        let mut command = if wrapper.is_empty() {
            Command::new(rustc)
        } else {
            Command::new(wrapper)
        };
        command.args(if wrapper.is_empty() {
            &filtered[1..]
        } else {
            &filtered
        });
        command
    };
    command
        .env_remove("CARGO_INCREMENTAL")
        .env_remove("BLOG_DEV_COMPILER_WRAPPER")
        .env_remove("BLOG_DEV_WORKSPACE_ROOT");
    Ok(command)
}

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

/// Build both modes for the finite setup command; the dev loop calls each mode separately.
fn build_into(
    root: &Path,
    native_target: Option<&Path>,
    output: &Path,
    mut run: impl FnMut(&mut Command, Duration) -> BuildResult,
) -> BuildResult {
    for mode in ["hydrate", "csr"] {
        build_mode(root, native_target, output, mode, &mut run)?;
    }
    Ok(())
}

/// Build one complete renderer into a caller-owned staging directory.
fn build_mode(
    root: &Path,
    native_target: Option<&Path>,
    output: &Path,
    mode: &str,
    mut run: impl FnMut(&mut Command, Duration) -> BuildResult,
) -> BuildResult {
    let targets = target_parent(root, native_target)?;
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
