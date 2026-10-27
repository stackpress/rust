# Serve verified asset bundles

Preload known build output once, then register exact asset routes before the
application builds. Requests read immutable bytes; no request-controlled path
ever reaches the filesystem.

## Prepare a manifest

The checked [example manifest](../../examples/assets/manifest.json) describes
[hello.txt](../../examples/assets/hello.txt). Its SHA-256 and size cover the exact
27 bytes, including the final newline:

```json
{
  "entries": [
    {
      "route": "/assets/hello.txt",
      "file": "hello.txt",
      "size": 27,
      "sha256": "ea2eca2a79a652bbb72eeff21730aae90ec0ea1f6d230b0bdae34330c6791d85",
      "mime": "text/plain; charset=utf-8",
      "cache": "revalidate"
    }
  ]
}
```

The root and entries reject unknown and duplicate fields. Set every field
explicitly: route, relative file, exact byte size, lowercase full SHA-256,
allowlisted MIME, and cache policy. See [asset reference](../api/assets.md)
for budgets and the complete MIME list.

## Register before build

Load a stable operator-owned tree and register its routes on ServerBuilder:

```rust
use stackpress_server::{Server, ServerBuilder};
use stackpress_server_http::{AssetBundle, AssetLimits};

/// Verify all representations before building the immutable application.
async fn application() -> stackpress_lib::Result<Server> {
    let bundle = AssetBundle::load("dist", "dist/manifest.json", AssetLimits::default())?;
    let mut builder = ServerBuilder::new();
    bundle.register(&mut builder)?;
    builder.build().await.map_err(|failure| failure.error)
}
```

Loading verifies the bounded manifest and every size/hash before exposing the
bundle. Registration creates ordinary GET and HEAD routes; request guards,
response/error hooks and cookies keep their normal lifecycle authority.
Built applications cannot gain routes during bind.

## Choose cache semantics

GET and HEAD return the verified length, explicit MIME, nosniff and a strong
quoted hash ETag. If-None-Match accepts weak/strong tags or `*` for bodyless 304.
`revalidate` emits `public, max-age=0, must-revalidate`. `immutable` emits
`public, max-age=31536000, immutable` and requires the full hash in the route's
basename. Response hooks may still edit or suppress this ordinary response.

Only regular files are accepted. Symlink components, FIFOs, devices, directories,
dot/parent segments, empty segments, patterns, percent escapes, Unicode,
backslashes, query and fragments are rejected. Route/file components use ASCII
letters/digits and `-_.`; file paths are relative to the selected root.
Keep startup files unchanged while loading; metadata/open is not atomic against
hostile concurrent writers.

After load, filesystem modification or removal cannot change the retained
representations. Unknown routes remain 404. There is no directory/SPA fallback,
range response or compression. [Immutable byte streaming](../../../server/docs/api/streaming-http.md)
shares storage and copies at most one bounded chunk per demand poll.
