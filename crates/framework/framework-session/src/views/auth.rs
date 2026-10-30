//! Source auth bodies and forms; only UI draft state lives here, never auth decisions.
use super::{Page, PageProps, controls::*, icons::icon, types::text};
use leptos::prelude::*;

/// Render each source auth body under a compact Frui-style modal layout.
pub(super) fn body(kind: Page, props: PageProps) -> AnyView {
    let contents=match kind{
        Page::Signup=>signup_form(&props),Page::Signin=>chooser(&props),
        Page::Username|Page::Phone=>password_form(kind,&props),Page::Email=>email_body(&props),
        Page::Otp|Page::TwoFactor=>challenge_form(kind,&props),
        Page::Link=>view!{<p role="alert">{if props.error.is_empty(){"Magic link is invalid or expired.".to_owned()}else{props.error.clone()}}</p>{different(&props,true)}}.into_any(),
        _=>().into_any(),
    };
    let glyph = match kind {
        Page::Signup | Page::Username => "user",
        Page::Phone => "phone",
        Page::Otp | Page::TwoFactor => "key-round",
        Page::Link => "link",
        _ => "lock",
    };
    let brand = if kind == Page::TwoFactor {
        ().into_any()
    } else {
        brand(&props)
    };
    let error = props.error.clone();
    view!{<div class="session-blank"><main class="session-auth"><div class="session-brand">{brand}</div><section class="session-modal">
        <header class="session-page-header">{icon(glyph)}<h1 tabindex="-1">{super::view_phrase(kind.title())}</h1>{theme_toggle()}</header>
        {(!error.is_empty()&&kind!=Page::Link).then(||view!{<p class="session-error" role="alert">{error}</p>})}{contents}
        {(kind==Page::Signup).then(||view!{<footer><a href=format!("{}/signin",props.config.base)>{super::view_phrase("Have an Account?")}</a></footer>})}
    </section></main></div>}.into_any()
}
/// Source logo-or-name brand branch; all asset URLs remain explicit public configuration.
fn brand(props: &PageProps) -> AnyView {
    if let Some(logo) = &props.config.logo {
        view!{<img class="session-logo" height="50" src=logo.clone() alt=props.config.brand.clone().unwrap_or_default()/>}.into_any()
    } else {
        props
            .config
            .brand
            .clone()
            .map(|name| view! {<h2>{name}</h2>})
            .into_any()
    }
}
/// Source method chooser splits footer entries and appends redirect only to main choices.
fn chooser(props: &PageProps) -> AnyView {
    let redirect =
        stackpress_framework_view::encode_uri_component(&props.input("redirect_uri", "/"));
    let options=props.config.menu.iter().filter(|entry|entry.type_.as_deref()!=Some("footer")).map(|entry|{
        let glyph=match entry.icon.as_deref(){Some("envelope"|"mail")=>"mail",Some("phone")=>"phone",Some("user")=>"user",Some("key")=>"key-round",Some("wand-magic-sparkles")=>"link",_=>"lock"};
        view!{<a class="session-option" href=format!("{}?redirect_uri={redirect}",entry.path) target=entry.target.clone()>{icon(glyph)}<span>{super::view_phrase(&entry.name)}</span>{icon("chevron-right")}</a>}
    }).collect_view();
    let footer=props.config.menu.iter().filter(|entry|entry.type_.as_deref()==Some("footer")).map(|entry|view!{<a href=entry.path.clone() target=entry.target.clone()>{super::view_phrase(&entry.name)}</a>}).collect_view();
    view! {<div class="session-options">{options}</div><footer>{footer}</footer>}.into_any()
}
/// Signup's source spread binding and strength meter share the exact password draft.
pub(super) fn signup_form(props: &PageProps) -> AnyView {
    let secret = RwSignal::new(props.bound("secret"));
    let fields = [
        ("name", "Name*", "text", true),
        ("email", "Email Address", "text", false),
        ("phone", "Phone Number", "tel", false),
        ("username", "Username", "text", false),
    ]
    .into_iter()
    .map(|(name, label, kind, required)| {
        field(props, name, label, kind, props.bound(name), required)
    })
    .collect_view();
    let password = input_field(
        "secret",
        "Password*",
        "password",
        secret,
        true,
        text(props.errors.get("secret")).unwrap_or_default(),
        false,
    );
    view!{<form method="post" class="session-form">{account_csrf_fields(props)}{fields}{password}{props.config.password.clone().map(|rules|password_strength(secret,rules))}{submit("Sign Up",false)}</form>}.into_any()
}
/// Username and phone retain separate source native required flags.
pub(super) fn password_form(kind: Page, props: &PageProps) -> AnyView {
    let (name, label, input_kind, required) = if kind == Page::Phone {
        ("phone", "Phone Number *", "tel", true)
    } else {
        ("username", "Username *", "text", false)
    };
    view!{<form method="POST" class="session-form">{account_csrf_fields(props)}{field(props,name,label,input_kind,String::new(),required)}{field(props,"secret","Password *","password",String::new(),false)}{submit("Submit",false)}{different(props,false)}</form>}.into_any()
}
/// Source email success branch is only code 200 + POST + magic, not merely selected mode.
fn email_body(props: &PageProps) -> AnyView {
    if props.code == 200 && props.method == "POST" && props.input("auth", "pass") == "magic" {
        view!{<div class="session-message"><p>{super::view_phrase("A magic link has been sent to your email address.")}</p><p>{super::view_phrase("Please check your inbox and click the link to sign in.")}</p>{different(props,false)}</div>}.into_any()
    } else {
        email_form(props)
    }
}
/// Inline email-mode controls are native buttons with named/pressed state and keyboard support.
pub(super) fn email_form(props: &PageProps) -> AnyView {
    let mode = RwSignal::new("pass".to_owned());
    let modes=[("pass","key-round","Sign in with password"),("otp","mail","Send one-time pin code"),("magic","link","Send magic link")].into_iter().map(|(value,glyph,label)|view!{
        <button type="button" class="session-icon-button" aria-label=super::phrase(label) aria-pressed=move ||(mode.get()==value).to_string() on:click=move |_|mode.set(value.into())>{icon(glyph)}</button>
    }).collect_view();
    let password = field(
        props,
        "secret",
        "Password *",
        "password",
        String::new(),
        false,
    );
    view!{<form method="POST" class="session-form">{account_csrf_fields(props)}<input type="hidden" name="auth" value=move ||mode.get()/>
        {field(props,"email","Email Address *","email",String::new(),false)}
        <div class="session-mode-row"><span>{move ||super::phrase(match mode.get().as_str(){"otp"=>"(send one-time pin code)","magic"=>"(send magic link)",_=>"Password *"})}</span><div role="group" aria-label="Sign in method">{modes}</div></div>
        <div hidden=move ||mode.get()!="pass"><fieldset disabled=move ||mode.get()!="pass">{password}</fieldset></div>{submit("Submit",false)}{different(props,false)}
    </form>}.into_any()
}
/// OTP and authenticator forms use source six-character text inputs, not a new numeric policy.
pub(super) fn challenge_form(kind: Page, props: &PageProps) -> AnyView {
    let email = text(props.results.get("token"))
        .filter(|email| !email.is_empty())
        .unwrap_or_else(|| props.input("email", ""));
    let instructions = if kind == Page::Otp {
        super::phrase_with("Please enter the one-time password sent to %s.", &[email])
    } else {
        super::phrase("Enter the 6-digit code from your authenticator app.")
    };
    view!{<form method="POST" class="session-form">{account_csrf_fields(props)}<p>{instructions}</p>{field(props,"code","Code","text",String::new(),true)}{submit("Submit",false)}{different(props,true)}</form>}.into_any()
}
/// Preserve source redirect retention only on the challenge/fallback variants.
fn different(props: &PageProps, redirect: bool) -> AnyView {
    let suffix = if redirect {
        format!(
            "?redirect_uri={}",
            stackpress_framework_view::encode_uri_component(&props.input("redirect_uri", "/"))
        )
    } else {
        String::new()
    };
    view!{<a class="session-form-footer" href=format!("{}/signin{suffix}",props.config.base)>{super::view_phrase("Choose a different sign-in method")}</a>}.into_any()
}
