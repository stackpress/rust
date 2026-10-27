//! Finite native proof process; writes JSON to stdout and never opens a listener.

/// Render one closed fixture; any failure is a nonzero process result.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::json!({
            "html": stackpress_example_ui_format::render_html()?,
            "rows": stackpress_example_ui_format::proof_rows()?,
            "snapshot": stackpress_example_ui_format::snapshot_json()?,
            "edit_en": stackpress_example_ui_format::editing::render(stackpress_framework_ui::format::DisplayLocale::English),
            "edit_fr": stackpress_example_ui_format::editing::render(stackpress_framework_ui::format::DisplayLocale::French),
        })
    );
    Ok(())
}
