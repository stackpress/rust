//! Scoped state behavior, bounds and disposal, independent of DOM adapters.

use super::*;

/// Root-local state cannot leak locale/theme/notification changes to another root.
#[test]
fn providers_are_explicit_isolated_and_disposed() {
    let first = Owner::new();
    let second = Owner::new();
    let a = first.with(|| {
        assert!(use_theme().is_err());
        assert!(use_notifications().is_err());
        let theme = ThemeState::new(Theme::Light);
        let notifications = Notifications::default();
        provide_context(theme);
        provide_context(notifications);
        assert_eq!(use_theme().unwrap().get(), Theme::Light);
        assert!(use_notifications().unwrap().entries().is_empty());
        (theme, notifications)
    });
    let b = second.with(|| (ThemeState::new(Theme::Light), Notifications::new()));
    a.0.toggle();
    a.1.notify(NoticeKind::Warning, "Check input".into())
        .unwrap();
    assert_eq!(a.0.get().as_str(), "dark");
    assert_eq!(b.0.get().as_str(), "light");
    assert!(b.1.entries().is_empty());
    a.0.set(Theme::Dark);
    a.0.toggle();
    assert_eq!(a.0.get(), Theme::Light);
    first.cleanup();
    assert!(a.0.0.try_get_untracked().is_none());
    assert!(a.1.entries.try_get_untracked().is_none());
    second.cleanup();
}

/// Duplicate notices coalesce and capacity/byte/key limits reject atomically.
#[test]
fn notification_limits_duplicates_and_dismissal() {
    let owner = Owner::new();
    owner.with(|| {
        let notices = Notifications::new();
        assert!(notices.notify(NoticeKind::Info, String::new()).is_err());
        assert!(notices.notify(NoticeKind::Info, "é".repeat(2049)).is_err());
        let id = notices.notify(NoticeKind::Info, "é".repeat(2048)).unwrap();
        assert_eq!(
            notices.notify(NoticeKind::Info, "é".repeat(2048)).unwrap(),
            id
        );
        for index in 1..20 {
            notices
                .notify(NoticeKind::Warning, format!("Notice {index}"))
                .unwrap();
        }
        assert_eq!(notices.entries().len(), 20);
        assert!(notices.notify(NoticeKind::Error, "Full".into()).is_err());
        assert!(notices.dismiss(id));
        assert!(!notices.dismiss(id));
        assert!(!notices.dismiss(999));
        assert_eq!(notices.entries()[0].message, "Notice 1");
        let next = notices
            .notify(NoticeKind::Error, "Replacement".into())
            .unwrap();
        assert!(next > id);
        notices.clear();
        notices.serial.set(u64::MAX);
        assert!(
            notices
                .notify(NoticeKind::Info, "Exhausted".into())
                .is_err()
        );
        assert!(notices.entries().is_empty());
        assert_eq!(NoticeKind::Info.role(), "status");
        assert_eq!(NoticeKind::Warning.role(), "status");
        assert_eq!(NoticeKind::Error.role(), "alert");
    });
    owner.cleanup();
}

/// Public budgets are checked, root-local and enforced without global mutation.
#[test]
fn configurable_notification_budgets() {
    let owner = Owner::new();
    owner.with(|| {
        for options in [
            NotificationOptions {
                max_notices: 0,
                max_message_bytes: 4,
            },
            NotificationOptions {
                max_notices: 21,
                max_message_bytes: 4,
            },
            NotificationOptions {
                max_notices: 1,
                max_message_bytes: 0,
            },
            NotificationOptions {
                max_notices: 1,
                max_message_bytes: 4097,
            },
        ] {
            assert!(Notifications::with_options(options).is_err());
        }
        let notices = Notifications::with_options(NotificationOptions {
            max_notices: 1,
            max_message_bytes: 4,
        })
        .unwrap();
        assert!(notices.notify(NoticeKind::Info, "12345".into()).is_err());
        notices.notify(NoticeKind::Info, "1234".into()).unwrap();
        assert!(notices.notify(NoticeKind::Info, "next".into()).is_err());
        assert!(
            Notifications::new()
                .notify(NoticeKind::Info, "12345".into())
                .is_ok()
        );
    });
    owner.cleanup();
}
