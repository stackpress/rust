# Round-trip a public page

A browser needs enough data to recreate a page, but it should receive only fields the
application deliberately exposes. Start with a single public string and verify that even
script-looking text survives as data.

## Create a consumer

Use a small binary crate outside this workspace. Add this dependency, replacing the absolute
path with your checkout location:

```toml
[dependencies]
stackpress-framework-view = { path = "/path/to/stackpress/rust/crates/framework/framework-view" }
```

No feature or async runtime is needed. Put the following program in `src/main.rs`:

```rust
use stackpress_framework_view::{PageId, PageSnapshot, PublicStatus, ViewError};

fn main() -> Result<(), ViewError> {
    let page = PageSnapshot {
        revision: 1,
        build: "demo-1".into(),
        page: PageId::try_from("welcome".to_owned())?,
        location: "/ui/welcome".into(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: PublicStatus::ok(),
        data: "Hello </script>".to_owned(),
    };
    let json = page.script_json()?;
    assert!(!json.contains("</script>"));
    let decoded = PageSnapshot::<String>::decode(&json, 1, "demo-1")?;
    assert_eq!(decoded.data, "Hello </script>");
    println!("{}: {}", decoded.page.as_str(), decoded.data);
    Ok(())
}
```

Run `cargo run`. Expected output is `welcome: Hello </script>`. The assertion confirms that
the encoded seed contains no literal script terminator; decoding restores the exact text.

## What happened

`PageId` checked a catalog name. `PageSnapshot` carried version and display settings alongside
your chosen data. `script_json` checked metadata and the logical byte limit, then escaped
characters that HTML's script parser would otherwise recognize.

`decode` checked the expected revision and build before returning data. It checks the envelope,
not application semantics: a larger application still validates its own fields, page membership,
and canonical location. The snapshot does not fetch, render, or authorize anything.

You can now transfer a bounded public seed without losing its text. Continue with
[Concepts](concepts.md), then [add a native page](guides/add-a-page.md).
