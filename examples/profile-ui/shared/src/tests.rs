//! Real controller state transitions independent of browser event plumbing.
use super::*;
use stackpress_example_profile_ui_contracts::{
    Profile, REVISION, SearchFailure, SearchFailureKind, TransportFailure,
};
use std::{cell::RefCell, rc::Rc};

/// Seed one meaningful public result without duplicating domain search logic.
fn seed() -> SearchPage {
    SearchPage {
        detail: None,
        input: SearchInput::new("abc".into()).unwrap(),
        reply: reply("Ada Lovelace"),
    }
}

/// An ambiguous acquired reply cannot replace useful prior rows or enter the keyed view.
#[test]
fn controller_rejects_ambiguous_rows_without_publishing() {
    let owner = Owner::new();
    owner.with(|| {
        let controller = SearchController::new(seed());
        let transport = UnsyncCallback::new(move |(_, done): (SearchInput, SearchCompletion)| {
            done(Ok(SearchReply {
                revision: REVISION,
                total: 2,
                rows: vec![
                    Profile {
                        id: "abc".into(),
                        name: "First duplicate".into(),
                    },
                    Profile {
                        id: "abc".into(),
                        name: "Second duplicate".into(),
                    },
                ],
            }));
        });
        controller.submit(transport, None);
        assert_eq!(
            controller.rows.get_untracked(),
            vec![("abc".into(), "Ada Lovelace".into())]
        );
        assert_eq!(controller.total.get_untracked(), 1);
        assert_eq!(controller.published.get_untracked(), 0);
        assert_eq!(
            controller.failure_kind.get_untracked(),
            Some(SearchFailureKind::Protocol)
        );
        assert!(!controller.pending.get_untracked());
    });
    owner.cleanup();
}

/// The real profile dictionary handles 0/1/2 consistently for text and node phrases.
#[cfg(feature = "ssr")]
#[test]
fn counted_profile_phrases() {
    for locale in ["en", "fr"] {
        for count in [0_u64, 1, 2] {
            let owner = Owner::new();
            owner.with(|| {
                stackpress_leptos::leptos_meta::provide_meta_context();
                let data = SearchPage { input: SearchInput::new("abc".into()).unwrap(), detail: None,
                    reply: SearchReply { revision: REVISION, total: count,
                        rows: (0..count).map(|index| Profile { id: format!("abc{index}"), name: "Public fixture".into() }).collect() } };
                let html = view! { <ProfileSearch seed=data initial_language=profile_language(locale).unwrap()/> }.to_html();
                let singular = count == 1 || (locale == "fr" && count == 0);
                let noun = match (locale, singular) { ("fr", true) => "profil", ("fr", false) => "profils", (_, true) => "profile", _ => "profiles" };
                assert!(html.contains(&format!("{count} {noun}</p>")), "{locale}/{count}: {html}");
                let suffix = match (locale, singular) { ("fr", true) => " profil public affiché.", ("fr", false) => " profils publics affichés.", (_, true) => " public profile.", _ => " public profiles." };
                assert!(html.contains(&format!("<strong>{count}</strong>{suffix}")), "{locale}/{count}: {html}");
            });
            owner.cleanup();
        }
    }
}

/// Navigation restores useful public state and independently preserves the return prefix.
#[test]
fn catalog_navigation_and_retained_history() {
    use super::navigation::{Navigation, PageNavigation};
    let owner = Owner::new();
    owner.with(|| {
        assert!(PageNavigation::new(Default::default(), None, "").is_err());
        let controller = SearchController::new(seed());
        let navigation = PageNavigation::new(Default::default(), None, "abc").unwrap();
        let writes = Rc::new(RefCell::new(Vec::new()));
        let output = writes.clone();
        let adapter = Navigation {
            request: RwSignal::new(None),
            write: UnsyncCallback::new(move |value| output.borrow_mut().push(value)),
        };
        let completions = Rc::new(RefCell::new(Vec::<SearchCompletion>::new()));
        let pending = completions.clone();
        let transport = UnsyncCallback::new(move |(_, done)| pending.borrow_mut().push(done));
        navigation.go(
            "/foreign".into(),
            true,
            controller,
            transport,
            None,
            adapter,
        );
        assert!(writes.borrow().is_empty());
        navigation.go(
            "/ui/profiles/abc?return=ab".into(),
            true,
            controller,
            transport,
            None,
            adapter,
        );
        assert_eq!(
            navigation.detail.get_untracked().unwrap().search_prefix,
            "ab"
        );
        completions.borrow()[0](Ok(reply("Detail")));
        navigation.completed(controller, adapter);
        assert_eq!(
            writes.borrow().last().unwrap().0,
            "/ui/profiles/abc?return=ab"
        );
        navigation.go(
            "/ui/profiles?id=abc".into(),
            false,
            controller,
            transport,
            None,
            adapter,
        );
        assert!(navigation.detail.get_untracked().is_none());
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
        assert_eq!(
            completions.borrow().len(),
            1,
            "history restores without a read"
        );
        navigation.go(
            "/ui/profiles/abc?return=ab".into(),
            false,
            controller,
            transport,
            None,
            adapter,
        );
        assert_eq!(controller.rows.get_untracked()[0].1, "Detail");
        navigation.go(
            "/ui/profiles?id=ab".into(),
            true,
            controller,
            transport,
            None,
            adapter,
        );
        assert!(navigation.detail.get_untracked().is_none());
        assert_eq!(controller.id.get_untracked(), "ab");
        // An uncached return performs the selected prefix read, never infers it from ID.
        assert_eq!(completions.borrow().len(), 2);
        navigation.go(
            "/ui/profiles?id=abc".into(),
            false,
            controller,
            transport,
            None,
            adapter,
        );
        completions.borrow()[1](Ok(reply("Late return")));
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
    });
    owner.cleanup();
}

/// Make a typed public result with no duplicated search implementation.
fn reply(name: &str) -> SearchReply {
    SearchReply {
        revision: REVISION,
        rows: vec![Profile {
            id: "abc".into(),
            name: name.into(),
        }],
        total: 1,
    }
}

/// Invalid input, late completion, empty result, errors and disposal share one controller.
#[test]
fn controller_preserves_newest_result_and_disposal() {
    let owner = Owner::new();
    let completions = Rc::new(RefCell::new(Vec::<SearchCompletion>::new()));
    let (controller, transport) = owner.with(|| {
        let controller = SearchController::new(SearchPage {
            detail: None,
            input: SearchInput::new("abc".into()).unwrap(),
            reply: reply("Seed"),
        });
        let callbacks = Rc::clone(&completions);
        let transport = UnsyncCallback::new(move |(_, done)| callbacks.borrow_mut().push(done));
        (controller, transport)
    });
    assert_eq!(controller.total.get_untracked(), 1);
    assert!(completions.borrow().is_empty(), "no initial request");
    controller.id.set(String::new());
    controller.submit(transport, None);
    assert!(!controller.field_error.get_untracked().is_empty());
    assert!(completions.borrow().is_empty());
    controller.id.set("abc".into());
    controller.submit(transport, None);
    controller.submit(transport, None);
    completions.borrow()[1](Ok(reply("Newest")));
    completions.borrow()[0](Ok(reply("Late")));
    assert_eq!(controller.rows.get_untracked()[0].1, "Newest");
    assert!(!controller.pending.get_untracked());
    controller.submit(transport, None);
    completions.borrow()[2](Ok(SearchReply {
        revision: REVISION,
        rows: Vec::new(),
        total: 0,
    }));
    assert_eq!(controller.total.get_untracked(), 0);
    assert!(controller.rows.get_untracked().is_empty());
    controller.submit(transport, None);
    completions.borrow()[3](Err(SearchFailure::Transport(TransportFailure::Unavailable)));
    assert!(
        controller
            .error
            .get_untracked()
            .contains("Search unavailable")
    );
    assert_eq!(
        controller.failure_kind.get_untracked(),
        Some(SearchFailureKind::Transport)
    );
    controller.submit(transport, None);
    let late = std::sync::Arc::clone(&completions.borrow()[4]);
    owner.cleanup();
    late(Ok(reply("Disposed")));
    assert!(controller.total.try_get_untracked().is_none());
}

/// Invalid submission and cancellation revoke old work before adapter callbacks.
#[test]
fn invalidation_cancellation_and_retry_preserve_useful_rows() {
    let owner = Owner::new();
    owner.with(|| {
        let controller = SearchController::new(seed());
        let completions = Rc::new(RefCell::new(Vec::<SearchCompletion>::new()));
        let callbacks = Rc::clone(&completions);
        let transport = UnsyncCallback::new(move |(_, done)| callbacks.borrow_mut().push(done));
        let cancelled = Rc::new(std::cell::Cell::new(0));
        let count = Rc::clone(&cancelled);
        let cancel = UnsyncCallback::new(move |()| count.set(count.get() + 1));
        controller.submit(transport, Some(cancel));
        assert!(controller.pending.get_untracked());
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
        controller.id.set(String::new());
        controller.submit(transport, Some(cancel));
        completions.borrow()[0](Ok(reply("Must not replace validation")));
        assert!(!controller.field_error.get_untracked().is_empty());
        assert!(!controller.pending.get_untracked());
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
        assert_eq!(completions.borrow().len(), 1);
        controller.id.set("abc".into());
        controller.submit(transport, Some(cancel));
        controller.cancel(Some(cancel));
        completions.borrow()[1](Ok(reply("Cancelled result")));
        assert!(controller.error.get_untracked().contains("cancelled"));
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
        controller.submit(transport, Some(cancel));
        completions.borrow()[2](Err(SearchFailure::Transport(TransportFailure::Unavailable)));
        assert_eq!(controller.notices.entries().len(), 1);
        controller.submit(transport, Some(cancel));
        assert!(controller.notices.entries().is_empty());
        completions.borrow()[3](Ok(reply("Retried")));
        completions.borrow()[3](Ok(reply("Duplicate completion")));
        assert_eq!(controller.rows.get_untracked()[0].1, "Retried");
        assert!(controller.error.get_untracked().is_empty());
        assert_eq!(cancelled.get(), 6);
    });
    owner.cleanup();
}

/// Invalid replies and exhausted generations cannot fabricate success or reuse rights.
#[test]
fn reply_validation_and_generation_exhaustion() {
    let owner = Owner::new();
    owner.with(|| {
        let controller = SearchController::new(seed());
        let completions = Rc::new(RefCell::new(Vec::<SearchCompletion>::new()));
        let callbacks = Rc::clone(&completions);
        let transport = UnsyncCallback::new(move |(_, done)| callbacks.borrow_mut().push(done));
        controller.submit(transport, None);
        let mut invalid = reply("Bad wire");
        invalid.revision = 999;
        completions.borrow()[0](Ok(invalid));
        assert!(controller.error.get_untracked().contains("Refresh"));
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
        controller.generation.set(Some(u64::MAX - 1));
        controller.submit(transport, None);
        controller.submit(transport, None);
        assert!(controller.generation.get_untracked().is_none());
        completions.borrow()[1](Ok(reply("Exhausted old call")));
        assert_eq!(completions.borrow().len(), 2);
        assert!(controller.error.get_untracked().contains("limit reached"));
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
    });
    owner.cleanup();
}

/// Shared markup uses the seed once; only explicit packaged bootstrap loads locally.
#[cfg(feature = "ssr")]
#[test]
fn shared_views_and_explicit_bootstrap() {
    let owner = Owner::new();
    owner.with(|| {
        stackpress_leptos::leptos_meta::provide_meta_context();
        let calls = Rc::new(std::cell::Cell::new(0));
        let count = Rc::clone(&calls);
        let transport = UnsyncCallback::new(move |(_, done): (SearchInput, SearchCompletion)| {
            count.set(count.get() + 1);
            done(Ok(reply("Loaded locally")));
        });
        let html = view! { <ProfileSearch seed=seed() transport/> }.to_html();
        assert_eq!(calls.get(), 0, "hydration seed is not refetched");
        assert!(html.contains("Ada Lovelace"));
        assert!(html.contains("Profile ID prefix"));
        assert!(html.contains("profile-id-help profile-id-error"));
        assert!(html.contains("<strong>1</strong>"));
        let html = view! { <ProfileSearch seed=seed() transport initial_search=true/> }.to_html();
        assert_eq!(calls.get(), 1);
        assert!(html.contains("Loaded locally"));
        let mut empty = seed();
        empty.reply.rows.clear();
        empty.reply.total = 0;
        let html =
            view! { <ProfileSearch seed=empty initial_language=profile_language("fr").unwrap()/> }
                .to_html();
        assert!(html.contains("Préfixe de l’identifiant"));
        assert!(html.contains("Aucun profil ne correspond"));
        assert!(html.contains("<strong>0</strong>"));
        let mut invalid = seed();
        invalid.input.id.clear();
        assert!(
            view! { <ProfileSearch seed=invalid/> }
                .to_html()
                .contains("Page data invalid")
        );
    });
    owner.cleanup();
}

/// Bootstrap success must not erase the independently reported storage recovery.
#[cfg(feature = "ssr")]
#[test]
fn recovery_notice_survives_initial_search() {
    let owner = Owner::new();
    owner.with(|| {
        stackpress_leptos::leptos_meta::provide_meta_context();
        let transport = UnsyncCallback::new(move |(_, done): (SearchInput, SearchCompletion)| {
            done(Ok(reply("Loaded locally")));
        });
        let html = view! { <ProfileSearch seed=seed() transport initial_search=true preference_recovered=true/> }.to_html();
        assert!(html.contains("Loaded locally"));
        assert!(html.contains("Saved preferences could not be loaded. Defaults are in use."));
    });
    owner.cleanup();
}

/// Detail is exact, with safe missing/hostile text and a query-preserving return URL.
#[cfg(feature = "ssr")]
#[test]
fn exact_detail_and_not_found() {
    let owner = Owner::new();
    owner.with(|| {
        let html = view! { <ProfileDetail id="abc" reply=reply("<script>Ada</script>") return_href="/ui/profiles?id=abc"/> }.to_html();
        assert!(html.contains("&lt;script&gt;Ada&lt;/script&gt;"));
        assert!(html.contains("Return to search"));
        assert!(html.contains("/ui/profiles?id=abc"));
        let html = view! { <ProfileDetail id="ab" reply=reply("Ada") return_href="/ui/profiles?id=abc"/> }.to_html();
        assert!(html.contains("Profile not found"));
        assert!(!html.contains("<h2>Ada"));
    });
    owner.cleanup();
}

/// Concurrent server roots keep locale maps independent.
#[cfg(feature = "ssr")]
#[test]
fn concurrent_locales_remain_isolated() {
    let renders = ["en", "fr"].map(|locale| std::thread::spawn(move || {
        let owner = Owner::new();
        let html = owner.with(|| {
            stackpress_leptos::leptos_meta::provide_meta_context();
            view! { <ProfileSearch seed=seed() initial_language=profile_language(locale).unwrap()/> }.to_html()
        });
        owner.cleanup();
        html
    }));
    let [english, french] = renders.map(|render| render.join().unwrap());
    assert!(english.contains("Profile ID prefix"));
    assert!(!english.contains("Préfixe de l’identifiant"));
    assert!(french.contains("Préfixe de l’identifiant"));
    assert!(!french.contains("Profile ID prefix"));
    assert_eq!(
        profile_language("es").unwrap().text("Profiles", &[]),
        "Profiles"
    );
    assert!(profile_language("../fr").is_err());
}

/// Form action uses the same explicit mount as the operation and asset bootstrap.
#[cfg(feature = "ssr")]
#[test]
fn mounted_form_action() {
    let owner = Owner::new();
    owner.with(|| {
        stackpress_leptos::leptos_meta::provide_meta_context();
        stackpress_framework_view::leptos::routing::provide_route_context(
            stackpress_framework_view::leptos::routing::BasePath::new("/office").unwrap(),
        );
        let html = view! { <ProfileSearch seed=seed()/> }.to_html();
        assert!(html.contains("action=\"/office/ui/profiles\""));
    });
    owner.cleanup();
}

/// Completed host rejections retain their safe message and a category distinct from transport.
#[test]
fn controller_retains_host_rejection_identity() {
    let owner = Owner::new();
    owner.with(|| {
        let controller = SearchController::new(seed());
        let transport = UnsyncCallback::new(move |(_, done): (SearchInput, SearchCompletion)| {
            done(Err(SearchFailure::Host(
                stackpress_example_profile_ui_contracts::HostError {
                    code: 422,
                    status: "Unprocessable Entity".into(),
                    error: "Choose another prefix.".into(),
                    errors: Default::default(),
                    total: 0,
                },
            )));
        });
        controller.submit(transport, None);
        assert_eq!(
            controller.failure_kind.get_untracked(),
            Some(SearchFailureKind::Host)
        );
        assert_eq!(
            controller.error.get_untracked(),
            "Search rejected: Choose another prefix."
        );
        assert_eq!(controller.rows.get_untracked()[0].1, "Ada Lovelace");
        assert!(!controller.pending.get_untracked());
    });
    owner.cleanup();
}
