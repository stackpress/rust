//! Moment's English and locally registered short relative-time thresholds.

/// Relative time for (target - now) seconds, matching Frui's Moment defaults.
/// Frui imports Moment core, so unregistered locale names retain English.
pub fn source_relative(seconds: f64, short: bool) -> String {
    if !seconds.is_finite() {
        return "Invalid date".into();
    }
    let elapsed = seconds.abs();
    let sec = elapsed.round();
    let min = (elapsed / 60.0).round();
    let hours = (elapsed / 3600.0).round();
    let days = (elapsed / 86400.0).round();
    let months = (elapsed / 86400.0 * 4800.0 / 146097.0).round();
    let years = (elapsed / 86400.0 * 4800.0 / 146097.0 / 12.0).round();
    // Moment chooses the first matching rounded threshold; weeks are disabled by default.
    let (key, number) = if sec <= 44.0 {
        ("s", sec)
    } else if min <= 1.0 {
        ("m", 1.0)
    } else if min < 45.0 {
        ("mm", min)
    } else if hours <= 1.0 {
        ("h", 1.0)
    } else if hours < 22.0 {
        ("hh", hours)
    } else if days <= 1.0 {
        ("d", 1.0)
    } else if days < 26.0 {
        ("dd", days)
    } else if months <= 1.0 {
        ("M", 1.0)
    } else if months < 11.0 {
        ("MM", months)
    } else if years <= 1.0 {
        ("y", 1.0)
    } else {
        ("yy", years)
    };
    let text = if short {
        match key {
            "s" => "now".into(),
            "m" => "1m".into(),
            "mm" => format!("{number}m"),
            "h" => "1h".into(),
            "hh" => format!("{number}h"),
            "d" => "1d".into(),
            "dd" => format!("{number}d"),
            "M" => "1M".into(),
            "MM" => format!("{number}M"),
            "y" => "1y".into(),
            _ => format!("{number}y"),
        }
    } else {
        match key {
            "s" => "a few seconds".into(),
            "m" => "a minute".into(),
            "mm" => format!("{number} minutes"),
            "h" => "an hour".into(),
            "hh" => format!("{number} hours"),
            "d" => "a day".into(),
            "dd" => format!("{number} days"),
            "M" => "a month".into(),
            "MM" => format!("{number} months"),
            "y" => "a year".into(),
            _ => format!("{number} years"),
        }
    };
    if short {
        if seconds > 0.0 {
            format!("-{text}")
        } else {
            text
        }
    } else if seconds > 0.0 {
        format!("in {text}")
    } else {
        format!("{text} ago")
    }
}
