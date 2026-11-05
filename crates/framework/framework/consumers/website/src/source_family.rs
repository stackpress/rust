//! Only portable generated adapters enter this artifact; native source/stores/client are absent.
#[path = "../../generated-source/ui/mod.rs"]
pub mod ui;

/// This finite application's root explicitly permits visible GET detail links.
/// It grants no route/event authority and is shared by SSR, hydration and CSR.
pub fn ui() -> stackpress_framework_admin::source::pages::Ui {
    stackpress_framework_admin::source::pages::Ui {
        table,
        ..ui::pages::ui()
    }
}
/// Reuse the shared trailing action column; the application supplies its presentation policy.
fn table(
    module: &str,
    rows: &serde_json::Value,
    instance: &str,
) -> stackpress_framework_schema::SchemaResult<leptos::prelude::AnyView> {
    use leptos::prelude::*;
    provide_context(stackpress_framework_admin::row_actions::RowActions::new(
        "/admin",
        Callback::new(|_| true),
    ));
    ui::pages::table(module, rows, instance)
}
