//! Primitives for an application's compiled `stackpress` executable.
//!
//! This package intentionally has no stock/empty binary. M0 proves native terminal
//! input ownership. Parsing, formatting and prompts are usable independently of a
//! server. Prompts never run implicitly during capability dispatch. Application
//! execution and exit-code mapping are a later integration milestone.
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

pub use args::{ArgumentError, Terminal, TerminalOptions};
pub use control::{ColorMode, Control, Level, OutputEnvironment};

pub use stackpress_server::ConfigLayers;

#[cfg(feature = "tty")]
pub mod tty;
