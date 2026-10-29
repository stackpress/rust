//! Pinned extraction parity output; runs without any listener.
use futures::StreamExt;
use leptos::prelude::*;

use stackpress_framework_view::leptos::{
    async_render::AsyncLeptosRenderer,
    render::{LeptosRenderer, initialize_ssr},
    replay::ResourceReplay,
    stream::{StreamLeptosRenderer, StreamLimits},
};
use stackpress_framework_view::{
    AsyncPageRenderer, BasePath, PageAssets, PageContract, PageRenderer, PageSnapshot,
    PublicLocation, ViewError,
};

/// Render deliberately hostile plain text and eager metadata identically in every mode.
fn content(page: &PageSnapshot<String>) -> Result<AnyView, ViewError> {
    Ok(
        view! { <leptos_meta::Title text="title </title>&雪"/><p>{page.data.clone()}</p> }
            .into_any(),
    )
}

/// Pin complete public bytes, settings, error precedence and route metadata before moving code.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_ssr()?;
    let mut outputs = Vec::new();
    for theme in [Some("light"), Some("dark"), None] {
        let mut contract = PageContract::new(1, "parity")?;
        contract.theme = theme.map(str::to_owned);
        let assets = PageAssets {
            build: "parity".into(),
            base: BasePath::new("/demo/v1")?,
            module: PublicLocation::new("/start.js")?,
            stylesheet: Some(PublicLocation::new("/style.css")?),
            theme_bootstrap: Some(stackpress_framework_view::theme::bootstrap_location()?),
        };
        let snapshot = PageSnapshot {
            revision: 1,
            build: "parity".into(),
            page: "sample".to_owned().try_into()?,
            location: "/demo/v1/sample?q=%E9%9B%AA&x=0".into(),
            locale: contract.locale.clone(),
            theme: contract.theme.clone(),
            status: Default::default(),
            data: "</script>&雪\u{2028}\u{2029}".to_owned(),
        };
        let seed = snapshot.script_json()?;
        let make_snapshot = || PageSnapshot::<String>::decode(&seed, 1, "parity");
        let sync = LeptosRenderer::new(contract.clone(), assets.clone(), content)?
            .render(make_snapshot()?)
            .await?
            .into_html();
        let asynchronous = AsyncLeptosRenderer::new(contract.clone(), assets.clone(), content)?
            .render_async(make_snapshot()?)
            .await?
            .into_html();
        let mut streamed = StreamLeptosRenderer::new(
            contract.clone(),
            assets.clone(),
            StreamLimits::default(),
            content,
        )?
        .render(make_snapshot()?)
        .await?;
        let status = streamed.status;
        let mut bytes = Vec::new();
        while let Some(chunk) = streamed.body.next().await {
            bytes.extend(chunk?);
        }
        let mut invalid = make_snapshot()?;
        invalid.build = "wrong".into();
        let error = LeptosRenderer::new(contract.clone(), assets.clone(), content)?
            .render(invalid)
            .await
            .err()
            .unwrap();
        outputs.push(serde_json::json!({"theme":theme,"seed":snapshot.script_json()?,"assets":assets.script_json()?,"sync":sync,"async":asynchronous,"stream":String::from_utf8(bytes)?,"status":status,"error":format!("{error:?}: {error}")}));
    }
    let replay = ResourceReplay::new(
        vec![(0, "</template></script>&雪\u{2028}\u{2029}".into())],
        1024,
    )?
    .script_json()?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"pages":outputs,"replay":replay,"mount":BasePath::new("/demo/v1")?.join("/p?q=%2f")?.as_str()})
        )?
    );
    Ok(())
}
