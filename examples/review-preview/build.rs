//! Freeze the generated-output selection into each copied review executable.
/// Rebuild when the operator changes the variant; runtime flags cannot change compiled labels.
fn main() {
    println!("cargo:rerun-if-env-changed=STACKPRESS_REVIEW_VARIANT");
    println!("cargo:rustc-check-cfg=cfg(stackpress_review_release)");
    // Release review hosts are forbidden even if someone enables release debug assertions.
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        println!("cargo:rustc-cfg=stackpress_review_release");
    }
    let variant = std::env::var("STACKPRESS_REVIEW_VARIANT").unwrap_or_else(|_| "disabled".into());
    assert!(
        ["disabled", "plugin"].contains(&variant.as_str()),
        "invalid review build variant"
    );
    println!("cargo:rustc-env=STACKPRESS_REVIEW_VARIANT={variant}");
}
