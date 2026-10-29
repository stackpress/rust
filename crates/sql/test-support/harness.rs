//! Test-only discovery/selection shared by the opt-in SQL executables.
//! Fixture ownership and environment-gated skip reporting remain in each caller.

/// Libtest-compatible selection, with no fixture or database side effects.
#[derive(Default)]
pub(crate) struct Harness {
    /// Discovery prints only protocol lines and never executes cases.
    list: bool,
    /// An exact filter matches the complete case name rather than a substring.
    exact: bool,
    /// Ignored-only discovery/execution uses the caller's current environment gate.
    ignored_only: bool,
    /// Positional test filters are alternatives, as in libtest.
    filters: Vec<String>,
    /// Each skip rejects matching cases after positive selection.
    skips: Vec<String>,
}

impl Harness {
    /// Parse the public harness arguments after private child modes have been dispatched.
    pub(crate) fn from_env() -> Self {
        Self::parse(std::env::args().skip(1))
    }

    /// Read option values separately so `terse` cannot become a test filter.
    fn parse(arguments: impl IntoIterator<Item = String>) -> Self {
        let mut harness = Self::default();
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--list" => harness.list = true,
                "--exact" => harness.exact = true,
                "--ignored" => harness.ignored_only = true,
                "--skip" => harness
                    .skips
                    .push(arguments.next().expect("--skip needs a name")),
                "--format" | "--color" | "--test-threads" => {
                    arguments.next().expect("harness option needs a value");
                }
                // Capture/verbosity never alter fixture execution. The generated-DDL
                // switch is interpreted by MySQL M2 before selecting its case registry.
                "--nocapture" | "--no-capture" | "--include-ignored" | "--quiet" | "-q"
                | "--g37-ddl" => {}
                option if option.starts_with('-') => panic!("unsupported harness option: {option}"),
                filter => harness.filters.push(filter.to_owned()),
            }
        }
        harness
    }

    /// Match one filter using the same exact/substring rule for includes and skips.
    fn matches(&self, name: &str, filter: &str) -> bool {
        if self.exact {
            name == filter
        } else {
            name.contains(filter)
        }
    }

    /// Select cases without suppressing ordinary cargo-test unset-URL skip messages.
    pub(crate) fn selects(&self, name: &str, ignored: bool) -> bool {
        // Ignored-only runs cannot execute enabled/live cases; default runs retain
        // disabled cases so their existing harness can report why they were skipped.
        (!self.ignored_only || ignored)
            && (self.filters.is_empty()
                || self.filters.iter().any(|filter| self.matches(name, filter)))
            && !self.skips.iter().any(|filter| self.matches(name, filter))
    }

    /// Return true after emitting discovery, including an empty ignored/filtered list.
    pub(crate) fn discover<'a>(&self, cases: impl IntoIterator<Item = (&'a str, bool)>) -> bool {
        if !self.list {
            return false;
        }
        // No summary, setup output, or blank lines may contaminate nextest's stdout.
        for (name, ignored) in cases {
            if self.selects(name, ignored) {
                println!("{name}: test");
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    /// Prove discovery options cannot leak into filters and both gate states are selectable.
    #[test]
    fn discovery_and_runtime_gates() {
        let all = super::Harness::parse(["--list", "--format", "terse"].map(str::to_owned));
        assert!(all.list);
        assert!(all.selects("m1::values", false));
        assert!(all.selects("m1::values", true));
        let ignored =
            super::Harness::parse(["--list", "--format", "terse", "--ignored"].map(str::to_owned));
        assert!(!ignored.selects("m2::sqlite_reference", false));
        assert!(ignored.selects("m1::values", true));
    }

    /// Exact selection runs only one named group, and unknown names select no fixture.
    #[test]
    fn exact_substring_and_skip_selection() {
        let exact =
            super::Harness::parse(["--exact", "m1::values", "--nocapture"].map(str::to_owned));
        assert!(exact.selects("m1::values", false));
        assert!(!exact.selects("m1::values_extra", false));
        assert!(!exact.selects("m1::reuse", false));
        let unknown = super::Harness::parse(["missing", "--exact"].map(str::to_owned));
        assert!(!unknown.selects("m1::values", false));
        let skip = super::Harness::parse(["m1::", "--skip", "reuse"].map(str::to_owned));
        assert!(skip.selects("m1::values", false));
        assert!(!skip.selects("m1::reuse", false));
        assert!(!skip.selects("m2::contract", false));
    }
}
