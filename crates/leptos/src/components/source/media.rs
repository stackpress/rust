//! Source image film/carousel layout; auto means overflow:auto, never a timer.
use super::SourceAppearance;
use leptos::prelude::*;

/// Source carousel navigation preserves arbitrary initial indices and original wrap boundaries.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceCarouselState {
    /// Current index; Frui does not clamp an authored defaultIndex.
    pub index: f64,
    /// Current frame count.
    pub frames: usize,
    /// Repeat at either boundary.
    pub repeat: bool,
}
impl SourceCarouselState {
    /// Apply the original previous/next transition, including an empty repeating collection.
    pub fn step(&mut self, next: bool) {
        let proposed = self.index + if next { 1.0 } else { -1.0 };
        self.index = if next && proposed >= self.frames as f64 {
            if self.repeat { 0.0 } else { self.index }
        } else if !next && proposed < 0.0 {
            if self.repeat {
                self.frames as f64 - 1.0
            } else {
                self.index
            }
        } else {
            proposed
        };
    }
}

/// Native source image film, with unrestricted src values and original slot styling.
#[component]
pub fn SourceImageFilm(
    /// Image URLs in source order, including duplicates.
    value: Vec<String>,
    /// Film root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
    /// Per-frame source slot.
    #[prop(optional)]
    frame: SourceAppearance,
    /// Per-image source slot.
    #[prop(optional)]
    image: SourceAppearance,
) -> impl IntoView {
    view! {<div class=format!("sp-source-film {}",appearance.class_name) style=appearance.style>
        {value.into_iter().map(|src|view!{<div class=frame.class_name.clone() style=frame.style.clone()><img src=src alt="" class=image.class_name.clone() style=image.style.clone()/></div>}).collect_view()}
    </div>}
}

/// Source image carousel: native scroll, initial index and original overflow flags/slots.
/// The source role supplies no Previous/Next children, so no extra arrow controls are imposed.
#[component]
pub fn SourceImageCarousel(
    /// Image URLs in authored order.
    value: Vec<String>,
    /// Original unbounded defaultIndex.
    #[prop(optional)]
    default_index: f64,
    /// Original boundary policy, available to callers using the shared controller.
    #[prop(optional)]
    repeat: bool,
    /// Source overflow:auto switch (not timed advancement).
    #[prop(optional)]
    auto: bool,
    /// Source overflow:hidden switch.
    #[prop(optional)]
    hidden: bool,
    /// Source overflow:scroll switch.
    #[prop(optional)]
    scroll: bool,
    /// Optional shared external navigation controller.
    #[prop(optional_no_strip)]
    state: Option<RwSignal<SourceCarouselState>>,
    /// Root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
    /// Film slot appearance.
    #[prop(optional)]
    film: SourceAppearance,
    /// Image slot appearance.
    #[prop(optional)]
    image: SourceAppearance,
) -> impl IntoView {
    let state = state.unwrap_or_else(|| {
        RwSignal::new(SourceCarouselState {
            index: default_index,
            frames: value.len(),
            repeat,
        })
    });
    let viewport = NodeRef::<leptos::html::Div>::new();
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        use wasm_bindgen::JsCast;
        let index = state.with(|state| state.index);
        if index < 0.0 || index.fract() != 0.0 {
            return;
        }
        let Some(viewport) = viewport.get() else {
            return;
        };
        let Some(frame) = viewport
            .query_selector_all("[data-source-frame]")
            .ok()
            .and_then(|frames| frames.get(index as u32))
            .and_then(|frame| frame.dyn_into::<leptos::web_sys::Element>().ok())
        else {
            return;
        };
        let offset =
            frame.get_bounding_client_rect().left() - viewport.get_bounding_client_rect().left();
        let options = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&options, &"left".into(), &offset.into());
        let _ = js_sys::Reflect::set(&options, &"behavior".into(), &"smooth".into());
        if let Ok(scroll) = js_sys::Reflect::get(viewport.as_ref(), &"scrollBy".into())
            .and_then(|value| value.dyn_into::<js_sys::Function>())
        {
            let _ = scroll.call1(viewport.as_ref(), &options);
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    let _ = state;
    let overflow = if auto {
        "auto"
    } else if scroll {
        "scroll"
    } else if hidden {
        "hidden"
    } else {
        "visible"
    };
    view! {<div class=format!("sp-source-carousel {}",appearance.class_name) style=appearance.style>
        <div node_ref=viewport tabindex="0" role="region" aria-label="Image carousel" class="sp-source-carousel-viewport" style=format!("overflow:{overflow}")>
            <div class=format!("sp-source-carousel-film {}",film.class_name) style=film.style>
                {value.into_iter().map(|src|view!{<div data-source-frame="" class="sp-source-carousel-frame"><img src=src alt="" class=image.class_name.clone() style=image.style.clone()/></div>}).collect_view()}
            </div>
        </div>
    </div>}
}
