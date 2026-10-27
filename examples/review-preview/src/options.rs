//! Explicit finite launch inputs, shared by listener-free proof and devmetrics entry points.
use stackpress_framework_view::BasePath;
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

/// Validated operator inputs; browser fields never select paths or origin.
#[derive(Clone, Debug)]
pub struct Options {
    /// Required nonzero loopback port, even for port-free transport simulation.
    pub port: u16,
    /// Checked outer application mount.
    pub base: BasePath,
    /// Existing owner-selected output directory.
    pub directory: PathBuf,
    /// Matching browser renderer mode.
    pub csr: bool,
    /// Compiled bundle selection.
    pub variant: String,
    /// Independent process lifetime at most two hours.
    pub duration: Duration,
}
impl Options {
    /// Parse exact pairs and reject duplicate, unknown or unbounded inputs.
    /// # Errors
    /// Returns invalid launch configuration; there is no default port/directory.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        // Parse the closed operator vocabulary before publishing any paths or host state.
        let mut values = BTreeMap::new();
        for pair in args.chunks(2) {
            if pair.len() != 2
                || ![
                    "--port",
                    "--base",
                    "--directory",
                    "--render",
                    "--variant",
                    "--duration-seconds",
                ]
                .contains(&pair[0].as_str())
                || values.insert(pair[0].as_str(), pair[1].as_str()).is_some()
            {
                return Err("Expected unique named launch pairs".into());
            }
        }
        let port = values
            .get("--port")
            .ok_or("Explicit port required")?
            .parse::<u16>()
            .map_err(|_| "Invalid port")?;
        let seconds = values
            .get("--duration-seconds")
            .copied()
            .unwrap_or("7200")
            .parse::<u64>()
            .map_err(|_| "Invalid duration")?;
        if port == 0 || !(1..=7200).contains(&seconds) {
            return Err("Port/duration out of range".into());
        }
        // The prepared browser module and compiled native selection must agree.
        let mode = values.get("--render").copied().unwrap_or("hydrate");
        let variant = values.get("--variant").copied().unwrap_or("disabled");
        if !["hydrate", "csr"].contains(&mode) || !["disabled", "plugin"].contains(&variant) {
            return Err("Unknown render/variant".into());
        }
        Ok(Self {
            port,
            base: BasePath::new(values.get("--base").copied().unwrap_or(""))
                .map_err(|e| e.to_string())?,
            directory: PathBuf::from(
                values
                    .get("--directory")
                    .ok_or("Explicit directory required")?,
            ),
            csr: mode == "csr",
            variant: variant.into(),
            duration: Duration::from_secs(seconds),
        })
    }
    /// The startup-selected exact origin; no request header participates.
    pub fn origin(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
    /// Join an application-owned path to the validated outer base.
    pub fn path(&self, suffix: &str) -> String {
        format!("{}{suffix}", self.base.as_str())
    }
}
