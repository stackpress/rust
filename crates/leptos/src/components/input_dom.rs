//! Thin native select event reader. Admission/state policy stays in inputs.rs.
use wasm_bindgen::JsCast;
/// Copy current native selections without holding reactive state during DOM calls.
pub(super) fn selected_values(event: &leptos::ev::Event) -> Vec<String> {
    let Some(select) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
    else {
        return Vec::new();
    };
    let options = select.selected_options();
    (0..options.length())
        .filter_map(|index| options.item(index))
        .filter_map(|element| element.dyn_into::<web_sys::HtmlOptionElement>().ok())
        .map(|option| option.value())
        .collect()
}
