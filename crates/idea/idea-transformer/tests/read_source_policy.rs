//! All source entrypoints reject special files under an independent process watchdog.
use stackpress_idea_transformer::{ProjectLoader, SourceLoader, read_source};
use std::process::Command;
use std::time::Duration;

#[path = "support/watchdog.rs"]
mod watchdog;

/// Compare source entrypoints on regular, encoded, symlink and nonregular inputs.
#[test]
#[ignore = "run under source watchdog"]
fn source_policy_child() {
    let fixture = tempfile::tempdir().unwrap();
    let file = fixture.path().join("schema.idea");
    std::fs::write(&file, "model A {}\n").unwrap();
    assert_eq!(read_source(&file, 11).unwrap(), "model A {}\n");
    assert!(read_source(&file, 10).is_err());
    std::fs::write(&file, [0xff]).unwrap();
    assert!(read_source(&file, 11).is_err());
    assert!(read_source(fixture.path(), 11).is_err());
    let missing = read_source(&fixture.path().join("missing"), 11).unwrap_err();
    assert!(std::error::Error::source(&missing).is_some());
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        std::fs::write(&file, "model A {}\n").unwrap();
        let alias = fixture.path().join("alias.idea");
        symlink(&file, &alias).unwrap();
        assert_eq!(read_source(&alias, 11).unwrap(), "model A {}\n");

        let fifo = fixture.path().join("fifo.idea");
        assert!(
            Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            ProjectLoader::new(fixture.path())
                .unwrap()
                .load(None, "fifo.idea", 11)
                .is_err()
        );
        assert!(read_source(&fifo, 11).is_err());
        assert!(read_source(std::path::Path::new("/dev/null"), 11).is_err());
    }
}

/// A blocked open is terminated/reaped even when no executor thread can run a timer.
#[test]
fn source_policy_is_watchdog_bounded() {
    let result = watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "source_policy_child",
            "--nocapture",
        ]),
        "source-policy",
        Duration::from_secs(2),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
