# Source roles

`components::source` supplies the shared Leptos behavior used by the source-admin
catalog. These components accept the original source values without the separate
Typed UI constructors' row, byte, URL, mailbox or color admission policies.
Application code owns content, destinations, validation, upload handlers and IO.
Include `components::source::SOURCE_STYLES` alongside the existing base/core styles.

The `source` feature enables `SourceMarkdown`, `SourceMarkdownEditor` and
`source_markdown` through the already locked pulldown-cmark dependency. The admin
crate selects it with `source`; use admin `source-ssr` for native date/relative-time
formatting, and `source,hydrate` or `source,csr` for separate browser artifacts.
No source component starts a listener, fetches a grammar, uploads a file or installs
a JS library. Stored Markdown HTML remains unsanitized, like the original content
path; sanitization is an application decision under G43.

## Public surfaces

- Code: `SourceCode`, `SourceCodeEditor`, `source_code_tab`.
- Markdown: `SourceMarkdown`, `SourceMarkdownEditor`, `source_markdown` (`source`).
- Masks: `SourceMaskInput`, `SourceMaskToken`, `source_mask`, `source_mask_value`,
  `source_mask_delete`.
- Choices: `SourceRadio`, `SourceSelect`, `SourceChoice`, `SourceSuggestInput`,
  `SourceCountrySelect`, `SourceCurrencySelect`.
- Files: `SourceFileInput`, `SourceFile`, `SourceUpload`, `SourceUploadCompletion`,
  `SourceFileState`. Set `multiple`/`image` to select the four original file roles.
- Collections: `SourceRows`, `SourceFieldset`, `SourceTextList`,
  `SourceFieldsetDisplay`.
- Displays: `SourceColor`, `SourceEmailLink`, `SourceRating`, `SourceSpread`,
  `SourceList`, `SourceTable`, `SourceCountry`, `SourceImageFilm`,
  `SourceImageCarousel`, `SourceCarouselState`.
- Shared conversion: `SourceAppearance`, `source_overflow`, `source_color_hex`,
  `source_relative`, `source_small`, `source_formula`.
- Lossless data: `CountryData`, `CurrencyData`, `COUNTRIES`, `CURRENCIES`, `COLORS`
  and their `*_SHA256` source identifiers.

Every component lives in the caller's Leptos Owner. Editable controls publish
through callbacks/signals; displays render escaped text except the explicitly raw
Markdown/HTML paths. `SourceUploadCompletion` can outlive the mounted control;
late completions update only live root-local signals.

The existing `Input` accepts source class/style props and an optional
`omit_empty_associations` switch. Source non-text adapters use that switch to
retain their original bare native markup. Actual descriptions/errors are still
associated; the shared control's default Typed markup remains unchanged.

## G55 differences from Frui

Owner decision G55 deliberately chooses Rust-native implementations. The following
are the complete rich-role differences; other source arguments and fixed values
remain in the source catalog.

| Source role | Rust behavior and original anchor |
| --- | --- |
| `field.code` | Native monospaced textarea and UTF-16 Tab/Shift+Tab editing replace CodeMirror's view, history, extension/setup system and language loading. `extensions`, `setup`, `numbers`, `language` remain metadata but add no CodeMirror behavior. Tab replaces a selection; Shift+Tab removes one leading tab from the caret's line. Original: `frui/src/form/CodeEditor.tsx`, `getEditorOptions`, `getLanguageExtension`, `useCodeEditor`. |
| `field.json` | The same native textarea; source fixed `language=json`, `setup=basic` remain metadata. Invalid JSON is retained. The original CodeEditor does not parse, validate or pretty-print JSON. |
| `view.code` | Escaped monospaced text replaces Shiki syntax coloring, its themes/highlighter and default stylesheet. Source language/line-number labels and authored class/style props remain available. Original: `frui/src/view/Code.tsx`, `Code`. |
| `view.json` | The same plain display with fixed `language=json`; no validation or pretty-printing is invented. Structured values use the existing source display conversion. Original: `attributes.ts` `view.json`, `frui/src/view/Code.tsx`. |
| `field.markdown` | Monospaced native textarea with Edit/Preview controls and iframe preview. pulldown-cmark replaces markdown-to-jsx and React's static-markup renderer. HTML serialization, extension syntax and wrapper choices therefore follow pulldown-cmark. Original: `frui/src/form/MarkdownEditor.tsx`, `MarkdownEditor`. |
| `view.markdown` | pulldown-cmark with tables, footnotes, strikethrough and task lists replaces markdown-to-jsx. Embedded HTML is retained without an added sanitizer. Original: `frui/src/view/Markdown.tsx`. |
| `field.mask` | Rust common-mask grammar and keyboard adapter replace Inputmask's engine. Supported tokens: `9` ASCII digit; `a` Latin/Cyrillic letter; `*` either; backslash escapes; optional `[...]`; finite `{n}`/`{n,m}` repetitions; ordinary punctuation including `+` and literal parentheses. Unsupported: alternation, regex/alias modes, group quantifiers, open-ended quantifiers, custom definitions and Inputmask plugin hooks. An unsupported standalone pattern remains ordinary text. Original: `frui/src/form/MaskInput.tsx`, `useMaskInput`, and Inputmask 5.0.9. |
| `view.formula` | Pure Rust arithmetic replaces arbitrary JavaScript eval. Supports decimal/scientific/radix numbers, `+ - * / %`, parentheses and unary minus. Non-arithmetic JS is unsupported and produces 0. Source first-occurrence-only `{this}`, each data key and first unknown-template replacement are retained. IEEE Infinity/NaN display as `Infinity`/`NaN`; parse errors return 0. Original: `frui/src/view/Formula.tsx`, `Formula`. |
| `view.relative`, `view.rel` | Shared Moment thresholds and English/short strings; no Moment locale bundle is added. Browser construction uses native `Date`; native SSR accepts numeric milliseconds, RFC3339 and `YYYY-MM-DD` in UTC. Other browser-specific date spellings have no native parser and display `Invalid date`. Original: `frui/src/view/DateFormat.tsx`, `DateFormat`. |

JSON pass-through also follows `stackpress-view/src/transform/form/field.ts` and
`transform/view/format.ts`: neither generator adds JSON serialization or validation.
Formula data keys from generated source fields are literal identifiers. Arbitrary
application keys containing RegExp syntax are also treated literally; the original
`Formula.tsx` interpolates those keys into JavaScript RegExp fragments. That behavior
outside normalized source field names is unsupported by the native evaluator.

## Source details retained

Country selectors store ISO2 in the native form and return the complete original
country record through onUpdate. Country displays look up ISO3. Clear removes the native form key; the original
country/currency record callback does not publish a replacement for an absent selection. Currency selectors
include all original currencies, with the source USD flag override; currency
views include only fiat currencies with a matching country flag. Size flags on
country/currency views are ignored by the original too. All 198 country records,
116 currency records and the complete color map are retained in original order.
The recorded hashes identify the exact upstream source files; the proof compares
serialized compiled Rust data with every upstream field.

File/image selection calls the supplied upload hook with native browser File
objects. There is no built-in upload or FileReader fallback. Single-file reset
removes the hidden URL without publishing an empty URL; list updates publish the
full list, including an empty list. Captured arrays/queues retain the original
out-of-order completion behavior. `source::roles::SourceUploader` supplies the
hook to generated source-admin controls. Successful file names are URL hidden
fields; the file chooser itself remains unnamed.

The source date/time/number lists use ordinary text inputs because upstream
TextList does not forward the supplied type to its row controls. Removals leave
index holes and Add appends, preserving existing row identities. Existing Typed
collection components and the earlier source string-list adapter remain intact.
Generated fieldsets call the actual selected child role functions and use nested
native form names. Single fieldsets use one-object label/value tables; arrays use
heading columns and one row per object. Optional falsy child displays stay empty.

Carousel `auto` is CSS overflow auto, with no slideshow timer. Initial indices
are not clamped. The original ImageCarousel drops its `frame` prop, so the source
adapter does too; ordinary ImageFilm forwards its frame slot. Native scroll
containers remain keyboard accessible. Radio controls reuse the existing native
Radio; check-style indicators use the pinned Lucide check master, with semantics
on the input and decorative geometry hidden from assistive technology.

## Verification

`proofs/source_roles.py` compiles the actual aggregate `source_client` consumers,
checks managed-output freshness and compiled-data losslessness, then runs cached
headless Chromium using intercepted requests. It exercises hydrate and CSR in
light/dark at 375/1280 widths. Native tests normalize, emit and SSR-render all 108
formerly unavailable placement/name pairs. This evidence is separate from owner
visual acceptance and installed Tauri/platform acceptance.
