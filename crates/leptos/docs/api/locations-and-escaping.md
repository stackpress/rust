# Checked locations and escaping

`navigation` and `safety` are available without a render mode. Images and ordinary links have
different policies; do not widen image admission by substituting `safe_href`.

```rust
use stackpress_leptos::{components::ImageSource, navigation::LocalLocation, safety};

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let location = LocalLocation::new("/images/profile.png")?;
    assert_eq!(location.as_str(), "/images/profile.png");
    let _image = ImageSource::new(location.as_str(), 2048, 64, 64)?;
    assert_eq!(safety::escape_html("<name>"), "&lt;name&gt;");
    Ok(())
}
```

`LocalLocation::new(value)` accepts a string convertible to `String` and preserves exact encoded
bytes. It rejects empty/relative/protocol-relative origins, fragments, raw Unicode/whitespace,
malformed percent escapes, literal/encoded dot segments and more than 4096 bytes. Query bytes
remain exact, without decoding/normalization. **Returns** `LocalLocation` or fixed safe
`LocationError`; `as_str` borrows the admitted bytes.

`ImageSource::new(path, bytes, width, height)` additionally requires a local PNG/JPEG/WebP/AVIF
suffix, no query, declared bytes in 1..=1 MiB and dimensions in 1..=4096. It returns checked image
metadata or `DisplayError`; its `Location` variant retains the library's `LocationError`.
`safe_href` remains the separate broader link/navigation policy.

`safety::escape_html(&str) -> String` escapes ordinary HTML text/attribute delimiters.
`safety::escape_script_json(&str) -> String` escapes terminators and Unicode separators in
already serialized JSON. It does not serialize an arbitrary string into JSON or sanitize HTML.
The page integration library keeps its own implementations, with a shared differential regression
table preventing drift where their policies agree.
