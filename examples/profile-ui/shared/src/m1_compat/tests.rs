//! Temporary compatibility behavior, removed with the shim at integration.

use super::*;

/// Mounts join/strip only at exact boundaries and never become caller-supplied schemes.
#[test]
fn checked_mount_and_scoped_context() {
    assert_eq!(BasePath::new("/").unwrap(), BasePath::default());
    let base = BasePath::new("/office/team").unwrap();
    assert_eq!(base.as_str(), "/office/team");
    assert_eq!(
        base.join("/ui/profiles").unwrap().as_str(),
        "/office/team/ui/profiles"
    );
    assert_eq!(base.strip("/office/team/ui/profiles"), Some("/ui/profiles"));
    assert!(base.strip("/office/teams/ui/profiles").is_none());
    assert!(base.strip("/office/team//evil").is_none());
    assert!(base.join("//evil").is_err());
    assert!(base.join("javascript:x").is_err());
    assert!(base.join("/x\\evil").is_err());
    assert!(base.join(&format!("/{}", "x".repeat(4096))).is_err());
    for invalid in [
        "relative",
        "//evil",
        "/../evil",
        "/office/",
        "/with space",
        "/a?b",
    ] {
        assert!(BasePath::new(invalid).is_err());
    }
    let owner = Owner::new();
    owner.with(|| {
        assert_eq!(use_route_context().base, BasePath::default());
        provide_route_context(base.clone());
        assert_eq!(use_route_context().base, base);
    });
    owner.cleanup();
}
