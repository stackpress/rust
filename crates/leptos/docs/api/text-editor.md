# TextEditor and HTML

Import `TextEditor`, `TextEditorOptions`, `TextEditorUpdate` and `HTML` from
`stackpress_leptos::components`. Include `BASE_STYLES` and `CORE_STYLES` for the
existing light/dark tokens and compact Lucide toolbar. No extra feature or editor
dependency is needed; select one renderer mode as usual.

```rust
use stackpress_leptos::leptos::prelude::*;
use stackpress_leptos::components::{HTML, TextEditor};

fn article_editor() -> impl IntoView {
    let contents = RwSignal::new("<p>Article</p>".to_owned());
    view! {
        <TextEditor name="contents" value=contents style=true code=true
            on_change=Callback::new(move |html: String| contents.set(html))/>
        <HTML value=contents/>
    }
}
```

Both components insert HTML verbatim. They add no sanitization, escaping or
application content policy. App authors secure stored HTML and its URLs under
their own trust model. The optional `rich::SafeHtml`/`RichContent` API retains its
separate, explicit sanitizer contract.

## Props and callbacks

| Props | Contract |
| --- | --- |
| `value: Signal<String>` | External HTML, default empty. User edits publish through callbacks. External changes reconcile the hidden input and inactive editing surface, preserving the active caret. |
| `default_value: String` | Initial nonempty fallback before the value effect. Like frui, omitted `value` subsequently reconciles the hidden value to empty even when a default was supplied. |
| `name`, `id`, `class_name`, `disabled`, `required`, `attributes: Vec<AnyAttribute>` | Apply to the hidden form input, like frui's attribute spread. `disabled` omits that form value; it does not disable editing. `required` on a hidden input is not native content validation. |
| `editor_id` | Optional visible textbox identity for an external label. |
| `dir` | Initial direction string; nonempty enables direction controls and literal `rtl` selects RTL layout. |
| `direction_controls: Option<bool>` | Overrides direction-control visibility for the source dictionary's legacy Boolean `dir`. |
| `options: TextEditorOptions` | Complete toolbar carrier. All flags default false. Individual component flags override the corresponding carrier flag, including explicit false. |
| `error` | Reserved, ignored, as in frui. |
| `on_change: Callback<String>` | HTML on input or code toggle; no trimming or sanitization. Closing source mode returns the browser's normalized innerHTML. The hidden input is written synchronously before the callback; on source-mode exit it still holds the authored source during the callback, then reconciles to the normalized value. |
| `on_update: Callback<TextEditorUpdate>` | HTML plus `input`, `codeViewToggle`, or original command name. Commands explicitly publish `input` then their name. Native browser input events also propagate, so one command can cause more than one input notification. |

## Toolbar flags

All original toolbar choices are supported: `history`, `font`, `size`, `format`,
`paragraph`, `blockquote`, `style`, `color`, `highlight`, `text`, `text_style`,
`remove`, `indent`, `align`, `rule`, `list`, `lineheight`, `table`, `link`, `image`,
`image_gallery`, `video`, `audio`, `math`, `fullscreen`, `showblocks`, `code`,
`preview`, `print`, `save` and `template`. Rust's `text_style` and `image_gallery`
correspond to frui's `textStyle` and `imageGallery`.

Commands use the browser's `contentEditable`, `execCommand`, Selection and Range
implementations, including frui's original font-size values. Source mode disables
formatting controls. Fullscreen, source toggle, preview, print and save remain
available. Image/gallery read files through FileReader locally, never a network
upload. Video/audio, tables, links and math placeholders use the original prompts.
Preview and HTML download use the visual editor's HTML; print uses the whole page.

SSR emits the initial form value and HTML; hydration attaches the same component.
CSR initializes the same HTML before user edits. Build these modes separately and
provide identical initial props for hydration. Browser command support remains
the execution host's responsibility.

Source-admin `field.editor` and `view.html` delegate to these components. Source
attribute props keep their original names and positional order. `view.html` has no
additional attribute arguments.

Related: [API index](README.md), [hydration and CSR](../guides/hydrate-and-mount.md),
[optional sanitized content](../guides/optional-helpers.md).
