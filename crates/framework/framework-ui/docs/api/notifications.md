# Notifications, Notice, and options

Bounded immediate root-local messages. Source: [state.rs](../../src/state.rs).

```rust
use stackpress_framework_ui::{
    leptos::prelude::*,
    state::{NoticeKind, NotificationOptions, Notifications, use_notifications},
};
fn main() -> Result<(), &'static str> {
    let owner = Owner::new();
    let result = owner.with(|| {
        let notices = Notifications::with_options(NotificationOptions {
            max_notices: 2,
            max_message_bytes: 80,
        })?;
        let id = notices.notify(NoticeKind::Info, "Loaded".into())?;
        assert_eq!(notices.notify(NoticeKind::Info, "Loaded".into())?, id);
        assert_eq!(notices.entries().len(), 1);
        assert!(notices.dismiss(id));
        notices.clear();
        Ok(())
    });
    owner.cleanup();
    result
}
fn announce_loaded() -> Result<u64, &'static str> {
    use_notifications()?.notify(NoticeKind::Info, "Loaded".into())
}
```

Contents: [Data](#1-data) · [Store](#2-store) · [Context](#3-context)

## 1. Data

Messages are plain text, not HTML. IDs identify active messages independently of list position.

| Type | Public fields/variants | Meaning |
| --- | --- | --- |
| `Notice` | `id: u64`, `kind: NoticeKind`, `message: String` | Clone, Debug, PartialEq/Eq row. |
| `NoticeKind` | `Info`, `Warning`, `Error` | Copy/Clone, Debug, PartialEq/Eq severity. |
| `NotificationOptions` | `max_notices: usize`, `max_message_bytes: usize` | Copy/Clone, Debug, Default, strict Serialize/Deserialize settings. |

`NoticeKind::label(self)` returns `Information`, `Warning`, or `Error` as `&'static str`.
`role(self)` returns `alert` for Error and `status` otherwise. Both take no additional parameters.
Options default to 20 active messages and 4,096 message bytes. Deserialization rejects unknown
fields; range validation happens in `Notifications::with_options`, not in serde alone.

## 2. Store

`Notifications` is a Copy/Clone signal handle. Create and mutate it in the active owning root's
UI execution context, not as a concurrent cross-thread queue.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `new()` / `Default` | None | Empty store, default budgets, serial starts at zero. |
| `with_options(options)` | `NotificationOptions` | Requires max_notices 1–20 and message bytes 1–4,096. |
| `entries(self)` | None | Reactive read, cloned ordered `Vec<Notice>`. |
| `notify(self, kind, message)` | `NoticeKind`, `String` | Rejects empty/oversize text; coalesces identical active kind+text; enforces capacity and checked ID increment. |
| `dismiss(self, id)` | `u64` | Removes exactly one matching key; unknown/repeated removal is harmless. |
| `clear(self)` | None | Removes all rows without resetting ID sequence. |

**Returns** `new`/Default return Self; `with_options` returns `Result<Self, &'static str>`.
`entries` returns rows. `notify` returns `Result<u64, &'static str>`: an existing active ID for
a duplicate, otherwise a new monotonic ID starting at 1. Invalid text, full capacity, or key
exhaustion returns an error without changing accepted entries/serial. A duplicate still succeeds
at capacity. `dismiss` returns whether it removed a row; `clear` returns unit. Dismissed IDs
are never reused in that root; re-notifying a dismissed message gets a new key.

## 3. Context

`use_notifications()` has no parameters and returns `Result<Notifications, &'static str>`;
missing context yields `Notification state requires LayoutProvider`. No timer, persistence,
cookie, automatic eviction, or global registry is implied. Copying the handle shares its
root's state; disposal ends its validity. Marshal host completions back to the UI root.

Related: [NotifierContainer](notifier-container.md), [LayoutProvider](layout-provider.md).
