# Asset bundle API

AssetBundle is immutable, Clone and Arc-backed. Load verified representations
before build, then register ordinary exact routes. See the
[asset guide](../guides/assets.md) for a manifest and registration example.
Source: [assets.rs](../../src/assets.rs).

## Operations

| Operation | Parameters | Returns and effects |
| --- | --- | --- |
| AssetBundle::load | root: impl AsRef<Path>, manifest: impl AsRef<Path>, limits: AssetLimits | Result<Self>; bounds JSON before parse and synchronously preloads only verified regular files. Manifest is the explicit path supplied, not implicitly joined to root. |
| register | &self, &mut ServerBuilder | Result<()>; installs exact GET and HEAD routes; registration errors propagate under builder rules. |
| len | &self | usize entry count. |
| is_empty | &self | bool. |

Failed loading registers no routes. Validate/load before any listener. Built
Server cannot accept new routes during bind. Startup paths must remain stable
and operator-owned; portable metadata/open is not atomic against hostile writers.

## Manifest and limits

The strict root is an object with `entries: Vec<AssetEntry>`. Unknown/duplicate
JSON fields fail. AssetEntry is Clone/Debug/Deserialize with these required fields:

| Field | Contract |
| --- | --- |
| route: String | Exact absolute ASCII pathname, never a pattern. |
| file: String | Relative ASCII path under the root. |
| size: usize | Exact checked byte count. |
| sha256: String | 64 lowercase hexadecimal characters matching file bytes. |
| mime: String | One exact allowlisted MIME below. |
| cache: AssetCache | revalidate or immutable JSON spelling. |

AssetCache is Clone/Copy/Debug/Deserialize. Revalidate emits public, max-age=0,
must-revalidate; Immutable requires the full content hash in the route basename
and emits public, max-age=31536000, immutable.

AssetLimits is Clone/Debug. Defaults are `manifest_bytes = 64 * 1024`,
`entries = 1024`, `file_bytes = 64 * 1024 * 1024`,
`total_bytes = 128 * 1024 * 1024`. Limits must be positive, byte read caps
representable, and counts/declared aggregate are checked before asset opens.
Each file read is further capped by its declared size; exact size and hash
must match. No request filesystem access occurs.

Allowed MIME strings are `text/plain; charset=utf-8`,
`text/html; charset=utf-8`, `text/css; charset=utf-8`,
`text/javascript; charset=utf-8`, `application/javascript`, `application/json`,
`application/wasm`, `image/png`, `image/jpeg`, `image/gif`, `image/webp`,
`image/svg+xml`, `image/x-icon`, and `font/woff2`.

Route/file components allow ASCII alphanumeric plus `-_.`. Empty, dot/parent,
percent-escaped, Unicode, backslash, pattern, query or fragment segments reject.
Symlink components, directories, FIFOs and devices are not asset inputs.

## Response behavior

GET/HEAD expose explicit MIME, nosniff, verified length and a strong quoted hash
ETag. Weak/strong If-None-Match or `*` may select bodyless 304. The byte source
is immutable and independent of the original file after load. Ordinary
lifecycle hooks retain response authority. There is no range/compression,
directory index or SPA fallback; unknown routes remain 404.
