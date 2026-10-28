//! One real old/new consumer pins cookies, files and HTTP/Tauri public envelopes.
use serde_json::{Value, json};
use stackpress_framework_language::LocaleRegistry;
#[cfg(not(feature = "after"))]
use stackpress_framework_preferences::{
    CookieScope, FlashCookie, FlashNotice, SettingsAction, ThemePreference,
};
use stackpress_framework_view::BasePath;
#[cfg(feature = "after")]
use stackpress_framework_view::{
    adapters::web::CookieScope,
    flash::{FlashCookie, FlashNotice},
    preferences::SettingsAction,
};

/// Own only this run's private temporary settings directory.
struct Fixture(std::path::PathBuf);
impl Drop for Fixture {
    /// Remove fixture files even if a contract assertion unwinds.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Compare public serialization from actual web and native command implementations.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::PathBuf::from(std::env::var("ISSUE15_GOLDEN_DIRECTORY")?)
        .join(format!("golden-{}", std::process::id()));
    std::fs::create_dir(&root)?;
    let fixture = Fixture(root);
    let registry = || {
        LocaleRegistry::new(
            vec!["en".into(), "fr".into()],
            "en".into(),
            Default::default(),
        )
        .unwrap()
    };
    let mut cookies = Vec::new();
    for origin in ["http://preferences.invalid", "https://preferences.invalid"] {
        for mount in ["", "/demo/v1"] {
            let scope = CookieScope::new(origin.into(), BasePath::new(mount)?)?;
            #[cfg(not(feature = "after"))]
            let action = SettingsAction::new(registry(), scope.clone(), ThemePreference::System);
            #[cfg(feature = "after")]
            let action = SettingsAction::new(registry(), scope.clone());
            let body = json!({"locale":"fr","theme":"dark","location":format!("{mount}/en/preferences?q=a%2Fb&return=%252F")}).to_string();
            let reply = action.apply("POST", Some(origin), &body)?;
            let flash = FlashCookie::new(scope);
            let issued =
                flash.issue(&FlashNotice::new("golden".into(), "Saved <public>".into())?)?;
            let consumed = flash.consume(issued.split(';').next().unwrap())?;
            assert_eq!(
                consumed.notice.as_ref().unwrap().message(),
                "Saved <public>"
            );
            cookies.push(json!({"origin":origin,"mount":mount,"settings":reply.cookies,"reply":serde_json::to_string(&reply)?,"flash":issued,"clear":consumed.clear_cookie}));
        }
    }
    let mut app = stackpress_server::ServerBuilder::new();
    stackpress_example_ui_preferences_web::configure(
        &mut app,
        BasePath::new("/demo/v1")?,
        false,
        false,
        "https://preferences.invalid".into(),
    )?;
    let app = app.build().await?;
    let body = r#"{"locale":"fr","theme":"dark","location":"/demo/v1/en/preferences?q=a%2Fb&return=%252F"}"#;
    let request = http::Request::builder()
        .method("POST")
        .uri("/demo/v1/ui-preferences")
        .header("host", "preferences.invalid")
        .header("origin", "https://preferences.invalid")
        .header("content-type", "application/json")
        .body(body.as_bytes().to_vec())?;
    let outcome = stackpress_server::http::HttpAdapter::new(Default::default())?
        .handle(&app, request)
        .await?;
    let stackpress_server::http::HttpOutput::Response(response) = outcome.output else {
        panic!("missing HTTP response")
    };
    let http = json!({"status":response.status().as_u16(),"body":std::str::from_utf8(response.body())?,"cookies":response.headers().get_all("set-cookie").iter().map(|value|value.to_str().unwrap()).collect::<Vec<_>>()});
    assert_eq!(http["status"], 200);
    let host = native_preferences::PreferencesHost::new(fixture.0.join("native"));
    let update = r#"{"revision":1,"preferences":{"locale":"fr","theme":"dark"}}"#;
    let native_reply = host.update(update)?;
    let file_name = "ui-preferences-v1.json";
    let stored = std::fs::read_to_string(fixture.0.join("native").join(file_name))?;
    assert_eq!(stored, update);
    drop(host);
    let reopened = native_preferences::PreferencesHost::new(fixture.0.join("native")).load()?;
    assert_eq!(native_reply, reopened);
    // Legacy theme input is deliberately the one G39 value-domain difference.
    let legacy = r#"{"revision":1,"preferences":{"locale":"fr","theme":"system"}}"#;
    std::fs::write(fixture.0.join("native").join(file_name), legacy)?;
    let load = native_preferences::PreferencesHost::new(fixture.0.join("native")).load()?;
    let load_again = native_preferences::PreferencesHost::new(fixture.0.join("native")).load()?;
    assert_eq!(load, load_again);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("native").join(file_name))?,
        legacy
    );
    let loaded: Value = serde_json::from_str(&load)?;
    assert_eq!(loaded["preferences"]["locale"], "fr");
    assert_eq!(loaded["recovered"], false);
    let scope = CookieScope::new("https://preferences.invalid".into(), BasePath::default())?;
    #[cfg(not(feature = "after"))]
    let action = SettingsAction::new(registry(), scope, ThemePreference::System);
    #[cfg(feature = "after")]
    let action = SettingsAction::new(registry(), scope);
    let cookie = action.select(
        "/preferences",
        "stackpress-locale=fr; stackpress-theme=system",
    )?;
    assert_eq!(cookie.locale, "fr");
    let report = json!({"stable":{"cookies":cookies,"http":http,"file_name":file_name,"file":stored,"tauri_update":native_reply,"tauri_load":reopened},"legacy":{"cookie":serde_json::to_value(cookie)?,"file_reply":loaded,"file_unchanged":true}});
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
