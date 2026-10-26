//! Primitives for an application's compiled `stackpress` executable.
//!
//! This package intentionally has no stock/empty binary. M0 proves native terminal
//! input ownership. Parsing, formatting and prompts are usable independently of a
//! server. Prompts never run implicitly during capability dispatch. [`run`] enters
//! the server lifecycle once; [`main_with`] supplies process argv/config/sinks for
//! an application-owned executable with its selected compiled plugins.
//!
//! ```
//! use stackpress_cli::Terminal;
//! use stackpress_lib::ParseLimits;
//! let args = ["-v", "emit", "profile-search", "--id", "abc123"].map(String::from);
//! let terminal = Terminal::parse(&args, ParseLimits::default())?;
//! assert!(terminal.options().verbose);
//! assert_eq!(terminal.expect::<String>(&["id"])?, Some("abc123".into()));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod args;
pub mod control;
pub mod prompt;

mod diagnostics;
mod entry;
mod execution;

pub use entry::{main_with, main_with_args};
pub use execution::{CliContext, CliRun, CliSinks, run};

pub use args::{ArgumentError, Terminal, TerminalOptions};
pub use control::{ColorMode, Control, Level, OutputEnvironment};

pub use stackpress_server::ConfigLayers;

#[cfg(feature = "tty")]
pub mod tty;

// Unit-test modules share one watchdog definition rather than loading the same file twice.
#[cfg(test)]
#[path = "../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;
