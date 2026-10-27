//! Closed, finite collision policy shared by native tests and browser measurement.
use super::ControlError;
/// Place below an anchor, flip above if needed, then clamp within an eight-pixel inset.
/// # Errors
/// Rejects non-finite geometry, negative sizes and unusably small viewports.
pub(crate) fn place_popup(
    anchor: [f64; 4],
    popup: [f64; 2],
    viewport: [f64; 2],
) -> Result<[f64; 2], ControlError> {
    if anchor
        .iter()
        .chain(popup.iter())
        .chain(viewport.iter())
        .any(|v| !v.is_finite())
        || anchor[2] < 0.0
        || anchor[3] < 0.0
        || popup.iter().any(|v| *v < 0.0)
        || viewport.iter().any(|v| *v < 16.0)
    {
        return Err(ControlError);
    }
    // Prefer below, flip above if constrained, then clamp both axes to the viewport inset.
    let below = anchor[1] + anchor[3] + 4.0;
    let top = if below + popup[1] > viewport[1] - 8.0 {
        anchor[1] - popup[1] - 4.0
    } else {
        below
    };
    Ok([
        anchor[0].clamp(8.0, (viewport[0] - popup[0] - 8.0).max(8.0)),
        top.clamp(8.0, (viewport[1] - popup[1] - 8.0).max(8.0)),
    ])
}

/// Multiply at most 64 computed CSS zoom values without using rounded element sizes.
/// # Errors
/// Rejects non-finite/nonpositive values, unsupported units and excessive traversal.
pub(crate) fn effective_zoom(values: &[String]) -> Result<f64, ControlError> {
    if values.len() > 64 {
        return Err(ControlError);
    }
    // Computed zoom values are non-inherited; multiply the finite ancestor chain once.
    let mut scale = 1.0;
    for value in values {
        let value = value.trim();
        let zoom = if value == "normal" || value.is_empty() {
            1.0
        } else if let Some(number) = value.strip_suffix('%') {
            number.parse::<f64>().map_err(|_| ControlError)? / 100.0
        } else {
            value.parse::<f64>().map_err(|_| ControlError)?
        };
        if !zoom.is_finite() || zoom <= 0.0 {
            return Err(ControlError);
        }
        scale *= zoom;
        if !scale.is_finite() || !(0.0001..=10_000.0).contains(&scale) {
            return Err(ControlError);
        }
    }
    Ok(scale)
}
