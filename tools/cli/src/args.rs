//! Supplied argv parsing; this module never reads process arguments or config files.

use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use stackpress_lib::{Error, Nest, ParseLimits, data::parse_args};

use crate::control::{ColorMode, Control};

/// A checked operator boundary or a target-data parsing failure.
#[derive(Debug)]
pub enum ArgumentError {
    /// Malformed application syntax; the message contains no raw argument payload.
    Usage(&'static str),
    /// The indexed argument could not be represented as UTF-8.
    NonUtf8(usize),
    /// The existing foundation parser or typed lookup rejected target data.
    Data(Error),
}

impl fmt::Display for ArgumentError {
    /// Render concise diagnostics without echoing potentially private argument values.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Public diagnostics describe the boundary without echoing secret argument contents.
        match self {
            Self::Usage(message) => formatter.write_str(message),
            Self::NonUtf8(index) => write!(formatter, "argument {index} is not UTF-8"),
            Self::Data(_) => formatter.write_str("invalid target arguments or lookup type"),
        }
    }
}

impl std::error::Error for ArgumentError {
    /// Retain the foundation diagnostic for explicit developer inspection.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Only data failures have an underlying cause; syntax/encoding already name the problem.
        match self {
            Self::Data(error) => Some(error),
            _ => None,
        }
    }
}

impl From<Error> for ArgumentError {
    /// Preserve parser causes without treating them as operator syntax.
    fn from(error: Error) -> Self {
        // Preserve the original typed diagnostic for explicit developer inspection.
        Self::Data(error)
    }
}

/// Application presentation defaults and request-local operator settings.
#[derive(Debug, Default)]
pub struct TerminalOptions {
    /// Request developer diagnostics from the execution adapter.
    pub verbose: bool,
    /// Metadata only: this never grants a capability permission to destroy data.
    pub force: bool,
    /// One explicitly selected JSON data file; parsing performs no filesystem I/O.
    pub bootstrap: Option<PathBuf>,
    /// Application label for explicitly requested human output.
    pub brand: String,
    /// Policy for human output; machine envelopes never use Control.
    pub color: ColorMode,
}

/// Immutable parsed command, original target tokens, bounded data and operator settings.
#[derive(Debug)]
pub struct Terminal {
    /// Event selected after the literal `emit` separator.
    command: String,
    /// Original tokens, never joined or re-tokenized.
    args: Vec<String>,
    /// Eagerly parsed data; false/zero/empty/null remain present.
    data: Nest,
    /// Checked operator options and application presentation defaults.
    options: TerminalOptions,
}

impl Terminal {
    /// Move checked command/data/settings into the execution adapter without cloning target input.
    pub(crate) fn into_parts(self) -> (String, Nest, TerminalOptions) {
        // Original argv has served its parsing purpose; no later event reparses process arguments.
        (self.command, self.data, self.options)
    }

    /// Parse argv **without the executable name**, requiring `emit <event>`.
    ///
    /// # Errors
    /// Rejects unknown/missing operator options, an empty event, control characters
    /// in the event, budgets, and invalid target argv under lib's strict grammar.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::Terminal;
    /// use stackpress_lib::ParseLimits;
    /// let argv = ["-v", "emit", "search", "--id", "42"].map(String::from);
    /// let terminal = Terminal::parse(&argv, ParseLimits::default())?;
    /// assert_eq!(terminal.command(), "search");
    /// assert!(terminal.options().verbose);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(args: &[String], limits: ParseLimits) -> Result<Self, ArgumentError> {
        // Reuse the same operator boundary with neutral application presentation defaults.
        Self::parse_with_options(args, limits, TerminalOptions::default())
    }

    /// Parse supplied OS arguments, rejecting rather than replacing invalid UTF-8.
    ///
    /// # Errors
    /// Returns the first non-UTF-8 index or the same errors as [`Self::parse`].
    ///
    /// # Examples
    /// ```
    /// use std::ffi::OsString;
    /// use stackpress_cli::Terminal;
    /// let args = [OsString::from("emit"), OsString::from("search")];
    /// assert_eq!(Terminal::parse_os(&args, Default::default())?.command(), "search");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_os(args: &[OsString], limits: ParseLimits) -> Result<Self, ArgumentError> {
        // Bound the original OS input before allocating its UTF-8 copy.
        check_budget(args.iter().map(|arg| arg.as_encoded_bytes().len()), limits)?;
        let strings = args
            .iter()
            .enumerate()
            .map(|(index, arg)| {
                // Report the first invalid index; replacement characters could select another
                // event.
                arg.to_str()
                    .map(String::from)
                    .ok_or(ArgumentError::NonUtf8(index))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::parse(&strings, limits)
    }

    /// Apply application defaults before parsing explicit operator overrides.
    ///
    /// Grouped `-vf`, `-vfb file.json`, `-bfile.json` and `--bootstrap=file.json`
    /// are accepted. `--` may end operator options immediately before `emit`.
    /// Every token after the event belongs to the target, including `-v/-f/-b`.
    ///
    /// # Errors
    /// Returns the same errors as [`Self::parse`]. No config file is opened.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{Terminal, TerminalOptions};
    /// let defaults = TerminalOptions { brand: "[app]".into(), ..Default::default() };
    /// let argv = ["emit", "search"].map(String::from);
    /// let terminal = Terminal::parse_with_options(&argv, Default::default(), defaults)?;
    /// assert_eq!(terminal.control().brand(), "[app]");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse_with_options(
        args: &[String],
        limits: ParseLimits,
        mut options: TerminalOptions,
    ) -> Result<Self, ArgumentError> {
        // Include operator and event bytes/nodes, not merely the target parser's subset.
        check_budget(args.iter().map(String::len), limits)?;
        let mut index = 0;
        while let Some(arg) = args.get(index) {
            // The separator owns the boundary; operator-looking target flags are never stolen.
            if arg == "emit" {
                break;
            }
            // Consume this operator token; value-taking options may advance the cursor again.
            index += 1;
            match arg.as_str() {
                // Bare switches affect only request-local operator metadata.
                "--verbose" => options.verbose = true,
                "--force" => options.force = true,
                "--bootstrap" | "-b" => {
                    // A named JSON path is data for later startup, never code loaded during
                    // parsing.
                    options.bootstrap = Some(take_file(args, &mut index)?.into());
                }
                // End operator processing; the next token must still be the literal emit separator.
                "--" => break,
                _ if arg.starts_with("--bootstrap=") => {
                    let file = &arg["--bootstrap=".len()..];
                    if file.is_empty() {
                        // Reject an empty inline path before publishing parsed options.
                        return Err(ArgumentError::Usage("bootstrap requires a file path"));
                    }
                    options.bootstrap = Some(file.into());
                }
                _ if arg.starts_with('-') && !arg.starts_with("--") => {
                    // Grouped short flags have their own explicit value-consumption rule.
                    parse_short(&arg[1..], args, &mut index, &mut options)?;
                }
                _ => {
                    // Unknown pre-emit tokens cannot silently become a target command.
                    return Err(ArgumentError::Usage(
                        "expected operator options followed by emit",
                    ));
                }
            }
        }
        // Require a real separator and nonempty printable event before allocating parsed state.
        if args.get(index).map(String::as_str) != Some("emit") {
            return Err(ArgumentError::Usage("expected emit <event>"));
        }
        let event = args
            .get(index + 1)
            .ok_or(ArgumentError::Usage("emit requires an event"))?;
        if event.trim().is_empty() || event.chars().any(char::is_control) {
            // Invalid selectors never reach event matching or human diagnostics.
            return Err(ArgumentError::Usage(
                "event must be nonempty and contain no controls",
            ));
        }
        // Preserve token boundaries and delegate all target coercion/limits to the approved parser.
        let target = &args[index + 2..];
        let data = parse_args(target, 0, &limits)?;
        Ok(Self {
            command: event.clone(),
            args: target.to_vec(),
            data,
            options,
        })
    }

    /// Borrow the selected event name.
    pub fn command(&self) -> &str {
        // Borrow the checked selector without permitting post-parse replacement.
        &self.command
    }

    /// Borrow original target argv, preserving spaces and shell punctuation literally.
    pub fn args(&self) -> &[String] {
        // Expose token fidelity without cloning or granting mutable argv access.
        &self.args
    }

    /// Borrow parsed target data without mutable aliases or copying.
    pub fn data(&self) -> &Nest {
        // Keep parsed state immutable so alias lookups cannot observe out-of-band edits.
        &self.data
    }

    /// Borrow application/operator settings separately from untrusted target data.
    pub fn options(&self) -> &TerminalOptions {
        // Operator metadata stays outside the capability's untrusted data namespace.
        &self.options
    }

    /// Create human-output policy without acquiring input or choosing an output stream.
    pub fn control(&self) -> Control {
        // Formatting owns its printable label but acquires neither a stream nor a terminal lease.
        Control::new(&self.options.brand, self.options.color)
    }

    /// Decode the first **present literal key** in alias order.
    ///
    /// Absence returns None; false/zero/empty/null never fall through to another alias.
    /// Decode a present null with `T = Option<_>` to distinguish it from absence.
    ///
    /// # Errors
    /// Returns a type/wire conversion error for the first present alias, without
    /// trying later aliases. Callers can use `?.unwrap_or(default)` for defaults.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::Terminal;
    /// let args = ["emit", "search", "--active", "false", "--limit", "0"].map(String::from);
    /// let terminal = Terminal::parse(&args, Default::default())?;
    /// assert_eq!(terminal.expect::<bool>(&["active", "a"])?, Some(false));
    /// assert_eq!(terminal.expect::<u32>(&["limit"])?, Some(0));
    /// assert_eq!(terminal.expect::<String>(&["name"])?.unwrap_or("Ada".into()), "Ada");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn expect<T: DeserializeOwned>(
        &self,
        aliases: &[&str],
    ) -> Result<Option<T>, ArgumentError> {
        // Presence, rather than truthiness, determines which alias owns a decode failure or value.
        for key in aliases {
            if let Some(value) = self.data.get(key) {
                // Convert only the selected value using the owning Nest's finite limits.
                let json = value.to_json_with(&self.data.limits())?;
                return serde_json::from_value(json)
                    .map(Some)
                    .map_err(Error::from)
                    .map_err(Into::into);
            }
        }
        // Absence remains distinguishable from any present JSON value, including null.
        Ok(None)
    }
}

/// Bound total argv allocation and checked arithmetic before parsing or cloning.
fn check_budget(
    lengths: impl Iterator<Item = usize>,
    limits: ParseLimits,
) -> Result<(), ArgumentError> {
    // Checked accumulation bounds the full supplied argv before any copies/parser allocation.
    let mut bytes = 0usize;
    for (index, length) in lengths.enumerate() {
        bytes = bytes
            .checked_add(length)
            .ok_or_else(|| Error::limit("argument size overflow"))?;
        if bytes > limits.max_bytes || index >= limits.max_entries {
            // Neither operator tokens nor an oversized collection bypass the target parser's
            // budget.
            return Err(Error::limit("argument budget exceeded").into());
        }
    }
    Ok(())
}

/// Consume one explicit file token; a following option is a missing-value error.
fn take_file<'a>(args: &'a [String], index: &mut usize) -> Result<&'a str, ArgumentError> {
    // A separate option cannot be consumed as the preceding bootstrap option's missing value.
    let value = args
        .get(*index)
        .ok_or(ArgumentError::Usage("bootstrap requires a file path"))?;
    if value.is_empty() || value.starts_with('-') {
        return Err(ArgumentError::Usage("bootstrap requires a file path"));
    }
    // Advance only after the selected file value has passed the shape check.
    *index += 1;
    Ok(value)
}

/// Parse grouped short operator flags, with `b` consuming the remaining group or next token.
fn parse_short(
    group: &str,
    args: &[String],
    index: &mut usize,
    options: &mut TerminalOptions,
) -> Result<(), ArgumentError> {
    // A lone dash is not a supported operator group, even though lib may ignore it as target data.
    if group.is_empty() {
        return Err(ArgumentError::Usage("unknown operator option"));
    }
    for (offset, flag) in group.char_indices() {
        // Character offsets preserve UTF-8 slicing safety even for an invalid non-ASCII switch.
        match flag {
            'v' => options.verbose = true,
            'f' => options.force = true,
            'b' => {
                // `b` ends the group; its suffix is a path, never additional switches.
                let suffix = &group[offset + 1..];
                let file = if suffix.is_empty() {
                    // No attached path: take exactly the next explicit value token.
                    take_file(args, index)?
                } else {
                    // An attached path ends the switch group and is preserved literally.
                    suffix
                };
                options.bootstrap = Some(file.into());
                return Ok(());
            }
            _ => return Err(ArgumentError::Usage("unknown operator option")),
        }
    }
    Ok(())
}
