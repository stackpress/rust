//! Frui ColorInput conversion and synchronized native picker/text input.
use super::{COLORS, SourceAppearance};
use leptos::prelude::*;

/// Normalize named/short/long hex and rgb/rgba to Frui's native six-digit picker value.
/// Unsupported color spellings fall back to #000000; the authored text is still retained.
pub fn source_color_hex(color: &str) -> String {
    if let Some((_, hex)) = COLORS
        .iter()
        .find(|(name, _)| *name == color.to_lowercase())
    {
        return (*hex).into();
    }
    let hex = color.strip_prefix('#').unwrap_or(color);
    if matches!(hex.len(), 3 | 6) && hex.bytes().all(|c| c.is_ascii_hexdigit()) {
        let hex = if hex.len() == 3 {
            hex.chars().flat_map(|c| [c, c]).collect::<String>()
        } else {
            hex.into()
        };
        return format!("#{}", hex.to_lowercase());
    }
    let lower = color.to_lowercase();
    let rgb = lower
        .strip_prefix("rgb(")
        .or_else(|| lower.strip_prefix("rgba("));
    if let Some(rgb) = rgb.and_then(|rgb| rgb.strip_suffix(')')) {
        let parts = rgb.split(',').collect::<Vec<_>>();
        let alpha = parts.len() == 4;
        if matches!(parts.len(), 3 | 4) {
            let mut channels = Vec::new();
            for (index, part) in parts[..3].iter().enumerate() {
                let part = if index > 0 { part.trim_start() } else { part };
                if part.is_empty() || part.len() > 3 || !part.bytes().all(|c| c.is_ascii_digit()) {
                    return "#000000".into();
                }
                channels.push(part.parse::<u16>().unwrap_or(0).min(255));
            }
            if alpha {
                let a = parts[3].trim_start();
                if a.is_empty() || !a.bytes().all(|c| c.is_ascii_digit() || c == b'.') {
                    return "#000000".into();
                }
                // parseFloat accepts a numeric prefix, but a dot-only alpha is NaN upstream.
                let mut dot = false;
                let prefix = a
                    .chars()
                    .take_while(|c| {
                        if *c == '.' {
                            if dot {
                                false
                            } else {
                                dot = true;
                                true
                            }
                        } else {
                            true
                        }
                    })
                    .collect::<String>();
                if prefix.parse::<f64>().is_err() {
                    return "#000000".into();
                }
            }
            return format!("#{:02x}{:02x}{:02x}", channels[0], channels[1], channels[2]);
        }
    }
    "#000000".into()
}

/// Synchronized source color picker and permissive text, without additional color validation.
#[component]
pub fn SourceColorInput(
    /// Native text identity.
    #[prop(into)]
    id: String,
    /// Native successful text key; picker remains unnamed.
    #[prop(into)]
    name: String,
    /// Root-owned authored color, including unrecognized strings.
    value: RwSignal<String>,
    /// Source onUpdate callback from either native input.
    #[prop(optional)]
    on_update: Option<Callback<String>>,
    /// Root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
    /// Text slot appearance.
    #[prop(optional)]
    input: SourceAppearance,
    /// Picker slot appearance.
    #[prop(optional)]
    picker: SourceAppearance,
    /// Native requiredness on the text control.
    #[prop(optional)]
    required: bool,
) -> impl IntoView {
    let publish = Callback::new(move |text: String| {
        value.set(text.clone());
        if let Some(callback) = on_update {
            callback.run(text);
        }
    });
    view! {<div class=format!("sp-source-color-input {}",appearance.class_name) style=appearance.style>
        <input type="color" aria-label="Choose color" class=picker.class_name style=picker.style value=move||source_color_hex(&value.get()) prop:value=move||source_color_hex(&value.get()) on:input=move|event|publish.run(event_target_value(&event))/>
        <input class=format!("sp-input {}",input.class_name) style=input.style type="text" id=id name=name required=required value=move||value.get() prop:value=move||value.get() on:input=move|event|publish.run(event_target_value(&event))/>
    </div>}
}
