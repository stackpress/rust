//! Saved Light/Dark are SSR-complete; unset alone admits the one-shot external asset.
#![cfg(feature = "ssr")]
use leptos::prelude::*;
use stackpress_framework_ui::{
    components::LayoutProvider,
    render::{LeptosRenderer, initialize_ssr},
    state::{Notifications, Theme, ThemeState},
};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageRenderer, PageSnapshot, PublicLocation,
    PublicStatus, theme,
};

/// Assert real native output, including readable unset fallback and asset ordering.
#[tokio::test]
async fn saved_choices_and_unset_head() {
    initialize_ssr().unwrap();
    for choice in [Some("light"), Some("dark"), None] {
        let mut contract = PageContract::new(1, "theme-test").unwrap();
        contract.theme = choice.map(str::to_owned);
        let renderer = LeptosRenderer::new(contract, PageAssets {
            build: "theme-test".into(), base: BasePath::new("/demo").unwrap(),
            module: PublicLocation::new("/assets/start.js").unwrap(), stylesheet: None,
            theme_bootstrap: Some(theme::bootstrap_location().unwrap()),
        }, move |_: &PageSnapshot<()>| {
            let theme = ThemeState::new(if choice == Some("dark") { Theme::Dark } else { Theme::Light });
            Ok(view! { <LayoutProvider theme notifications=Notifications::new()><p>"Readable without JavaScript"</p></LayoutProvider> })
        }).unwrap();
        let html = renderer
            .render(PageSnapshot {
                revision: 1,
                build: "theme-test".into(),
                page: PageId::try_from("theme".to_owned()).unwrap(),
                location: "/demo/theme".into(),
                locale: "en".into(),
                theme: choice.map(str::to_owned),
                status: PublicStatus::ok(),
                data: (),
            })
            .await
            .unwrap()
            .into_html();
        assert!(html.contains("Readable without JavaScript"));
        assert!(html.contains(&format!("data-theme=\"{}\"", choice.unwrap_or("light"))));
        assert_eq!(html.contains("data-theme-unset"), choice.is_none());
        assert_eq!(
            html.contains(&format!(
                "<script src=\"/demo/assets/{}\"></script>",
                theme::BOOTSTRAP_FILE
            )),
            choice.is_none()
        );
        if choice.is_none() {
            assert!(
                html.find(theme::BOOTSTRAP_FILE).unwrap()
                    < html.find("id=\"stackpress-ui\"").unwrap()
            );
        }
    }
    let owner = Owner::new();
    owner.with(|| {
        let theme = ThemeState::new(Theme::Light);
        theme.toggle();
        assert_eq!(theme.get(), Theme::Dark);
        theme.toggle();
        assert_eq!(theme.get(), Theme::Light);
    });
    owner.cleanup();
}
