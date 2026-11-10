//! Stage complete build batches before publishing bundles or replacing the last good server.

use super::change::Work;
use crate::{BuildResult, build_into, target_parent};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Build paths belong to this copied blog, never another worktree or a global target.
pub(super) struct Builder {
    /// Authored app directory and working directory for every child.
    root: PathBuf,
    /// Native app compilation target, distinct from the two browser targets.
    native: PathBuf,
    /// Monotonic candidate names avoid modifying an executable that is still running.
    sequence: u64,
}

/// A complete successful batch; failed builds never reach publication.
pub(super) struct Prepared {
    /// Staged browser directory, present only after both renderers and bindgen succeed.
    browser: Option<PathBuf>,
    /// Immutable candidate executable, ready before the previous process is stopped.
    pub executable: Option<PathBuf>,
}

impl Builder {
    /// Resolve Cargo's optional relative override before changing any output.
    pub fn new(root: &Path, native: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        // Reuse the browser helper's validation and sibling target convention.
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

    /// Generate first, compile browser consumers, then compile/copy the candidate native app.
    /// The runner receives actual commands, allowing finite plan/error/cancellation tests.
    pub fn prepare(
        &mut self,
        work: &Work,
        mut run: impl FnMut(&mut Command, Duration) -> BuildResult,
    ) -> Result<Prepared, Box<dyn std::error::Error>> {
        let stage = self.root.join(".build/dev/browser");
        if work.generate {
            let mut command = self.cargo();
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
        if work.browser {
            if stage.exists() {
                std::fs::remove_dir_all(&stage)?;
            }
            // Both builds need the debug marker configuration used by Leptos's patcher.
            build_into(
                &self.root,
                Some(&self.native),
                &stage,
                |command, timeout| {
                    command.env("LEPTOS_WATCH", "1");
                    run(command, timeout)
                },
            )?;
        }
        let executable = if work.native {
            let mut command = self.cargo();
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
            // Only successful complete builds get a new immutable executable path.
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
            browser: work.browser.then_some(stage),
            executable,
        })
    }

    /// Shared Cargo policy is explicit for generation and native builds.
    fn cargo(&self) -> Command {
        let mut command = Command::new("cargo");
        command
            .current_dir(&self.root)
            .env("CARGO_TARGET_DIR", &self.native)
            .env("LEPTOS_WATCH", "1");
        command
    }
}

impl Prepared {
    /// Publish both browser modes together, restoring the old directory if rename fails.
    pub fn publish(&self, root: &Path) -> BuildResult {
        let Some(source) = &self.browser else {
            return Ok(());
        };
        let destination = root.join(".build/browser");
        let backup = root.join(".build/dev/previous-browser");
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

/// Every replacement gets the same runtime-only reload and source CSS environment.
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
