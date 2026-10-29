//! Actual Leptos route matching and SSR Head/base agreement.
#![cfg(feature = "leptos-ssr")]

use leptos::prelude::*;
use stackpress_leptos::leptos_router::{
    PossibleRouteMatch,
    components::{Route, Router, Routes},
};

use stackpress_framework_view::leptos::{
    render::{LeptosRenderer, initialize_ssr},
    routing::*,
};
use stackpress_framework_view::{
    PageAssets, PageContract, PageId, PageLimits, PageRenderer, PageSnapshot, PublicLocation,
    PublicStatus, ViewError,
};

/// Use one definition to produce the native pattern and Leptos descriptor.
#[test]
fn compiled_path_uses_leptos_segments() {
    let route = PageRoute::new(
        "detail",
        &[
            RouteSegment::Static("ui"),
            RouteSegment::Static("profiles"),
            RouteSegment::Param("id"),
        ],
    )
    .unwrap();
    assert_eq!(route.pattern(), "/ui/profiles/:id");
    let path = CatalogPath::new(route);
    assert!(!path.optional());
    assert!(path.test("/other/abc").is_none());
    let matched = path.test("/ui/profiles/a%2Fb").unwrap();
    assert!(matched.is_complete());
    assert_eq!(matched.params()[0].1, "a%2Fb");
    let mut generated = Vec::new();
    path.generate_path(&mut generated);
    assert_eq!(generated.len(), 3);
    let owner = Owner::new();
    owner.with(|| {
        assert_eq!(use_route_context().base, BasePath::default());
        provide_route_context(BasePath::new("/demo").unwrap());
        assert_eq!(use_route_context().base.as_str(), "/demo");
    });
    owner.cleanup();
}

/// Complete native output owns one canonical/stylesheet/meta root with escaped plain titles.
#[tokio::test]
async fn head_router_and_limits() {
    initialize_ssr().unwrap();
    let route = PageRoute::new(
        "profiles",
        &[RouteSegment::Static("ui"), RouteSegment::Static("profiles")],
    )
    .unwrap();
    let contract = PageContract::new(1, "test").unwrap();
    let assets = PageAssets {
        theme_bootstrap: None,
        build: "test".into(),
        base: BasePath::new("/demo").unwrap(),
        module: PublicLocation::new("/assets/start.js").unwrap(),
        stylesheet: Some(PublicLocation::new("/assets/style.css").unwrap()),
    };
    let renderer = LeptosRenderer::new(contract.clone(), assets.clone(), move |snapshot: &PageSnapshot<String>| {
        let path = CatalogPath::new(route.clone());
        let title = snapshot.data.clone();
        Ok(view! {
            <stackpress_leptos::leptos_meta::Title text=title/>
            <stackpress_leptos::leptos_meta::Meta name="description" content="safe & \"quoted\""/>
            <Router base="/demo"><Routes fallback=|| "MISSING"><Route path=path view=|| view! { <p>"MATCHED"</p> }/></Routes></Router>
        })
    }).unwrap();
    let seed = || PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("profiles".to_owned()).unwrap(),
        location: "/demo/ui/profiles?id=a%26b".into(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: PublicStatus::ok(),
        data: "</title><script>pwned</script>&\"'".to_owned(),
    };
    for _ in 0..2 {
        let html = renderer.render(seed()).await.unwrap().into_html();
        assert!(html.contains("MATCHED"));
        assert!(!html.contains("MISSING"));
        assert!(
            html.contains("&lt;/title&gt;&lt;script&gt;pwned&lt;/script&gt;&amp;&quot;&#39;"),
            "{html}"
        );
        assert!(!html.contains("<script>pwned"));
        assert_eq!(html.matches("rel=\"canonical\"").count(), 1);
        assert_eq!(html.matches("id=\"stackpress-style\"").count(), 1);
        assert_eq!(html.matches("name=\"description\"").count(), 1);
        assert!(html.contains("/demo/assets/start.js"));
        assert!(html.contains("/demo/assets/style.css"));
        assert!(html.contains(r#"href="/demo/ui/profiles?id=a%26b""#));
    }
    let tiny = LeptosRenderer::new(
        PageContract {
            limits: PageLimits {
                html_bytes: 1,
                ..PageLimits::default()
            },
            ..contract.clone()
        },
        assets.clone(),
        |_: &PageSnapshot<String>| Ok(view! { <p>"large"</p> }),
    )
    .unwrap();
    assert!(matches!(
        tiny.render(seed()).await,
        Err(ViewError::HtmlLimit)
    ));
    let mut wrong = seed();
    wrong.locale = "fr".into();
    assert!(matches!(
        renderer.render(wrong).await,
        Err(ViewError::ContractMismatch)
    ));
    let failure = LeptosRenderer::new(
        contract,
        assets,
        |_: &PageSnapshot<String>| -> Result<String, ViewError> {
            Err(ViewError::from_source(std::io::Error::other("private")))
        },
    )
    .unwrap();
    assert!(matches!(
        failure.render(seed()).await,
        Err(ViewError::Failure(_))
    ));
}
