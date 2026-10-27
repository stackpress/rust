//! System is a host-seeded public token; old light/dark render paths keep their styles.
#![cfg(feature = "ssr")]
use leptos::prelude::*;
use stackpress_framework_ui::{
    components::{LayoutProvider, SystemThemeStyles},
    render::{LeptosRenderer, initialize_ssr},
    state::{Notifications, Theme, ThemeState},
};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageRenderer, PageSnapshot, PublicLocation,
    PublicStatus,
};

/// Real native rendering preserves the selected token and includes only opted-in system CSS.
#[tokio::test]
async fn explicit_system_styles_and_legacy_themes() {
    initialize_ssr().unwrap();
    for theme in [Theme::Light, Theme::Dark, Theme::System] {
        let mut contract = PageContract::new(1, "theme-test").unwrap();
        contract.theme = theme.as_str().into();
        let renderer=LeptosRenderer::new(contract,PageAssets{build:"theme-test".into(),base:BasePath::default(),module:PublicLocation::new("/assets/start.js").unwrap(),stylesheet:None},move |_:&PageSnapshot<()>|{
            let state=ThemeState::new(theme);let notices=Notifications::new();
            Ok(view!{<LayoutProvider theme=state notifications=notices>{(theme==Theme::System).then(||view!{<SystemThemeStyles/>})}<p>"Same component"</p></LayoutProvider>})
        }).unwrap();
        let html = renderer
            .render(PageSnapshot {
                revision: 1,
                build: "theme-test".into(),
                page: PageId::try_from("theme".to_owned()).unwrap(),
                location: "/theme".into(),
                locale: "en".into(),
                theme: theme.as_str().into(),
                status: PublicStatus::ok(),
                data: (),
            })
            .await
            .unwrap()
            .into_html();
        assert!(html.contains(&format!("data-theme=\"{}\"", theme.as_str())));
        assert_eq!(html.matches("id=\"stackpress-ui\"").count(), 1);
        assert_eq!(
            html.matches("id=\"stackpress-system-theme\"").count(),
            usize::from(theme == Theme::System)
        );
        assert!(html.contains(include_str!("../src/components/styles.css")));
    }
    let owner = Owner::new();
    owner.with(|| {
        let theme = ThemeState::new(Theme::System);
        assert_eq!(theme.get(), Theme::System);
        theme.toggle();
        assert_eq!(theme.get(), Theme::Dark);
        theme.toggle();
        assert_eq!(theme.get(), Theme::Light);
    });
    owner.cleanup();
}
