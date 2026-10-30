//! Pure selected API contracts/UI; native tool stores never enter this shared module.
#[path = "../../tool/generated/contracts/mod.rs"]
#[allow(
    missing_docs,
    reason = "generated aggregate modules are owned by the writer"
)]
pub mod contracts;
#[path = "../../tool/generated/ui/mod.rs"]
#[allow(
    missing_docs,
    reason = "generated aggregate modules are owned by the writer"
)]
pub mod ui;
/// Direct-library bridge to exactly the selected generated functions.
pub fn management() -> stackpress_framework_api::views::management::Ui {
    stackpress_framework_api::views::management::Ui {
        spec: ui::pages::spec,
        form: ui::pages::form,
        filters: ui::pages::filters,
        table: ui::pages::table,
        detail: ui::pages::detail,
    }
}
