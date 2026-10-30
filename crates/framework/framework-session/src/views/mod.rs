//! Portable Leptos account/auth forms and heads; explicit props replace React hooks.
//!
//! `PageProps` contains only the deliberate source view payload. Native configuration,
//! Engine, signing seeds and transport handles are not reachable from this module.
//!
//! # Direct form use
//! ```
//! use leptos::prelude::*;
//! use stackpress_framework_session::views::{self, PageProps};
//! let owner = Owner::new();
//! owner.with(|| {
//!     let props = PageProps::default();
//!     assert_eq!(views::use_account_base(&props), "/auth");
//!     assert!(views::email_form(&props).to_html().contains("name=\"auth\""));
//! });
//! owner.cleanup();
//! ```
mod account;
mod auth;
mod browser;
mod controls;
mod countries;
mod icons;
mod phone_input;
mod strength;
mod types;
pub use account::account_layout;
pub use controls::{account_back_header, account_csrf_fields, password_strength};
use leptos::prelude::*;
use stackpress_leptos::{
    components::LayoutProvider,
    state::{Notifications, Theme, ThemeState},
};
pub use strength::{PasswordCheck, PasswordEvaluation, evaluate_password, password_checklist};
pub use types::{Config, Menu, Page, PageProps, PasswordRules};

/// Render exactly one source body, independently of document framing/native loading.
pub fn body(kind: Page, props: PageProps) -> AnyView {
    if kind.account() {
        account::body(kind, props)
    } else {
        auth::body(kind, props)
    }
}
/// Source Head metadata with escaped values and explicit additional stylesheet links.
pub fn head(kind: Page, props: &PageProps) -> AnyView {
    let title = if kind == Page::Signup {
        "Signup"
    } else {
        kind.title()
    };
    document_head(
        &phrase(title),
        &phrase(kind.description()),
        kind.account(),
        props,
    )
}
/// Reusable source AccountHead for a custom title/description and explicit public page data.
/// Callers translate their own title and description, matching the source helper.
pub fn account_head(props: &PageProps, title: &str, description: &str) -> AnyView {
    document_head(title, description, true, props)
}
/// Share metadata/assets between the exhaustive page heads and custom account pages.
fn document_head(title: &str, description: &str, account: bool, props: &PageProps) -> AnyView {
    use leptos_meta::{Link, Meta, Style, Title};
    let title = title.to_owned();
    let description = description.to_owned();
    let favicon = props.config.favicon.clone();
    let mime = if favicon.ends_with(".png") {
        "image/png"
    } else if favicon.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "image/x-icon"
    };
    let extra = props
        .config
        .styles
        .iter()
        .map(|href| view! {<Link rel="stylesheet" type_="text/css" href=href.clone()/>})
        .collect_view();
    view! {
        <Title text=title.clone()/>
        <Meta name="description" content=description/>
        {account.then(|| view! {
            <Meta property="og:title" content=title/>
            <Meta property="og:url" content=props.path.clone()/>
        })}
        {(!favicon.is_empty()).then(|| view! {<Link rel="icon" type_=mime href=favicon/>})}
        <Link rel="stylesheet" type_="text/css" href="/styles/global.css"/>{extra}
        <Style id="stackpress-session">{include_str!("styles.css")}</Style>
    }
    .into_any()
}
/// Provide one root-local theme and render a source layout/body. Persistence remains the host's.
pub fn page(kind: Page, props: PageProps) -> AnyView {
    if let Some(language) = props.language.clone() {
        provide_context(stackpress_leptos::language::LanguageContext::new(language));
    }
    let theme = ThemeState::new(if props.theme.as_deref() == Some("dark") {
        Theme::Dark
    } else {
        Theme::Light
    });
    let pending = RwSignal::new(props.theme.is_none());
    // SSR/hydration begin with identical Light/pending attributes. Only the browser
    // effect adopts the host's already frozen one-shot preference after hydration.
    browser::adopt_theme(theme, pending);
    let notifications = Notifications::new();
    let heading = head(kind, &props);
    view!{<LayoutProvider theme=theme notifications=notifications initial_theme_pending=Signal::derive(move ||pending.get())>{heading}{body(kind,props)}</LayoutProvider>}.into_any()
}
/// Source path helper is a missing-only public config read, independent of native services.
pub fn use_account_base(props: &PageProps) -> &str {
    &props.config.base
}

/// Typed source helper aliases; explicit props replace implicit React hook contexts.
pub type AccountPageProps = PageProps;
/// Source signup input/errors plus public config are one explicit component argument.
pub type AuthSignupFormProps = PageProps;

/// Keep all source Body/Head/Page entry points available without duplicating render policy.
macro_rules! source_view {
    ($module:ident,$kind:ident) => {
        #[doc = concat!("Source ", stringify!($module), " Body/Head/Page entry points.")]
        pub mod $module {
            use super::*;
            /// Render this source body with explicit page props.
            pub fn body(props: PageProps) -> leptos::prelude::AnyView {
                super::body(Page::$kind, props)
            }
            /// Render this source document head with explicit public props.
            pub fn head(props: &PageProps) -> leptos::prelude::AnyView {
                super::head(Page::$kind, props)
            }
            /// Render this source page and its root-local presentation state.
            pub fn page(props: PageProps) -> leptos::prelude::AnyView {
                super::page(Page::$kind, props)
            }
        }
    };
}
source_view!(signup, Signup);
source_view!(signin, Signin);
source_view!(username, Username);
source_view!(email, Email);
source_view!(phone, Phone);
source_view!(otp, Otp);
source_view!(link, Link);
source_view!(two_factor, TwoFactor);
source_view!(profile, Account);
source_view!(update, Update);
source_view!(security, Security);
source_view!(password, Password);
source_view!(export, Export);
source_view!(remove, Remove);
source_view!(enroll, Enroll);
source_view!(remove_factor, RemoveFactor);
/// Direct signup form, suitable for application-owned page/layout composition.
pub fn signup_form(props: &PageProps) -> AnyView {
    auth::signup_form(props)
}
/// Direct username form, preserving source required/binding flags.
pub fn username_form(props: &PageProps) -> AnyView {
    auth::password_form(Page::Username, props)
}
/// Direct phone form, preserving the source native required flag.
pub fn phone_form(props: &PageProps) -> AnyView {
    auth::password_form(Page::Phone, props)
}
/// Direct email form, including the password/OTP/magic mode controls.
pub fn email_form(props: &PageProps) -> AnyView {
    auth::email_form(props)
}

/// Translate through the existing root-local language, retaining the source phrase when absent.
pub(crate) fn phrase(source: &str) -> String {
    phrase_with(source, &[])
}
/// Reactive text translation keeps component drafts intact when the root language changes.
pub(crate) fn view_phrase(source: &str) -> AnyView {
    let source = source.to_owned();
    (move || phrase(&source)).into_any()
}

/// Source text interpolation uses the existing checked language context and literal %s slots.
pub(crate) fn phrase_with(source: &str, values: &[String]) -> String {
    stackpress_leptos::language::use_language().map_or_else(
        |_| {
            let mut values = values.iter();
            source
                .split("%s")
                .enumerate()
                .map(|(index, part)| {
                    if index == 0 {
                        part.to_owned()
                    } else {
                        format!("{}{part}", values.next().map(String::as_str).unwrap_or(""))
                    }
                })
                .collect()
        },
        |language| language.text(source, values),
    )
}
