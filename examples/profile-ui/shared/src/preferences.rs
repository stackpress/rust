//! Optional persistence boundary; omitted hooks preserve the existing immediate UI.
use leptos::prelude::*;
use stackpress_framework_ui::{
    language::LanguageContext,
    state::{NoticeKind, Notifications, Theme, ThemeState},
};
use std::sync::Arc;

/// A specific user intent, never a native command/event or arbitrary URL.
#[derive(Clone, Debug)]
pub enum PreferenceChange {
    /// Change only locale, preserving the saved theme mode.
    Locale(String),
    /// Change only the explicitly selected light/dark theme.
    Theme(Theme),
}
/// Full acknowledged settings to publish together in this root.
#[derive(Clone, Debug)]
pub struct DisplayPreferences {
    /// Checked by the existing example phrase registry before publication.
    pub locale: String,
    /// Effective explicit color mode, resolved before mounting for native system mode.
    pub theme: Theme,
}
/// Completion must be marshalled back to the originating UI execution context.
pub type PreferenceCompletion = Arc<dyn Fn(Result<DisplayPreferences, ()>) + Send + Sync>;
/// Optional local adapter; performs persistence before invoking its completion.
pub type PreferenceTransport = UnsyncCallback<(PreferenceChange, PreferenceCompletion)>;

/// One root writer. Draft/search/navigation ownership is deliberately independent.
#[derive(Clone, Copy)]
pub(crate) struct PreferenceController {
    /// At most one admitted write may publish into this root.
    pending: RwSignal<bool>,
    /// Completion identity prevents an older adapter callback from taking ownership.
    generation: RwSignal<u64>,
    /// Root-local dictionary, published only after a checked acknowledgement.
    language: LanguageContext,
    /// Root-local effective theme; the transport retains any saved system mode.
    theme: ThemeState,
    /// Accessible failure notices share the existing root's bounded queue.
    notices: Notifications,
    /// Optional persistence adapter; absence keeps the legacy synchronous path.
    transport: Option<PreferenceTransport>,
}
impl PreferenceController {
    /// Bind existing root state without changing the default DOM or behavior.
    pub(crate) fn new(
        language: LanguageContext,
        theme: ThemeState,
        notices: Notifications,
        transport: Option<PreferenceTransport>,
    ) -> Self {
        Self {
            pending: RwSignal::new(false),
            generation: RwSignal::new(0),
            language,
            theme,
            notices,
            transport,
        }
    }
    /// Ignore repeated clicks while a single admitted save owns this root.
    pub(crate) fn change(self, change: PreferenceChange) {
        if self.pending.get_untracked() {
            return;
        }
        let Some(transport) = self.transport else {
            // Legacy web/desktop behavior remains synchronous and host independent.
            match change {
                PreferenceChange::Locale(locale) => {
                    if let Ok(language) = crate::profile_language(&locale) {
                        self.language.change(language);
                    }
                }
                PreferenceChange::Theme(theme) => self.theme.set(theme),
            }
            return;
        };
        // Each admitted save owns its completion; an older callback cannot publish
        // into a newer save even when an adapter incorrectly calls it twice.
        let Some(generation) = self.generation.get_untracked().checked_add(1) else {
            return;
        };
        self.generation.set(generation);
        self.pending.set(true);
        let complete: PreferenceCompletion = Arc::new(move |result| {
            // A disposed root cannot receive a late acknowledgement.
            if self.pending.try_get_untracked() != Some(true)
                || self.generation.try_get_untracked() != Some(generation)
            {
                return;
            }
            let admitted = result.and_then(|settings| {
                crate::profile_language(&settings.locale)
                    .map(|language| (language, settings.theme))
                    .map_err(|_| ())
            });
            batch(move || {
                self.pending.set(false);
                match admitted {
                    Ok((language, theme)) => {
                        self.language.change(language);
                        self.theme.set(theme);
                    }
                    Err(()) => {
                        let _ = self.notices.notify(
                            NoticeKind::Error,
                            "Preferences could not be saved. Your draft is preserved.".into(),
                        );
                    }
                }
            });
        });
        transport.run((change, complete));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    /// A delayed/failed write cannot publish settings, and duplicates cannot race it.
    #[test]
    fn publish_only_after_a_successful_acknowledgement() {
        let owner = Owner::new();
        owner.with(|| {
            let language = LanguageContext::new(crate::profile_language("en").unwrap());
            let theme = ThemeState::new(Theme::Light);
            let notices = Notifications::new();
            let completion = Arc::new(Mutex::new(None::<PreferenceCompletion>));
            let captured = completion.clone();
            let adapter = PreferenceTransport::new(move |(_, done)| {
                *captured.lock().unwrap() = Some(done);
            });
            let controller = PreferenceController::new(language, theme, notices, Some(adapter));
            controller.change(PreferenceChange::Locale("fr".into()));
            assert_eq!(language.get().locale(), "en");
            assert_eq!(theme.get(), Theme::Light);
            let old = completion.lock().unwrap().take().unwrap();
            controller.change(PreferenceChange::Theme(Theme::Dark));
            assert!(completion.lock().unwrap().is_none());
            old(Err(()));
            assert_eq!(language.get().locale(), "en");
            assert_eq!(notices.entries().len(), 1);
            controller.change(PreferenceChange::Locale("fr".into()));
            let done = completion.lock().unwrap().take().unwrap();
            old(Ok(DisplayPreferences {
                locale: "en".into(),
                theme: Theme::Dark,
            }));
            assert_eq!(theme.get(), Theme::Light);
            done(Ok(DisplayPreferences {
                locale: "fr".into(),
                theme: Theme::Dark,
            }));
            assert_eq!(language.get().locale(), "fr");
            assert_eq!(theme.get(), Theme::Dark);
            // Duplicate completions have no pending write to publish into.
            done(Ok(DisplayPreferences {
                locale: "en".into(),
                theme: Theme::Light,
            }));
            assert_eq!(language.get().locale(), "fr");
        });
        owner.cleanup();
    }
    /// Omitted adapters retain today's immediate locale/theme behavior.
    #[test]
    fn omitted_hook_is_immediate() {
        let owner = Owner::new();
        owner.with(|| {
            let language = LanguageContext::new(crate::profile_language("en").unwrap());
            let theme = ThemeState::new(Theme::Light);
            let controller = PreferenceController::new(language, theme, Notifications::new(), None);
            controller.change(PreferenceChange::Locale("fr".into()));
            controller.change(PreferenceChange::Theme(Theme::Dark));
            assert_eq!(language.get().locale(), "fr");
            assert_eq!(theme.get(), Theme::Dark);
        });
        owner.cleanup();
    }

    /// Invalid acknowledgements cannot publish, and disposal revokes pending callbacks.
    #[test]
    fn invalid_acknowledgement_and_disposed_root_are_inert() {
        let owner = Owner::new();
        let completion = Arc::new(Mutex::new(None::<PreferenceCompletion>));
        let (controller, language, theme) = owner.with(|| {
            let captured = completion.clone();
            let language = LanguageContext::new(crate::profile_language("en").unwrap());
            let theme = ThemeState::new(Theme::Light);
            let adapter = PreferenceTransport::new(move |(_, done)| {
                *captured.lock().unwrap() = Some(done);
            });
            (
                PreferenceController::new(language, theme, Notifications::new(), Some(adapter)),
                language,
                theme,
            )
        });
        controller.change(PreferenceChange::Theme(Theme::Dark));
        completion.lock().unwrap().take().unwrap()(Ok(DisplayPreferences {
            locale: "../fr".into(),
            theme: Theme::Dark,
        }));
        assert_eq!(language.get().locale(), "en");
        assert_eq!(theme.get(), Theme::Light);
        controller.change(PreferenceChange::Theme(Theme::Dark));
        let late = completion.lock().unwrap().take().unwrap();
        owner.cleanup();
        late(Ok(DisplayPreferences {
            locale: "fr".into(),
            theme: Theme::Dark,
        }));
        assert!(controller.pending.try_get_untracked().is_none());
    }
}
