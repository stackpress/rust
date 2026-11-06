//! Source carousel/film metadata adaptation without loaders, timers or added URL policy.
use super::countries::slot;
use super::props::{appearance, component};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::source::{SourceImageCarousel, SourceImageFilm};

/// Image collection display through the actual shared film/carousel components.
#[component]
pub fn MediaView(
    /// Source URLs, overflow flags and authored style slots.
    context: RoleContext,
) -> impl IntoView {
    let value = match context.value.get() {
        Presence::Value(Value::Array(values)) => {
            values.iter().map(super::metadata_string).collect()
        }
        _ => Vec::new(),
    };
    if component(&context).ends_with("carousel") {
        // Frui ImageCarousel destructures frame but does not forward it to Carousel.
        view!{<SourceImageCarousel value default_index=context.number_prop("defaultIndex",0.0) repeat=context.bool_prop("repeat",false) auto=context.bool_prop("auto",false) hidden=context.bool_prop("hidden",false) scroll=context.bool_prop("scroll",false) appearance=appearance(&context) film=slot(&context,"film") image=slot(&context,"image")/>}.into_any()
    } else {
        view!{<SourceImageFilm value appearance=appearance(&context) frame=slot(&context,"frame") image=slot(&context,"image")/>}.into_any()
    }
}
