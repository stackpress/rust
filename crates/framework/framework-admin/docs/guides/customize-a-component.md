# Replace a generated component

Select the replacement while constructing the app's catalog. Each replacement provides a canonical definition, revision, typed prop list, fixed props, and an explicit Rust component import. Its compiled component accepts `RoleContext` and returns a Leptos view.

The example's `accent_plugin()` replaces `field.input` before the framework catalog is registered. It fixes the label and placeholder, and maps the component key to the handwritten `crate::plugin::AccentField` component. Aliases and derived filter/span roles select that same mapping.

```sh
cargo run -p generated-admin-tool --locked -- emit generate --plugin
cargo run -p generated-admin-web --locked > /private/tmp/generated-admin-plugin.html
```

The visible label becomes `Plugin field label` and its placeholder becomes `Plugin fixed hint`, including where Idea supplies another placeholder. Fixed props override valid supplied props. Unknown props and incompatible literal types remain errors.

The same plugin selection also replaces `field.tags` with `crate::plugin::AccentTags`, fixes its label to `Plugin tags label`, and retains the typed collection controller through `TextListField`. The taglist alias on the span role resolves to that replacement too. Both hydrate and CSR rebuilds prove the new adapter; disabling removes its wrapper marker and fixed label.

Regenerate without the plugin and rebuild to remove it:

```sh
cargo run -p generated-admin-tool --locked -- emit generate
cargo run -p generated-admin-web --locked > /private/tmp/generated-admin.html
```

Do not patch generated files. The writer checks ownership and drift, and whole-root regeneration removes obsolete wrappers. The finite build/browser proof performs both selections and verifies SSR and hydrated changes.
