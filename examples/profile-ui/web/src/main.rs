//! Finite fixture bridge for browser automation; opens no socket or listener.

use stackpress_server::http::{HttpAdapter, HttpOutput};

/// Render a page or execute a typed POST through the actual HttpAdapter, then exit.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let base = stackpress_framework_view::BasePath::new(
        std::env::var("PROFILE_UI_BASE").unwrap_or_default(),
    )?;
    // Only this executable reads the operator's environment; libraries take layers.
    let config = std::env::var_os("PROFILE_UI_CONFIG").map(std::path::PathBuf::from);
    let layers = stackpress_example_profile_ui_web::backend::configuration(config.as_deref())?;
    let app =
        stackpress_example_profile_ui_web::application_with_config(base.clone(), layers).await?;
    if arguments.first().map(String::as_str) == Some("hostile") {
        use stackpress_example_profile_ui_contracts::{
            Profile, REVISION, SearchInput, SearchPage, SearchReply, snapshot,
        };
        let seed = snapshot(SearchPage {
            detail: None,
            input: SearchInput::new("abc".into())?,
            reply: SearchReply {
                revision: REVISION,
                rows: vec![Profile {
                    id: "abc123".into(),
                    name: "</script><script>globalThis.pwned=1</script>&\"\u{2028}\u{2029}".into(),
                }],
                total: 1,
            },
        })?;
        print!(
            "{}",
            stackpress_example_profile_ui_web::render_snapshot(seed).await?
        );
        return Ok(());
    }
    let adapter = HttpAdapter::new(Default::default())?;
    let request = if arguments.first().map(String::as_str) == Some("search") {
        http::Request::builder()
            .method("POST")
            .uri(format!("{}/ui-api/v1/profile-search", base.as_str()))
            .header("content-type", "application/json")
            .body(
                arguments
                    .get(1)
                    .ok_or("missing search JSON")?
                    .as_bytes()
                    .to_vec(),
            )?
    } else {
        let default_path = format!("{}/ui/profiles?id=abc", base.as_str());
        http::Request::builder()
            .uri(
                arguments
                    .first()
                    .map(String::as_str)
                    .unwrap_or(&default_path),
            )
            .body(Vec::new())?
    };
    match adapter.handle(&app, request).await?.output {
        HttpOutput::Response(response) => {
            use std::io::Write;
            if matches!(
                std::env::var("PROFILE_UI_DISPLAY").as_deref(),
                Ok("fr-dark" | "fr-unset")
            ) && arguments.first().map(String::as_str) != Some("search")
            {
                // Trusted finite proof configuration: re-render the actual loaded public seed.
                let html = std::str::from_utf8(response.body())?;
                let text = html
                    .split("id=\"stackpress-page\">")
                    .nth(1)
                    .ok_or("seed missing")?
                    .split("</script>")
                    .next()
                    .ok_or("seed missing")?;
                let mut seed = stackpress_framework_view::PageSnapshot::<
                    stackpress_example_profile_ui_contracts::SearchPage,
                >::decode(
                    text,
                    stackpress_example_profile_ui_contracts::REVISION,
                    stackpress_example_profile_ui_contracts::BUILD,
                )?;
                seed.locale = "fr".into();
                seed.theme = (std::env::var("PROFILE_UI_DISPLAY").as_deref() == Ok("fr-dark"))
                    .then(|| "dark".into());
                let html = stackpress_example_profile_ui_web::render_snapshot_with_base(seed, base)
                    .await?;
                std::io::stdout().write_all(html.as_bytes())?;
            } else {
                std::io::stdout().write_all(response.body())?;
            }
        }
        _ => return Err("output unavailable".into()),
    }
    Ok(())
}
