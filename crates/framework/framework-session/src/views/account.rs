//! Source account bodies, composed with shared labels, controls and Lucide actions.
use super::{Page, PageProps, controls::*, icons::icon, types::text};
use leptos::prelude::*;

/// Account layout has source Personal/Security navigation and one main landmark.
pub(super) fn body(kind: Page, props: PageProps) -> AnyView {
    let content = match kind {
        Page::Account => profile(&props),
        Page::Update => update(&props),
        Page::Security => security(&props),
        Page::Password => password(&props),
        Page::Export => export(&props),
        Page::Remove => remove(&props),
        Page::Enroll => enroll(&props),
        Page::RemoveFactor => remove_factor(&props),
        _ => ().into_any(),
    };
    layout(kind, &props, content)
}
/// Source AccountLayout accepts arbitrary child content without native/plugin dependencies.
/// Its active navigation is derived from the explicit request path in public props.
pub fn account_layout(props: &PageProps, children: AnyView) -> AnyView {
    let base = if props.config.base.is_empty() {
        "/auth"
    } else {
        &props.config.base
    };
    let kind = if props.path.starts_with(&format!("{base}/account/security")) {
        Page::Security
    } else {
        Page::Account
    };
    layout(kind, props, children)
}
/// Keep the same panel, source base fallback and active navigation for built-in/custom pages.
fn layout(kind: Page, props: &PageProps, content: AnyView) -> AnyView {
    let base = if props.config.base.is_empty() {
        "/auth".to_owned()
    } else {
        props.config.base.clone()
    };
    view!{<div class="session-panel"><nav class="session-nav" aria-label="Account">
        <a href=format!("{base}/account") aria-current=matches!(kind,Page::Account|Page::Update).then_some("page")>{icon("user")}{super::view_phrase("Personal Information")}</a>
        <a href=format!("{base}/account/security") aria-current=(!matches!(kind,Page::Account|Page::Update)).then_some("page")>{icon("lock")}{super::view_phrase("Security Settings")}</a>
    </nav><main class="session-account">{(!props.error.is_empty()).then(||view!{<p role="alert" class="session-error">{props.error.clone()}</p>})}{content}</main></div>}.into_any()
}
/// Source avatar path fallback, with a visible text alternative.
fn avatar(props: &PageProps) -> AnyView {
    let name = text(props.results.get("name"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "account".into());
    let slug = name
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");
    let image = text(props.results.get("image"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("/avatar/{slug}.png"));
    view! {<img class="session-avatar" src=image alt=super::phrase("Profile Picture")/>}.into_any()
}
/// Source profile fields expose the projected token, never the stored secret/hash.
fn profile(props: &PageProps) -> AnyView {
    let fields = [
        ("Name", text(props.results.get("name")).unwrap_or_default()),
        ("Username", props.identifier("username")),
        ("Email Address", props.identifier("email")),
        ("Phone Number", props.identifier("phone")),
    ];
    let rows=fields.into_iter().map(|(label,value)|view!{<tr><th scope="row">{super::view_phrase(label)}</th><td>{if value.is_empty(){"-".into()}else{value}}</td></tr>}).collect_view();
    view!{<header class="session-page-header">{icon("user")}<h1 tabindex="-1">{super::view_phrase("Personal Information")}</h1><a class="session-button" href=format!("{}/account/update",props.config.base)>{icon("pencil")}{super::view_phrase("Update")}</a></header><section class="session-card session-profile">{avatar(props)}<table class="session-details"><tbody>{rows}</tbody></table></section>}.into_any()
}
/// Source update retains profile-bound identifiers and the conditional-password help text.
fn update(props: &PageProps) -> AnyView {
    let image = RwSignal::new(text(props.results.get("image")).unwrap_or_default());
    let name = text(props.results.get("name")).unwrap_or_default();
    let fallback = format!(
        "/avatar/{}.png",
        if name.is_empty() {
            "account".into()
        } else {
            name.to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("-")
        }
    );
    let image_field = input_field("image", "Image", "text", image, false, String::new(), false);
    let identifiers = [
        ("username", "Username", "text"),
        ("email", "Email Address", "text"),
        ("phone", "Phone Number", "tel"),
    ]
    .into_iter()
    .map(|(key, label, kind)| field(props, key, label, kind, props.identifier(key), false))
    .collect_view();
    view!{ {account_back_header("Personal Information",None,Some("user"))}<section class="session-card session-profile"><aside><img class="session-avatar" alt=super::phrase("Profile Picture") src=move ||if image.get().is_empty(){fallback.clone()}else{image.get()}/></aside><form method="post" class="session-form">{account_csrf_fields(props)}{image_field}{field(props,"name","Name","text",name,true)}{identifiers}{field(props,"current","Current Password","password",String::new(),false)}<p class="session-note">{super::view_phrase("Required only when adding a new username, email, or phone sign-in method.")}</p>{submit("Update",false)}</form></section>}.into_any()
}
/// Four source security destinations; permission hints do not gate the operation.
fn security(props: &PageProps) -> AnyView {
    let links=[("password","Change Your Password"),("2fa","Set Up Two-Factor Authentication"),("remove","Delete your account"),("export","Export your data")].into_iter().map(|(path,label)|view!{<a class="session-option" href=format!("{}/account/security/{path}",props.config.base)><span>{super::view_phrase(label)}</span>{icon("chevron-right")}</a>}).collect_view();
    view!{{account_back_header("Security Settings",None,Some("lock"))}<section class="session-card session-security">{links}</section>}.into_any()
}
/// Password form includes source current/new fields and its deliberately unused CSRF input.
fn password(props: &PageProps) -> AnyView {
    view!{{account_back_header("Change Password",Some(format!("{}/account/security",props.config.base)),None)}<form method="post" class="session-card session-form">{account_csrf_fields(props)}{field(props,"current","Current Password","password",String::new(),true)}{field(props,"secret","New Password","password",String::new(),true)}{submit("Update Password",false)}</form>}.into_any()
}
/// Profile export warning is source copy; download is a normal native link.
fn export(props: &PageProps) -> AnyView {
    view!{{account_back_header("Export User Data",Some(format!("{}/account/security",props.config.base)),None)}<section class="session-card session-form"><p>{super::view_phrase("Download a complete copy of your account data including personal information and sign-in identifiers.")}</p><div class="session-notice"><h2>{icon("triangle-alert")}{super::view_phrase("Important Security Notice")}</h2><ul><li>{super::view_phrase("This file contains sensitive personal information including emails, phone numbers, and account details")}</li><li>{super::view_phrase("Do not share this file with unauthorized personnel or upload it to unsecured services")}</li><li>{super::view_phrase("Store the file securely and delete it after you have finished using it")}</li><li>{super::view_phrase("If you need to keep the file long-term, use encrypted storage or password-protected archives")}</li></ul></div><a class="session-button primary" href=format!("{}/account/security/export?download=true",props.config.base)>{icon("download")}{super::view_phrase("Download My Data")}</a></section>}.into_any()
}
/// Source account removal posts confirmed plus the current password.
fn remove(props: &PageProps) -> AnyView {
    view!{{account_back_header("Delete Account",Some(format!("{}/account/security",props.config.base)),None)}<form method="post" class="session-card session-form">{account_csrf_fields(props)}{hidden("confirmed","true")}{field(props,"secret","Password","password",String::new(),true)}{submit("Delete Account",true)}</form>}.into_any()
}
/// Enrollment deliberately renders the source URL/secret/authId, including request-secret flows.
fn enroll(props: &PageProps) -> AnyView {
    let base = &props.config.base;
    let redirect = props.input("redirect_uri", &format!("{base}/account/security/2fa"));
    let secret = text(props.results.get("secret")).unwrap_or_default();
    let url = text(props.results.get("url")).unwrap_or_default();
    let auth = text(props.results.get("authId")).unwrap_or_default();
    let remove = format!(
        "{base}/account/security/2fa/remove?authId={}&redirect_uri={}",
        stackpress_framework_view::encode_uri_component(&auth),
        stackpress_framework_view::encode_uri_component(&redirect)
    );
    view!{{account_back_header("Two-Factor Authentication",Some(format!("{base}/account/security")),None)}<form method="post" class="session-card session-form">{account_csrf_fields(props)}{hidden("redirect_uri",&redirect)}{(!auth.is_empty()).then(||hidden("authId",&auth))}
        {(!url.is_empty()).then(||view!{<label>{super::view_phrase("Scan the QR code using Authenticator App")}<img class="session-qr" src=url alt=super::phrase("QR Code for Two-Factor Authentication")/></label>})}
        {secret_field(secret.clone())}
        // Source detail.tsx submits both the readonly display and hidden secret controls.
        {hidden("secret", &secret)}
        <a href=format!("{base}/account/security/2fa?regenerate")>{super::view_phrase("Regenerate Secret")}</a>{field(props,"code","Enter the code from your Authenticator App","text",String::new(),true)}{submit("Verify",false)}
        {(!auth.is_empty()).then(||view!{<a class="session-button danger" href=remove>{super::view_phrase("Remove 2FA")}</a>})}
    </form>}.into_any()
}
/// Confirmed 2FA removal form retains source authId/redirect inputs and cancel link.
fn remove_factor(props: &PageProps) -> AnyView {
    let base = &props.config.base;
    let redirect = props.input("redirect_uri", &format!("{base}/account/security/2fa"));
    view!{{account_back_header("Remove Two-Factor Authentication",Some(format!("{base}/account/security/2fa")),None)}<section class="session-card session-form"><p>{super::view_phrase("Removing two-factor authentication will make your account less secure.")}</p><form method="post" class="session-form">{account_csrf_fields(props)}{hidden("redirect_uri",&redirect)}{hidden("authId",&props.input("authId",""))}{hidden("confirmed","1")}<div class="session-actions">{submit("Remove 2FA",true)}<a class="session-button" href=format!("{base}/account/security/2fa")>{super::view_phrase("Cancel")}</a></div></form></section>}.into_any()
}
