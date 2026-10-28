//! Host-owned fixed preference commands; no caller filesystem path or credential.
use stackpress_framework_language::LocaleRegistry;
use stackpress_framework_view::adapters::native::PreferenceFile;
use stackpress_framework_view::preferences::{
    LoadedPreferences, NativePreferenceUpdate, Preferences,
};
use stackpress_lib::{Error, ErrorKind, Result};
use std::{path::PathBuf, sync::Mutex};

/// Lazy state leaves legacy desktop startup untouched until a preference command.
pub(crate) struct PreferencesHost {
    /// Trusted Tauri app-config path; command JSON cannot replace it.
    directory: PathBuf,
    /// Serialize actual file operations; no guard crosses an await or DOM callback.
    file: Mutex<Option<PreferenceFile>>,
}
impl PreferencesHost {
    /// Bind only a host-selected app configuration directory, without reading/writing it.
    pub(crate) fn new(directory: PathBuf) -> Self {
        // Defer all directory/file work until an explicitly admitted fixed command.
        Self {
            directory,
            file: Mutex::new(None),
        }
    }
    /// Prepare a bounded store only after authoritative command admission.
    fn with_file<T>(&self, operation: impl FnOnce(&mut PreferenceFile) -> Result<T>) -> Result<T> {
        // One host mutex serializes synchronous file work; no guard crosses an await.
        let mut state = self.file.lock().map_err(|_| {
            Error::new("Native preference state poisoned").with_kind(ErrorKind::External)
        })?;
        // First admitted access prepares trusted fixed storage; command JSON supplies no path.
        if state.is_none() {
            let registry = LocaleRegistry::new(
                vec!["en".into(), "fr".into()],
                "en".into(),
                Default::default(),
            )
            .map_err(|cause| failure("configure native locale registry", cause))?;
            *state = Some(
                PreferenceFile::new(
                    self.directory.clone(),
                    registry,
                    Preferences {
                        locale: "en".into(),
                        theme: None,
                    },
                )
                .map_err(|cause| failure("open native preference storage", cause))?,
            );
        }
        let file = state
            .as_mut()
            .ok_or_else(|| Error::new("Native preference state unavailable"))?;
        // Execute under the same exclusive lease, then release the guard on every return.
        operation(file)
    }
    /// Load before mounting; invalid files explicitly recover without being rewritten.
    pub(crate) fn load(&self) -> Result<String> {
        self.with_file(|file| {
            serde_json::to_string(&file.load())
                .map_err(|cause| failure("encode native preferences", cause))
        })
    }
    /// Validate the fixed DTO before touching storage, then return only an acknowledgement.
    pub(crate) fn update(&self, json: &str) -> Result<String> {
        // Bound and decode the closed command before any lazy directory creation.
        if json.len() > 1024 {
            return Err(Error::new("Native preference input exceeds limit"));
        }
        let update: NativePreferenceUpdate = serde_json::from_str(json)
            .map_err(|cause| failure("decode native preferences", cause))?;
        // A future/unknown revision cannot mutate revision-1 storage.
        if update.revision != 1 {
            return Err(Error::new("Native preference revision unsupported"));
        }
        self.with_file(|file| {
            let preferences = file
                .update(update.preferences)
                .map_err(|cause| failure("persist native preferences", cause))?;
            serde_json::to_string(&LoadedPreferences {
                revision: 1,
                preferences,
                recovered: false,
            })
            .map_err(|cause| failure("encode native preference acknowledgement", cause))
        })
    }
}
/// Context is added once, with a concrete cause retained outside public IPC strings.
fn failure(context: &'static str, cause: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::new(context)
        .with_kind(ErrorKind::External)
        .with_cause(cause)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Fixed DTO validation precedes directory creation and a real save survives reopen.
    #[test]
    fn command_fields_and_restart() {
        let root =
            std::env::temp_dir().join(format!("sp-command-preferences-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let host = PreferencesHost::new(root.clone());
        for input in [
            "{",
            r#"{"revision":2,"preferences":{"locale":"fr","theme":"dark"}}"#,
            r#"{"revision":1,"preferences":{"locale":"fr","theme":"dark"},"path":"/outside"}"#,
        ] {
            assert!(host.update(input).is_err());
        }
        assert!(!root.exists());
        assert!(host.update(&"x".repeat(1025)).is_err());
        let initial: LoadedPreferences = serde_json::from_str(&host.load().unwrap()).unwrap();
        assert_eq!(initial.preferences.locale, "en");
        let input = r#"{"revision":1,"preferences":{"locale":"fr","theme":"dark"}}"#;
        let text = host.update(input).unwrap();
        assert!(!text.contains("path"));
        drop(host);
        let reopened: LoadedPreferences =
            serde_json::from_str(&PreferencesHost::new(root.clone()).load().unwrap()).unwrap();
        assert_eq!(reopened.preferences.locale, "fr");
        // Native failure is observable without corrupting the external target.
        let destination = root.join(stackpress_framework_view::adapters::native::FILE_NAME);
        std::fs::remove_file(&destination).unwrap();
        let sentinel = root.join("sentinel");
        std::fs::write(&sentinel, "UNCHANGED").unwrap();
        std::os::unix::fs::symlink(&sentinel, &destination).unwrap();
        assert!(PreferencesHost::new(root.clone()).update(input).is_err());
        let recovered: LoadedPreferences =
            serde_json::from_str(&PreferencesHost::new(root.clone()).load().unwrap()).unwrap();
        assert!(recovered.recovered);
        assert_eq!(std::fs::read_to_string(sentinel).unwrap(), "UNCHANGED");
        std::fs::remove_dir_all(root).unwrap();
    }
}
