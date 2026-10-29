# Add rich content and typed editing

Select `rich` and `format` independently of render mode. Rich content is checked markup;
formatted editing keeps a raw draft separate from a canonical typed value. Neither helper
reads a host locale, database, schema or persistent setting.

```rust
use stackpress_leptos::rich::SafeHtml;
use stackpress_leptos::format::{DisplayLocale,
    edit::{DecimalPolicy, EditSelection, EditState}};
use std::ops::Bound::Unbounded;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let checked = SafeHtml::html("<p><strong>Public text</strong><script>bad()</script></p>")?;
    assert!(!checked.as_str().contains("<script"));
    let policy = DecimalPolicy::new(2, 24, Unbounded, Unbounded)?;
    let mut edit = EditState::new(policy, None, DisplayLocale::French)?;
    edit.input("1234,135", EditSelection::end("1234,135"))?;
    edit.commit(true)?;
    assert_eq!(edit.canonical().as_deref(), Some("1234.14"));
    Ok(())
}
```

`SafeHtml` is the only input to `RichContent`; ordinary strings remain escaped text. Its closed
sanitizer policy, input/output ceilings and pinned Markdown behavior are described by the
[source contract](../../src/rich.rs). It does not admit arbitrary HTML as trusted content.

The decimal uses explicit French grammar and exact half-even rounding, without floating point.
A dirty locale change preserves the draft's captured grammar. `FormattedInput` submits only the
last valid canonical value, retains invalid native text, and handles composition/selection and
commit/cancel separately. Construct it with a root-owned `RwSignal<EditState<P>>`.

The independent consumer renders a real `FormattedInput` and checked rich sample. Its browser
proof exercises Enter rounding, invalid-draft retention, Escape and IME sequencing in both modes.
For the complete decimal/date/time/phone contracts and larger corpus, see the
[format proof](../../../../examples/ui-format/README.md). Generic components remain available
without either optional dependency graph.
