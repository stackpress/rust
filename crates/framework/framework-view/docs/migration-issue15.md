# Presentation API migration (issue 15)

`stackpress-framework-preferences` is removed. Add `stackpress-framework-view`
with the capabilities your application uses. The default view dependency graph stays
unchanged; importing a library or enabling a feature registers nothing.

| Previous import | Canonical import / feature |
| --- | --- |
| Preferences, SettingsAction, DTOs, PreferenceState, SaveState, ThemePreference | `stackpress_framework_view::preferences` / `preferences` |
| FlashNotice, FlashCookie, FlashRoot | `stackpress_framework_view::flash` / `flash` |
| ActionId, AllowedActions, HostAccess, HostPolicy | `stackpress_framework_view::access` / always available |
| CookieScope | `stackpress_framework_view::adapters::web` / `cookies` (also re-exported by preferences and flash) |
| PreferenceFile, FileError, FILE_NAME | `stackpress_framework_view::adapters::native` / `native-file`, Unix native only |
| Language routing, actions, selection, constants | `stackpress_framework_view::preferences::locale` (also re-exported by preferences) |

`framework-language` retains dependency-free `LocaleRegistry` and `PreferenceError`.
Replace `registry.select(&base, location, cookie)`, `registry.location(&base, location,
locale)` and `registry.prefixes()` with `LocaleRouting::new(&registry, &base)` followed
by `select(location, cookie)`, `location(location, locale)` or `prefixes()`.
There is no reverse language re-export. Registry enumeration is available through
`locales()`, `aliases()` and `default_locale()`.

`SettingsAction::new(registry, scope)` loses its fallback-theme argument.
`Preferences.theme` and `SettingsUpdate.theme` are `Option<ThemePreference>`;
`PageContract.theme` and `PageSnapshot.theme` are `Option<String>` checked as
`light`/`dark`. Missing/null means no explicit choice. Both theme enums contain only
Light and Dark. Saved `system` cookies/files read as unset at those boundaries;
live DTOs reject that token. Existing Light/Dark wires, cookie identities/scope and
native filename/revision remain unchanged. Loading never rewrites a legacy file.

`PageAssets` adds `theme_bootstrap: Option<PublicLocation>` (missing decodes as None).
For unset pages, opt into `theme::bootstrap_location()` and serve the exact
`theme::BOOTSTRAP_SOURCE` bytes under `theme::BOOTSTRAP_FILE`, with immutable caching.
The renderer emits a classic same-origin script before themed content. A strict CSP
can use `theme::bootstrap_tag(base, path, nonce)` in a custom shell; nonce input is
validated. Explicit saved choices need no bootstrap. Without its execution unset
SSR remains readable Light. The asset samples the OS once, never stores a choice,
and never tracks later OS changes. Only a user toggle saves.

`framework-ui` remains the Leptos layer. Its LayoutProvider accepts an optional
`initial_theme_pending: Signal<bool>` for the frozen ancestor CSS handoff; supply the
same initial Light tree for SSR/hydration, then adopt the document choice in one batch.
The profile and preferences examples demonstrate the integration. SystemThemeStyles,
SYSTEM_THEME_STYLES and system.css are removed.

Facade users replace the removed `framework-preferences` feature/import with
`framework-view-preferences`, `framework-view-flash`, `framework-view-native-file`,
and `stackpress::framework_view` paths as applicable. Host features remain explicit.
The agreed function entry-point convention is `view::{preferences,flash}::register`;
host registration implementation is the following milestone, coordinated with issue 14.
