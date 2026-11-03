//! Root-owned row-link presentation. This callback grants no authority to a route or event.

use leptos::prelude::*;
use serde_json::{Map, Value};
use stackpress_leptos::components::ChevronRight;

/// Public row identity passed to an application's optional detail URL adapter.
#[derive(Clone, Debug)]
pub struct RowDetail {
    /// Generated model route module.
    pub model: String,
    /// Complete public identity, independent of the caller's DOM row key.
    pub key: Value,
    /// Source detail path with identity members in schema order.
    pub path: String,
}

/// Explicit root context consumed by generated tables without changing their signatures.
///
/// `can` receives `("GET", full_detail_route)` for each row. The source action column
/// remains present when the callback returns false. Without this context, tables retain
/// their empty action cells; applications have supplied no visible link grant.
/// Protected routes and events still enforce their own policies independently.
///
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_admin::row_actions::RowActions;
/// let owner = Owner::new();
/// owner.with(|| {
///     provide_context(RowActions::new("/admin", Callback::new(|(method, route): (String, String)| {
///         method == "GET" && route == "/admin/profile/detail/p1"
///     })));
/// });
/// owner.cleanup();
/// ```
#[derive(Clone)]
pub struct RowActions {
    base: String,
    can: Callback<(String, String), bool>,
    href: Option<Callback<RowDetail, String>>,
}

impl RowActions {
    /// Select the source base URL and application-owned route visibility callback.
    /// The base is used verbatim, just as in the source `${base}/model/detail/id` template.
    pub fn new(base: impl Into<String>, can: Callback<(String, String), bool>) -> Self {
        Self {
            base: base.into(),
            can,
            href: None,
        }
    }

    /// Adapt the link target for an existing host while retaining the source permission check.
    /// The default target is relative `detail/<ordered identity path>`.
    pub fn with_href(mut self, href: Callback<RowDetail, String>) -> Self {
        self.href = Some(href);
        self
    }
}

/// Render a source detail button inside the generated trailing cell.
///
/// Only the public DTO supplies identity values. This does not register routes or authorize
/// a request. Native Leptos attribute escaping preserves the source URL as ordinary data.
pub fn detail_action(model: &str, identity: &[&str], row: &Value) -> AnyView {
    let Some(actions) = use_context::<RowActions>() else {
        return ().into_any();
    };
    // Source templates omit the entire column for models without identity members.
    if identity.is_empty() {
        return ().into_any();
    }
    let path = identity
        .iter()
        .map(|name| match row.get(*name) {
            Some(Value::String(value)) => value.clone(),
            // JS template interpolation uses one Number family, including integral reals.
            Some(Value::Number(value)) => value
                .as_f64()
                .map(number_text)
                .unwrap_or_else(|| value.to_string()),
            Some(value) => value.to_string(),
            None => "undefined".to_owned(),
        })
        .collect::<Vec<_>>()
        .join("/");
    let route = format!("{}/{model}/detail/{path}", actions.base);
    // Exactly one GET check precedes any href override or icon construction.
    if !actions.can.run(("GET".to_owned(), route)) {
        return ().into_any();
    }
    let href = if let Some(href) = actions.href {
        let key: Map<_, _> = identity
            .iter()
            .filter_map(|name| {
                row.get(*name)
                    .map(|value| ((*name).to_owned(), value.clone()))
            })
            .collect();
        href.run(RowDetail {
            model: model.to_owned(),
            key: Value::Object(key),
            path,
        })
    } else {
        format!("detail/{path}")
    };
    view! { <a class="sp-button sp-row-detail" href=href aria-label="View details"><ChevronRight/></a> }.into_any()
}

/// Preserve JavaScript's scalar identity spelling, as in the existing API page adapter.
fn number_text(value: f64) -> String {
    // Both signs of zero interpolate as the same source route member.
    if value == 0.0 {
        return "0".into();
    }
    // Outside the plain-decimal range, JavaScript retains an explicit positive exponent.
    if value.abs() >= 1e21 || value.abs() < 1e-6 {
        let scientific = format!("{value:e}");
        if let Some((mantissa, exponent)) = scientific.split_once('e')
            && !exponent.starts_with('-')
        {
            return format!("{mantissa}e+{exponent}");
        }
        return scientific;
    }
    value.to_string()
}
