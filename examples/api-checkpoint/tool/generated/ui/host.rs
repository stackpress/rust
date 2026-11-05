//! Generated source host catalog; the application selects concrete route/event registrars.
/// Native callbacks traverse this catalog once in source order, with admin registered last.
pub const MODELS: &[(&str, &str)] = &[
    ("application", "Application"),
    ("session", "Session"),
    ("profile", "Profile"),
    ("auth", "Auth"),
];

/// Only source-authored labels/icons are published; no native model or codec metadata.
pub fn presentation() -> stackpress_framework_schema::__private::serde_json::Value {
    stackpress_framework_schema::__private::serde_json::json!({
        "application":{"label":"App","plural":"Apps","icon":"laptop"},
        "session":{"label":"Session","plural":"Sessions","icon":"coffee"},
        "profile":{"label":"Profile","plural":"Profiles","icon":"user"},
        "auth":{"label":"Auth","plural":"Auth","icon":"lock"},
    })
}
