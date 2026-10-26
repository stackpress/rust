//! Build explicit app-command permissions only for an actual Tauri app build.

/// The default native library tests need neither Tauri assets nor build tooling.
fn main() {
    #[cfg(feature = "app")]
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["profile_search", "cancel_profile_search"]),
    ))
    .expect("valid explicit Tauri app ACL");
}
