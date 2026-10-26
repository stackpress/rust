//! Native SSR markup contracts, including safe text, semantics and layout slots.

use super::*;
use crate::state::{NoticeKind, Notifications, Theme, ThemeState, use_notifications, use_theme};
use leptos::prelude::*;

/// Navigation rejects executable/normalized schemes and accepts explicit safe links.
#[test]
fn safe_navigation_targets() {
    for target in [
        "/ui/profiles?id=a%20b",
        "../profiles",
        "#results",
        "?id=abc",
        "https://example.com/x",
        "http://localhost/path",
    ] {
        assert!(safe_href(target), "{target}");
    }
    for target in [
        "",
        "javascript:alert(1)",
        "JaVaScRiPt:x",
        "data:text/html,x",
        "//evil.test",
        "\\evil.test",
        "https:///evil.test",
        "https://u:p@evil.test",
        " javascript:x",
        "java\nscript:x",
        "/has space",
    ] {
        assert!(!safe_href(target), "{target}");
    }
}

/// Native controls retain label/error associations, safe text and explicit semantics.
#[test]
fn controls_render_native_accessible_markup() {
    let owner = Owner::new();
    owner.with(|| {
        let value = RwSignal::new("<b>unsafe</b>".to_owned());
        let html = view! {
            <FieldControl for_id="prefix" label="Prefix" help="Use an ID" error="Invalid prefix">
                <Input id="prefix" name="id" value described_by="prefix-help prefix-error" invalid=true/>
            </FieldControl>
            <Button button_type=ButtonType::Submit id="search".to_owned() name="action".to_owned() value="search".to_owned()>"Search"</Button>
            <Button disabled=true>"Disabled"</Button>
            <Button href="/ui/profiles".to_owned()>"Link"</Button>
            <Button href="javascript:x".to_owned()>"Unsafe"</Button>
            <Link href="javascript:x" label="<img>"/>
            <Alert message="<script>failure</script>"/>
            <Loader busy=true label="Searching…"/>
            <LoaderContainer busy=true><p>"Kept results"</p></LoaderContainer>
        }.to_html();
        assert!(html.contains("for=\"prefix\""));
        assert!(html.contains("id=\"prefix-error\" role=\"alert\""));
        assert!(html.contains("aria-describedby=\"prefix-help prefix-error\""));
        assert!(html.contains("aria-invalid=\"true\""));
        assert!(html.contains("type=\"submit\""));
        assert!(html.contains("disabled"));
        assert!(html.contains("href=\"/ui/profiles\""));
        assert!(!html.contains("javascript:"));
        assert!(html.contains("&lt;script&gt;failure&lt;/script&gt;"));
        assert!(html.contains("aria-busy=\"true\""));
        assert!(html.contains("Kept results"));
        assert!(html.contains("Searching…"));
        assert_eq!(ButtonType::Button.as_str(), "button");
        assert_eq!(ButtonType::Reset.as_str(), "reset");
    });
    owner.cleanup();
}

/// Keyed list emptiness, explicit layout composition and providers render together.
#[test]
fn list_layout_and_notifications() {
    let owner = Owner::new();
    owner.with(|| {
        leptos_meta::provide_meta_context();
        let theme = ThemeState::new(Theme::Dark);
        let notifications = Notifications::new();
        notifications.notify(NoticeKind::Info, "<b>Done</b>".into()).unwrap();
        notifications.notify(NoticeKind::Warning, "Check".into()).unwrap();
        let items = RwSignal::new(vec![ListItem { id: "1".into(), label: "<Ada>".into(), href: Some("/ui/profiles/1".into()) }, ListItem { id: "2".into(), label: "Grace".into(), href: None }]);
        let html = view! { <LayoutProvider theme notifications><LayoutBlank>
            <LayoutHead><h1>"Profiles"</h1><LayoutMenu label="Profiles"><Link href="/" label="Home"/></LayoutMenu></LayoutHead>
            <LayoutPanel><LayoutLeft label="Search help"><p>"Help"</p></LayoutLeft><LayoutMain>
                <List items empty="No profiles"/>
                <List items=Signal::derive(Vec::new) empty="Empty list"/>
                <NotifierContainer notifications/>
                {move || { assert!(use_theme().is_ok()); assert!(use_notifications().is_ok()); "" }}
            </LayoutMain><LayoutRight label="About"><p>"Public directory"</p></LayoutRight></LayoutPanel>
        </LayoutBlank></LayoutProvider> }.to_html();
        assert!(html.contains("data-theme=\"dark\""));
        assert!(html.contains("<main id=\"main-content\""));
        assert!(html.contains("aria-label=\"Search help\""));
        assert!(html.contains("aria-label=\"About\""));
        assert!(html.contains("data-item-id=\"1\""));
        assert!(html.contains("&lt;Ada&gt;"));
        assert!(html.contains("Empty list"));
        assert!(html.contains("&lt;b&gt;Done&lt;/b&gt;"));
        assert!(html.contains("Dismiss notification"));
    });
    owner.cleanup();
}
