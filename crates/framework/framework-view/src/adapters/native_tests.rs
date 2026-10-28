//! Real private files and child-watchdog evidence for hostile special-file input.
use super::*;
use crate::preferences::ThemePreference;
use std::{
    collections::BTreeMap,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// Test names are isolated from the adapter's fixed bounded staging file.
static FIXTURE_SERIAL: AtomicU64 = AtomicU64::new(0);

/// Own one unique test directory and remove only its own children.
struct Fixture(PathBuf);
impl Fixture {
    /// Put all data outside source and build trees, with process/counter isolation.
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sp-preferences-{}-{}",
            std::process::id(),
            FIXTURE_SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    /// Construct the actual adapter with explicit English/light defaults.
    fn store(&self) -> PreferenceFile {
        PreferenceFile::new(
            self.0.clone(),
            LocaleRegistry::new(vec!["en".into(), "fr".into()], "en".into(), BTreeMap::new())
                .unwrap(),
            desired("en", ThemePreference::Light),
        )
        .unwrap()
    }
}
/// A crash leaves at most one fixed staging child, safely removed on reopen.
#[test]
fn abandoned_staging_is_bounded_and_never_followed() {
    let fixture = Fixture::new();
    let outside = Fixture::new();
    let sentinel = outside.0.join("sentinel");
    fs::write(&sentinel, b"UNCHANGED").unwrap();
    std::os::unix::fs::symlink(&sentinel, fixture.0.join(TEMP_NAME)).unwrap();
    let mut store = fixture.store();
    assert!(!fixture.0.join(TEMP_NAME).exists());
    assert_eq!(fs::read(&sentinel).unwrap(), b"UNCHANGED");
    fs::write(fixture.0.join(TEMP_NAME), b"occupied").unwrap();
    assert!(store.update(desired("fr", ThemePreference::Dark)).is_err());
    drop(store);
    let mut store = fixture.store();
    store.update(desired("fr", ThemePreference::Dark)).unwrap();
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 1);
}
impl Drop for Fixture {
    /// Symlink removal never follows a fixture's external target.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Readable intended settings for independent persistence assertions.
fn desired(locale: &str, theme: ThemePreference) -> Preferences {
    Preferences {
        locale: locale.into(),
        theme: Some(theme),
    }
}

/// First-install defaults, atomic save, reopen, private mode and explicit invalid input.
#[test]
fn real_restart_and_invalid_updates() {
    let fixture = Fixture::new();
    let mut store = fixture.store();
    assert!(!store.load().recovered);
    let expected = desired("fr", ThemePreference::Dark);
    assert_eq!(store.update(expected.clone()).unwrap(), expected);
    assert_eq!(fixture.store().load().preferences, expected);
    assert_eq!(
        fs::metadata(fixture.0.join(FILE_NAME)).unwrap().mode() & 0o777,
        0o600
    );
    assert!(
        store
            .update(desired("unknown", ThemePreference::Light))
            .is_err()
    );
    assert_eq!(fixture.store().load().preferences, expected);
    // Legacy System is unset, without resetting French or rewriting the file.
    let legacy = br#"{"revision":1,"preferences":{"locale":"fr","theme":"system"}}"#;
    fs::write(fixture.0.join(FILE_NAME), legacy).unwrap();
    assert_eq!(
        fixture.store().load().preferences,
        Preferences {
            locale: "fr".into(),
            theme: None
        }
    );
    assert!(!fixture.store().load().recovered);
    assert_eq!(fs::read(fixture.0.join(FILE_NAME)).unwrap(), legacy);
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 1);
}
/// Corrupt, truncated, oversized, wrong-version and unknown-field data recover explicitly.
#[test]
fn corrupt_inputs_are_bounded_and_not_rewritten() {
    let fixture = Fixture::new();
    let store = fixture.store();
    for bytes in [
        Vec::new(),
        b"{".to_vec(),
        vec![b'x'; 1025],
        br#"{"revision":2,"preferences":{"locale":"fr","theme":"dark"}}"#.to_vec(),
        br#"{"revision":1,"preferences":{"locale":"de","theme":"dark"}}"#.to_vec(),
        br#"{"revision":1,"preferences":{"locale":"fr","theme":"dark"},"path":"/outside"}"#
            .to_vec(),
    ] {
        fs::write(fixture.0.join(FILE_NAME), &bytes).unwrap();
        let loaded = store.load();
        assert!(loaded.recovered);
        assert_eq!(loaded.preferences, desired("en", ThemePreference::Light));
        assert_eq!(fs::read(fixture.0.join(FILE_NAME)).unwrap(), bytes);
    }
}
/// A symlink cannot disclose or overwrite its target; directory substitutions are rejected.
#[test]
fn symlinks_and_non_regular_files_are_not_followed() {
    let fixture = Fixture::new();
    let outside = Fixture::new();
    let sentinel = outside.0.join("sentinel");
    fs::write(&sentinel, b"PRIVATE_OUTSIDE_SENTINEL").unwrap();
    let mut store = fixture.store();
    std::os::unix::fs::symlink(&sentinel, fixture.0.join(FILE_NAME)).unwrap();
    assert!(store.load().recovered);
    assert!(store.update(desired("fr", ThemePreference::Dark)).is_err());
    assert_eq!(fs::read(&sentinel).unwrap(), b"PRIVATE_OUTSIDE_SENTINEL");
    fs::remove_file(fixture.0.join(FILE_NAME)).unwrap();
    fs::create_dir(fixture.0.join(FILE_NAME)).unwrap();
    assert!(store.load().recovered);
    assert!(store.update(desired("fr", ThemePreference::Dark)).is_err());
    let alias = outside.0.join("alias");
    std::os::unix::fs::symlink(&fixture.0, &alias).unwrap();
    assert!(PreferenceFile::new(alias, store.registry.clone(), store.defaults.clone()).is_err());
}
/// Exercise a FIFO under a parent deadline; a blocking open is a failed proof.
#[test]
fn fifo_open_cannot_hang() {
    if std::env::var_os("SP_PREFERENCES_FIFO_CHILD").is_some() {
        let fixture = Fixture::new();
        let store = fixture.store();
        assert!(
            Command::new("mkfifo")
                .arg(fixture.0.join(FILE_NAME))
                .status()
                .unwrap()
                .success()
        );
        assert!(store.load().recovered);
        println!("FIFO_COMPLETED");
        return;
    }
    let executable = std::env::current_exe().unwrap();
    let mut child = Command::new(executable)
        .args([
            "adapters::native::tests::fifo_open_cannot_hang",
            "--exact",
            "--nocapture",
        ])
        .env("SP_PREFERENCES_FIFO_CHILD", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("FIFO child timed out; killed and reaped");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("FIFO_COMPLETED")
    );
}
