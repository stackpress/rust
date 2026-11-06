//! Portable dispatcher over the app and existing session/API page contracts.
use leptos::prelude::*;
use serde_json::Value;
pub use stackpress_framework_admin::source::roles::{Choice, RelationLoader, RelationRequest};
use stackpress_framework_api::views::{
    ApiOauthPage, Consent,
    management::{Management, ManagementPage},
};
use stackpress_framework_session::views::{Page as SessionPage, PageProps};
use stackpress_framework_view::{PageSnapshot, ViewError};

/// Install metadata within the current root's reactive Owner.
pub fn meta_context() {
    leptos_meta::provide_meta_context();
}

/// Existing public wire contracts, selected by their document seed element.
#[derive(Clone)]
pub enum Root {
    /// App snapshot checked against this app's revision/build.
    Blog(crate::Page),
    /// Existing source session snapshot, with its own build contract.
    Session(SessionPage, PageProps),
    /// Generated application model admin.
    Management(Management),
    /// Existing API authorization page.
    Consent(Consent),
}
impl Root {
    /// Decode an existing seed without serializing native services or guessing its payload.
    /// # Errors
    /// Retains snapshot decoder and JSON failures.
    pub fn decode(id: &str, text: &str) -> Result<Self, ViewError> {
        match id {
            "stackpress-page" => {
                let header: Value = serde_json::from_str(text).map_err(ViewError::from_source)?;
                if header["build"] == "blog-v1" {
                    Ok(Self::Blog(
                        PageSnapshot::<crate::Page>::decode(text, 1, "blog-v1")?.data,
                    ))
                } else {
                    let (kind, props) =
                        PageSnapshot::<(SessionPage, PageProps)>::decode(text, 1, "session-v1")?
                            .data;
                    Ok(Self::Session(kind, props))
                }
            }
            "api-management-seed" => serde_json::from_str(text)
                .map(Self::Management)
                .map_err(ViewError::from_source),
            "api-consent-seed" => serde_json::from_str(text)
                .map(Self::Consent)
                .map_err(ViewError::from_source),
            _ => Err(ViewError::from_source(std::io::Error::other(
                "page seed missing",
            ))),
        }
    }
    /// Render the same components as native SSR with explicit browser transports.
    /// # Errors
    /// Retains source article formatting errors before mounting a partial root.
    pub fn view(
        self,
        submit: Callback<Value>,
        search: Callback<Value>,
    ) -> Result<AnyView, ViewError> {
        Ok(match self {
            Self::Blog(page) => (
                crate::views::head(&page),
                crate::views::page(page).map_err(ViewError::from_source)?,
            )
                .into_any(),
            Self::Session(kind, props) => stackpress_framework_session::views::page(kind, props),
            Self::Management(page) => {
                view! {<ManagementPage page ui=crate::ui::pages::ui() submit search/>}.into_any()
            }
            Self::Consent(page) => view! {<ApiOauthPage page/>}.into_any(),
        })
    }
}

/// Relation labels follow the blog's name/title templates, retaining joined rows.
pub fn choices(value: Value) -> Vec<Choice> {
    value
        .get("results")
        .unwrap_or(&value)
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            Some(Choice {
                id: row.get("id")?.as_str()?.into(),
                label: row
                    .get("name")
                    .or_else(|| row.get("title"))?
                    .as_str()?
                    .into(),
                row: row.clone(),
            })
        })
        .collect()
}
