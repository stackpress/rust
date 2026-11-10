//! Prioritize hydration/native builds; stage and publish each complete renderer separately.
use super::change::Work;
use crate::{BuildResult, build_mode, target_parent};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

/// Build paths belong to this copied blog, never another worktree or a global target.
#[derive(Clone)]
pub(super) struct Builder {
    /// Authored app directory and working directory for every child.
    root: PathBuf,
    /// Native target; browser consumers have independent sibling targets.
    native: PathBuf,
    /// Immutable executable names avoid modifying a running server.
    sequence: u64,
}
/// A successful critical batch, or a complete deferred CSR bundle.
pub(super) struct Prepared {
    /// One staged renderer, paired with its public mode directory.
    browser: Option<(PathBuf, &'static str)>,
    /// Immutable executable, copied before stopping the previous process.
    pub executable: Option<PathBuf>,
}
impl Builder {
    /// Resolve Cargo's optional target override before changing output.
    pub fn new(root: &Path, native: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let parent = target_parent(root, native)?;
        let native = match native {
            Some(path) if path.is_absolute() => path.to_path_buf(),
            Some(path) => root.join(path),
            None => parent.join("native"),
        };
        Ok(Self {
            root: root.into(),
            native,
            sequence: 0,
        })
    }
    /// Generate first, then build hydration and native concurrently, at one Cargo job each.
    /// Both scoped workers finish before publication; either failure retains all old products.
    pub fn prepare(
        &mut self,
        work: &Work,
        run: impl Fn(&mut Command, Duration) -> BuildResult + Sync,
    ) -> Result<Prepared, Box<dyn std::error::Error>> {
        if work.generate {
            let mut command = self.cargo(2)?;
            command.args([
                "run",
                "-p",
                "blog-tool",
                "--locked",
                "--offline",
                "--",
                "generate-app",
            ]);
            run(&mut command, Duration::from_secs(600))?;
        }
        // Two independent targets permit concurrent compiler trees without Cargo lock contention.
        let browser = std::thread::scope(|scope| {
            let hydrate = scope.spawn(|| {
                if work.browser {
                    self.renderer("hydrate", &run)
                        .map(Some)
                        .map_err(|e| e.to_string())
                } else {
                    Ok(None)
                }
            });
            let native = (|| -> BuildResult {
                if work.native {
                    let mut command = self.cargo(if work.browser { 1 } else { 2 })?;
                    command.args([
                        "build",
                        "-p",
                        "blog-web",
                        "--bin",
                        "serve",
                        "--locked",
                        "--offline",
                    ]);
                    run(&mut command, Duration::from_secs(600))?;
                }
                Ok(())
            })()
            .map_err(|e| e.to_string());
            // Join on errors too: no build may outlive this batch or overwrite a later stage.
            let browser = hydrate
                .join()
                .map_err(|_| "hydrate worker panicked".to_owned())?;
            native?;
            browser
        })?;
        let executable = if work.native {
            self.sequence += 1;
            let path = self.root.join(format!(
                ".build/dev/serve-{}{}",
                self.sequence,
                std::env::consts::EXE_SUFFIX
            ));
            std::fs::create_dir_all(path.parent().ok_or("candidate has no parent")?)?;
            std::fs::copy(
                self.native
                    .join(format!("debug/serve{}", std::env::consts::EXE_SUFFIX)),
                &path,
            )?;
            Some(path)
        } else {
            None
        };
        Ok(Prepared {
            browser,
            executable,
        })
    }
    /// Compile and bindgen one renderer in isolation; incomplete staging is never public.
    fn renderer(
        &self,
        mode: &'static str,
        run: impl Fn(&mut Command, Duration) -> BuildResult,
    ) -> Result<(PathBuf, &'static str), Box<dyn std::error::Error>> {
        let stage = self.root.join(".build/dev/browser").join(mode);
        if stage.exists() {
            std::fs::remove_dir_all(&stage)?;
        }
        build_mode(
            &self.root,
            Some(&self.native),
            stage.parent().ok_or("stage has no parent")?,
            mode,
            |command, timeout| {
                policy(command)?;
                if mode == "csr" {
                    command.env("CARGO_BUILD_JOBS", "2");
                }
                run(command, timeout)
            },
        )?;
        Ok((stage, mode))
    }
    /// Generation/native builds override an inherited gate environment without changing gate policy.
    fn cargo(&self, jobs: u8) -> Result<Command, Box<dyn std::error::Error>> {
        let mut command = Command::new("cargo");
        command
            .current_dir(&self.root)
            .env("CARGO_TARGET_DIR", &self.native);
        policy(&mut command)?;
        command.env("CARGO_BUILD_JOBS", jobs.to_string());
        Ok(command)
    }
}
/// Default to one job per concurrent compiler; solitary native/CSR builds receive two.
fn policy(command: &mut Command) -> BuildResult {
    command
        .env("LEPTOS_WATCH", "1")
        .env("CARGO_INCREMENTAL", "1")
        .env("CARGO_BUILD_JOBS", "1");
    if command.get_program() == "cargo" {
        // Cargo supplies incremental rustc flags. Hide its environment switch from sccache
        // 0.19, which otherwise refuses even nonincremental dependency compilations.
        // The wrapper also keeps unwatched framework path dependencies cacheable.
        let wrapper = std::env::var_os("RUSTC_WRAPPER").unwrap_or_default();
        let root = command
            .get_current_dir()
            .ok_or("dev compiler has no working directory")?
            .to_owned();
        command
            .env("BLOG_DEV_COMPILER_WRAPPER", wrapper)
            .env("BLOG_DEV_WORKSPACE_ROOT", root)
            .env("RUSTC_WRAPPER", std::env::current_exe()?);
    }
    Ok(())
}
impl Prepared {
    /// Swap one complete mode, restoring the last-good directory if publication fails.
    pub fn publish(&self, root: &Path) -> BuildResult {
        let Some((source, mode)) = &self.browser else {
            return Ok(());
        };
        let destination = root.join(".build/browser").join(mode);
        let backup = root.join(".build/dev").join(format!("previous-{mode}"));
        std::fs::create_dir_all(destination.parent().ok_or("bundle has no parent")?)?;
        if backup.exists() {
            std::fs::remove_dir_all(&backup)?;
        }
        let previous = destination.exists();
        if previous {
            std::fs::rename(&destination, &backup)?;
        }
        if let Err(error) = std::fs::rename(source, &destination) {
            if previous {
                std::fs::rename(&backup, &destination)?;
            }
            return Err(error.into());
        }
        if previous {
            std::fs::remove_dir_all(backup)?;
        }
        Ok(())
    }
}
/// One cancelable CSR worker; Drop joins after killing/reaping its compiler group.
pub(super) struct Csr {
    /// Superseding source edits cancel this worker independently of Ctrl-C.
    cancel: Arc<AtomicBool>,
    /// Only a joined successful result may publish.
    worker: Option<std::thread::JoinHandle<Result<Prepared, String>>>,
}
impl Csr {
    /// Start only after the critical reload has been delivered.
    pub fn start(builder: Builder, stop: Arc<AtomicBool>) -> Self {
        Self::start_with(builder, move |command, timeout, cancel| {
            crate::bounded_until(command, timeout, || {
                stop.load(Ordering::Relaxed) || cancel.load(Ordering::Relaxed)
            })
        })
    }
    /// Substitute finite commands to test concurrency/cancellation without sockets.
    pub(super) fn start_with(
        builder: Builder,
        run: impl Fn(&mut Command, Duration, &AtomicBool) -> BuildResult + Send + 'static,
    ) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let canceled = cancel.clone();
        let worker = std::thread::spawn(move || {
            builder
                .renderer("csr", |command, timeout| run(command, timeout, &canceled))
                .map(|browser| Prepared {
                    browser: Some(browser),
                    executable: None,
                })
                .map_err(|e| e.to_string())
        });
        Self {
            cancel,
            worker: Some(worker),
        }
    }
    /// Nonblocking completion leaves source polling and patch feedback responsive.
    pub fn finished(&self) -> bool {
        self.worker.as_ref().is_some_and(|w| w.is_finished())
    }
    /// Consume completed output; canceled and failed staging stays private.
    pub fn finish(&mut self) -> Result<Prepared, Box<dyn std::error::Error>> {
        self.worker
            .take()
            .ok_or("CSR worker already joined")?
            .join()
            .map_err(|_| "CSR worker panicked")?
            .map_err(Into::into)
    }
}
impl Drop for Csr {
    /// Superseding edits and Ctrl-C cannot leave a compiler writing a reused staging path.
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
/// Every replacement gets runtime-only reload and source CSS environment.
pub(super) fn server_command(root: &Path, executable: &Path, options: &super::Options) -> Command {
    let mut command = Command::new(executable);
    command
        .current_dir(root)
        .arg(options.port.to_string())
        .arg("config/develop.json")
        .env("BLOG_DEV_RELOAD_PORT", options.reload_port.to_string())
        .env("BLOG_DEV_ASSETS", root.join("plugins/assets/assets"))
        .env("LEPTOS_WATCH", "1");
    command
}
#[cfg(test)]
#[path = "../../tests/cases/dev_build.rs"]
mod tests;
