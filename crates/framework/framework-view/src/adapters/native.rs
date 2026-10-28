//! Fixed nonsecret app-config file; no caller path, shell, identity or credentials.
//!
//! The host must exclusively own this private directory and serialize its store.
//! This is not a cooperative external-writer database. Open uses no-follow and
//! nonblocking flags, then fstat-equivalent metadata before any bounded read.
//! Updates sync a create-new temporary file, atomically rename, then sync the
//! directory. A late sync error is unconfirmed persistence, never rollback.
use crate::preferences::{LoadedPreferences, Preferences};
use serde::{Deserialize, Serialize};
use stackpress_framework_language::LocaleRegistry;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::PathBuf,
};

/// File ceiling applies before reading and again while reading/serializing.
pub const MAX_FILE_BYTES: u64 = 1024;
/// Fixed basename, independent of command JSON and all public preferences.
pub const FILE_NAME: &str = "ui-preferences-v1.json";
/// A single fixed staging name bounds leftovers even after repeated process crashes.
const TEMP_NAME: &str = ".ui-preferences-v1.pending";

/// Safe public failure with an optional concrete native I/O source.
#[derive(Debug)]
pub struct FileError(Option<io::Error>);
impl std::fmt::Display for FileError {
    /// Never disclose paths, private causes or file contents to IPC callers.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Preferences unavailable")
    }
}
impl std::error::Error for FileError {
    /// Native diagnostics may inspect a cause; public serialization does not exist.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.as_ref().map(|error| error as _)
    }
}
impl From<io::Error> for FileError {
    /// Retain the concrete I/O cause without interpolating its Display.
    fn from(error: io::Error) -> Self {
        Self(Some(error))
    }
}

/// Versioned file data never bypasses registry and theme admission after decoding.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    revision: u32,
    #[serde(deserialize_with = "saved_preferences")]
    preferences: Preferences,
}

/// Decode legacy System only while reading saved data, preserving other settings.
fn saved_preferences<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Preferences, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Saved {
        locale: String,
        #[serde(default)]
        theme: Option<String>,
    }
    let saved = Saved::deserialize(deserializer)?;
    // Unknown values still trigger the adapter's existing corrupt-file recovery.
    let theme = match saved.theme.as_deref() {
        None | Some("system") => None,
        Some(value) => Some(
            crate::preferences::ThemePreference::parse(value)
                .ok_or_else(|| serde::de::Error::custom("invalid saved theme"))?,
        ),
    };
    Ok(Preferences {
        locale: saved.locale,
        theme,
    })
}

/// Exclusive app-owned store, supplied a trusted app-config directory by its host.
pub struct PreferenceFile {
    /// Only fixed child names are used under this private host-controlled directory.
    directory: PathBuf,
    /// Immutable canonical locale authority.
    registry: LocaleRegistry,
    /// Explicit fallback, including first-install behavior.
    defaults: Preferences,
}
impl PreferenceFile {
    /// Prepare a private directory without following a final-component symlink.
    ///
    /// # Errors
    /// Rejects invalid defaults, symlink/non-directory or group/other-writable root,
    /// and I/O failures. Trusted OS ancestors must remain under host ownership.
    pub fn new(
        directory: PathBuf,
        registry: LocaleRegistry,
        defaults: Preferences,
    ) -> Result<Self, FileError> {
        if registry.canonical(&defaults.locale) != Some(defaults.locale.as_str()) {
            return Err(FileError(None));
        }
        // Create only the host-selected app-config path, never a path from invoke.
        fs::create_dir_all(&directory)?;
        let metadata = fs::symlink_metadata(&directory)?;
        if !metadata.is_dir() || metadata.mode() & 0o022 != 0 {
            return Err(FileError(None));
        }
        // Exclusive host ownership makes an old staging name abandoned at startup.
        // Unlink the name itself, including a substituted symlink, without reading it.
        match fs::remove_file(directory.join(TEMP_NAME)) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(Self {
            directory,
            registry,
            defaults,
        })
    }
    /// Load checked settings; missing is normal, invalid/unreadable input recovers.
    /// No corrupt data or symlink target is removed or rewritten during a read.
    pub fn load(&self) -> LoadedPreferences {
        let result = self.read();
        let (preferences, recovered) = match result {
            Ok(Some(preferences)) => (preferences, false),
            Ok(None) => (self.defaults.clone(), false),
            Err(_) => (self.defaults.clone(), true),
        };
        LoadedPreferences {
            revision: 1,
            preferences,
            recovered,
        }
    }
    /// Validate a canonical locale without regional/case/alias guessing in a file.
    fn valid(&self, preferences: &Preferences) -> bool {
        self.registry.canonical(&preferences.locale) == Some(preferences.locale.as_str())
    }
    /// Open without following symlinks or waiting on a substituted FIFO/device.
    fn read(&self) -> Result<Option<Preferences>, FileError> {
        let opened = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(self.directory.join(FILE_NAME));
        let file = match opened {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        // This is metadata on the actual opened handle, not a stale path check.
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
            return Err(FileError(None));
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(FileError(None));
        }
        let stored: Stored = serde_json::from_slice(&bytes).map_err(|_| FileError(None))?;
        if stored.revision != 1 || !self.valid(&stored.preferences) {
            return Err(FileError(None));
        }
        Ok(Some(stored.preferences))
    }
    /// Atomically persist validated settings; publish only the returned acknowledgement.
    ///
    /// # Errors
    /// Rejects invalid locale, symlink/non-regular destination, excess serialization,
    /// occupied staging name or I/O/sync failure. A failure after
    /// rename may still be saved; no automatic retry or rollback is promised.
    pub fn update(&mut self, preferences: Preferences) -> Result<Preferences, FileError> {
        if !self.valid(&preferences) {
            return Err(FileError(None));
        }
        let destination = self.directory.join(FILE_NAME);
        match fs::symlink_metadata(&destination) {
            Ok(metadata) if !metadata.is_file() => return Err(FileError(None)),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let bytes = serde_json::to_vec(&Stored {
            revision: 1,
            preferences: preferences.clone(),
        })
        .map_err(|_| FileError(None))?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(FileError(None));
        }
        let (mut file, temporary) = self.temporary()?;
        // The cleanup lease removes only this create-new temporary name on any exit.
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary.0, destination)?;
        File::open(&self.directory)?.sync_all()?;
        Ok(preferences)
    }
    /// Create the fixed staging child exclusively, mode 0600, with no-follow flags.
    fn temporary(&self) -> Result<(File, Temporary), FileError> {
        let path = self.directory.join(TEMP_NAME);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)?;
        Ok((file, Temporary(path)))
    }
}
/// Remove a temporary name after error or rename; never remove the destination.
struct Temporary(PathBuf);
impl Drop for Temporary {
    /// Best-effort cleanup does not replace an original write/sync failure.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
#[path = "native_tests.rs"]
mod tests;
