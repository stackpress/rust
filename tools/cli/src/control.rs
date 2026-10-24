//! Human diagnostics through explicit writers; never use this for machine envelopes.

use std::io::{self, IsTerminal, Write};

use crate::prompt::PromptOptions;

/// ANSI policy for explicitly selected human output.
///
/// # Examples
/// ```
/// use stackpress_cli::{ColorMode, Control, Level, OutputEnvironment};
/// let output = Control::new("app", ColorMode::Never)
///     .format(Level::Error, "failed", &[], OutputEnvironment::default());
/// assert_eq!(output, "app failed\n");
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorMode {
    /// Use color only on terminals when NO_COLOR is absent or empty.
    #[default]
    Auto,
    /// Explicitly request ANSI even for a supplied non-terminal writer.
    Always,
    /// Never generate ANSI.
    Never,
}

/// Human diagnostic severity; no filtering or host logging is implicit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    /// Plain output.
    Output,
    /// Red diagnostic.
    Error,
    /// Blue information.
    Info,
    /// Green completion.
    Success,
    /// Magenta system notice.
    System,
    /// Yellow caution.
    Warning,
}

/// Snapshot the properties of the **chosen output**, allowing pure buffer tests.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OutputEnvironment {
    /// True only when the selected output is a terminal.
    pub is_terminal: bool,
    /// True when NO_COLOR has a nonempty value.
    pub no_color: bool,
}

impl OutputEnvironment {
    /// Detect native stderr/environment without reading or mutating stdin.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::OutputEnvironment;
    /// let selected_output = OutputEnvironment::stderr();
    /// // Pass this snapshot with an explicitly selected stderr writer.
    /// let _ = selected_output.is_terminal;
    /// ```
    pub fn stderr() -> Self {
        // Snapshot only the selected output and NO_COLOR; stdin ownership is unrelated.
        Self {
            is_terminal: io::stderr().is_terminal(),
            no_color: std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()),
        }
    }
}

/// Immutable brand/color formatting policy, with a caller-owned writer per operation.
#[derive(Debug)]
pub struct Control {
    /// Untrusted labels are normalized to printable text on construction.
    brand: String,
    /// Only this policy and the chosen output environment can enable generated ANSI.
    color: ColorMode,
}

impl Control {
    /// Create policy; control characters in the label become visible replacement characters.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let control = Control::new(" [app] ", ColorMode::Auto);
    /// assert_eq!(control.brand(), "[app]");
    /// ```
    pub fn new(brand: &str, color: ColorMode) -> Self {
        // Normalize the label once so every later diagnostic has the same printable prefix.
        Self {
            brand: display_text(brand).trim().to_owned(),
            color,
        }
    }

    /// Borrow the printable application label.
    pub fn brand(&self) -> &str {
        // Return the normalized label, not the untrusted original application text.
        &self.brand
    }

    /// Format sequential `%s` substitutions and one terminating newline, without I/O.
    ///
    /// Replacement values are literal: `%s` inside an inserted value is not consumed
    /// by later substitutions. All supplied text is sanitized before generated ANSI.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control, Level, OutputEnvironment};
    /// let control = Control::new("app", ColorMode::Never);
    /// let text = control.format(Level::Info, "%s -> %s", &["a", "b"],
    ///     OutputEnvironment::default());
    /// assert_eq!(text, "app a -> b\n");
    /// ```
    pub fn format(
        &self,
        level: Level,
        message: &str,
        variables: &[&str],
        environment: OutputEnvironment,
    ) -> String {
        // Walk only placeholders from the original template, never an inserted variable.
        let mut remainder = message;
        let mut substituted = String::new();
        for variable in variables {
            let Some((prefix, suffix)) = remainder.split_once("%s") else {
                // Extra variables have no placeholder and therefore contribute no output.
                break;
            };
            // Consume only the template suffix; inserted values never become a new template.
            substituted.push_str(prefix);
            substituted.push_str(variable);
            remainder = suffix;
        }
        // Keep unused placeholders literal, then sanitize the whole resulting human message.
        substituted.push_str(remainder);
        let message = display_text(&substituted);
        let text = format!("{} {}", self.brand, message).trim().to_owned();
        let enabled = match self.color {
            // Auto is conservative; explicit modes are deliberate caller overrides.
            ColorMode::Auto => environment.is_terminal && !environment.no_color,
            ColorMode::Always => true,
            ColorMode::Never => false,
        };
        // Level selects a fixed trusted escape sequence; callers cannot supply a color template.
        let color = match level {
            Level::Output => None,
            Level::Error => Some(31),
            Level::Info => Some(34),
            Level::Success => Some(32),
            Level::System => Some(35),
            Level::Warning => Some(33),
        };
        if enabled && let Some(color) = color {
            // Trusted framing surrounds already-sanitized text and resets color before newline.
            return format!("\x1b[{color}m{text}\x1b[0m\n");
        }
        // Plain output also gets exactly one renderer-owned terminating newline.
        format!("{text}\n")
    }

    /// Write a formatted diagnostic once; partial writes are never retried as a new message.
    ///
    /// # Errors
    /// Returns the writer's I/O failure. Arbitrary synchronous writers may block;
    /// this API provides no cancellation guarantee for caller code.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control, Level, OutputEnvironment};
    /// let mut output = Vec::new();
    /// Control::new("", ColorMode::Never).write(&mut output, Default::default(),
    ///     Level::Warning, "Check %s", &["input"])?;
    /// assert_eq!(output, b"Check input\n");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn write(
        &self,
        writer: &mut (dyn Write + Send),
        environment: OutputEnvironment,
        level: Level,
        message: &str,
        variables: &[&str],
    ) -> io::Result<()> {
        // Prepare before writing, and propagate a partial failure without retrying another message.
        writer.write_all(
            self.format(level, message, variables, environment)
                .as_bytes(),
        )
    }

    /// Write uncolored human output through the selected sink.
    ///
    /// # Errors
    /// Propagates synchronous writer errors.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let mut output = Vec::new();
    /// Control::new("app", ColorMode::Never).output(&mut output,
    ///     Default::default(), "Hello %s", &["Ada"])?;
    /// assert_eq!(output, b"app Hello Ada\n");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn output(
        &self,
        writer: &mut (dyn Write + Send),
        environment: OutputEnvironment,
        message: &str,
        variables: &[&str],
    ) -> io::Result<()> {
        // Keep output framing and error propagation in the common writer boundary.
        self.write(writer, environment, Level::Output, message, variables)
    }

    /// Write a red diagnostic when color policy allows it.
    ///
    /// # Errors
    /// Propagates synchronous writer errors.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let mut output = Vec::new();
    /// Control::new("app", ColorMode::Never).error(&mut output,
    ///     Default::default(), "Hello %s", &["Ada"])?;
    /// assert_eq!(output, b"app Hello Ada\n");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn error(
        &self,
        writer: &mut (dyn Write + Send),
        environment: OutputEnvironment,
        message: &str,
        variables: &[&str],
    ) -> io::Result<()> {
        // Keep error framing and error propagation in the common writer boundary.
        self.write(writer, environment, Level::Error, message, variables)
    }

    /// Write blue information when color policy allows it.
    ///
    /// # Errors
    /// Propagates synchronous writer errors.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let mut output = Vec::new();
    /// Control::new("app", ColorMode::Never).info(&mut output,
    ///     Default::default(), "Hello %s", &["Ada"])?;
    /// assert_eq!(output, b"app Hello Ada\n");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn info(
        &self,
        writer: &mut (dyn Write + Send),
        environment: OutputEnvironment,
        message: &str,
        variables: &[&str],
    ) -> io::Result<()> {
        // Keep info framing and error propagation in the common writer boundary.
        self.write(writer, environment, Level::Info, message, variables)
    }

    /// Write a green completion when color policy allows it.
    ///
    /// # Errors
    /// Propagates synchronous writer errors.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let mut output = Vec::new();
    /// Control::new("app", ColorMode::Never).success(&mut output,
    ///     Default::default(), "Hello %s", &["Ada"])?;
    /// assert_eq!(output, b"app Hello Ada\n");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn success(
        &self,
        writer: &mut (dyn Write + Send),
        environment: OutputEnvironment,
        message: &str,
        variables: &[&str],
    ) -> io::Result<()> {
        // Keep success framing and error propagation in the common writer boundary.
        self.write(writer, environment, Level::Success, message, variables)
    }

    /// Write a magenta system notice when color policy allows it.
    ///
    /// # Errors
    /// Propagates synchronous writer errors.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let mut output = Vec::new();
    /// Control::new("app", ColorMode::Never).system(&mut output,
    ///     Default::default(), "Hello %s", &["Ada"])?;
    /// assert_eq!(output, b"app Hello Ada\n");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn system(
        &self,
        writer: &mut (dyn Write + Send),
        environment: OutputEnvironment,
        message: &str,
        variables: &[&str],
    ) -> io::Result<()> {
        // Keep system framing and error propagation in the common writer boundary.
        self.write(writer, environment, Level::System, message, variables)
    }

    /// Write a yellow warning when color policy allows it.
    ///
    /// # Errors
    /// Propagates synchronous writer errors.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let mut output = Vec::new();
    /// Control::new("app", ColorMode::Never).warning(&mut output,
    ///     Default::default(), "Hello %s", &["Ada"])?;
    /// assert_eq!(output, b"app Hello Ada\n");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn warning(
        &self,
        writer: &mut (dyn Write + Send),
        environment: OutputEnvironment,
        message: &str,
        variables: &[&str],
    ) -> io::Result<()> {
        // Keep warning framing and error propagation in the common writer boundary.
        self.write(writer, environment, Level::Warning, message, variables)
    }

    /// Build an explicit prompt, required only when no default was supplied.
    ///
    /// Drive the returned options with [`crate::prompt::prompt`] or the native
    /// adapter. Creating options never acquires input ownership or blocks.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::{ColorMode, Control};
    /// let control = Control::new("app", ColorMode::Auto);
    /// let options = control.input("Name?", Some("Ada"));
    /// assert!(!options.required);
    /// assert_eq!(options.default.as_deref(), Some("Ada"));
    /// // Drive options with prompt or prompt_native when the application requests input.
    /// ```
    pub fn input(&self, question: &str, answer: Option<&str>) -> PromptOptions {
        // Presence of a default, including an empty one, makes the convenience prompt optional.
        let mut options = PromptOptions::new(question);
        options.default = answer.map(String::from);
        options.required = answer.is_none();
        options
    }
}

/// Remove C0/C1 terminal controls and bidi overrides from untrusted display text.
pub(crate) fn display_text(value: &str) -> String {
    // Retain ordinary Unicode text while removing terminal commands and directional overrides.
    value
        .chars()
        .map(|character| {
            // Caller text cannot introduce newlines/ESC; renderers own terminal commands.
            if character.is_control()
                || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
            {
                // A visible replacement cannot move the cursor or hide following diagnostic text.
                '\u{fffd}'
            } else {
                // Preserve printable content for subsequent Unicode-width measurement.
                character
            }
        })
        .collect()
}
