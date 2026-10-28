//! Build explicit app-command permissions only for an actual Tauri app build.

/// The default native library tests need neither Tauri assets nor build tooling.
fn main() {
    // A profile gate also rejects release builds that enable debug assertions.
    #[cfg(feature = "review-preview")]
    assert_eq!(
        std::env::var("PROFILE").as_deref(),
        Ok("debug"),
        "review-preview is restricted to debug builds"
    );
    #[cfg(feature = "app")]
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "profile_search",
            "cancel_profile_search",
            "load_preferences",
            "update_preferences",
        ]),
    ))
    .expect("valid explicit Tauri app ACL");
}
