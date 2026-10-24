# Status API

Application code/message values, independent of dispatch outcomes. Source:
[status.rs](../../src/status.rs). A native HTTP adapter must validate whether a code is legal for
its transport.

```rust
use stackpress_lib::Status;
use stackpress_lib::status::CODES;

assert_eq!(Status::OK.code, 200);
```

Examples are independent synchronous snippets. `Status` is `Clone`, `Debug`, `Eq`, `PartialEq`, and
`Serialize`; it has public `code: u16` and `message: Cow<'static, str>` fields.

Contents: [Catalog](#1-catalog) · [Construction and lookup](#2-construction-and-lookup)

## 1. Catalog

`CODES: &[Status]` contains the immutable catalog in source order. Associated constants carry the
following exact values; legacy `ABORT` is a catalog value, not `Flow::Abort`.

| Constant | Code | Message |
| --- | --- | --- |
| `CONTINUE` | 100 | Continue |
| `PROCESSING` | 102 | Processing |
| `OK` | 200 | OK |
| `CREATED` | 201 | Created |
| `ACCEPTED` | 202 | Accepted |
| `EMPTY` | 204 | No Content |
| `RESET` | 205 | Reset Content |
| `PARTIAL` | 206 | Partial Content |
| `MOVED` | 301 | Moved Permanently |
| `FOUND` | 302 | Found |
| `REDIRECT` | 303 | See Other |
| `CACHE` | 304 | Not Modified |
| `TEMPORARY` | 307 | Temporary Redirect |
| `PERMANENT` | 308 | Permanent Redirect |
| `ABORT` | 309 | Aborted |
| `BAD_REQUEST` | 400 | Bad Request |
| `UNAUTHORIZED` | 401 | Unauthorized |
| `FORBIDDEN` | 403 | Forbidden |
| `NOT_FOUND` | 404 | Not Found |
| `BAD_METHOD` | 405 | Method Not Allowed |
| `NOT_ACCEPTABLE` | 406 | Not Acceptable |
| `REQUEST_TIMEOUT` | 408 | Request Timeout |
| `CONFLICT` | 409 | Conflict |
| `GONE` | 410 | Gone |
| `LENGTH_REQUIRED` | 411 | Length Required |
| `TOO_LARGE` | 413 | Payload Too Large |
| `TOO_LONG` | 414 | URI Too Long |
| `UNSUPPORTED_TYPE` | 415 | Unsupported Media Type |
| `BAD_RANGE` | 416 | Range Not Satisfiable |
| `BAD_EXPECTATION` | 417 | Expectation Failed |
| `MISDIRECTED` | 421 | Misdirected Request |
| `UNPROCESSABLE` | 422 | Unprocessable Content |
| `LOCKED` | 423 | Locked |
| `BAD_DEPENDENCY` | 424 | Failed Dependency |
| `UPGRADE_REQUIRED` | 426 | Upgrade Required |
| `BAD_PRECONDITION` | 428 | Precondition Required |
| `TOO_MANY` | 429 | Too Many Requests |
| `HEADER_TOO_LARGE` | 431 | Request Header Fields Too Large |
| `LEGAL_REASONS` | 451 | Unavailable For Legal Reasons |
| `PAGE_EXPIRED` | 419 | Page Expired |
| `ERROR` | 500 | Internal Server Error |
| `NOT_IMPLEMENTED` | 501 | Not Implemented |
| `BAD_GATEWAY` | 502 | Bad Gateway |
| `UNAVAILABLE` | 503 | Service Unavailable |
| `RESPONSE_TIMEOUT` | 504 | Gateway Timeout |
| `BAD_VERSION` | 505 | HTTP Version Not Supported |
| `INSUFFICIENT_STORAGE` | 507 | Insufficient Storage |
| `INFINITE_LOOP` | 508 | Loop Detected |
| `NETWORK_AUTHENTICATION_REQUIRED` | 511 | Network Authentication Required |

```rust
assert!(CODES.iter().any(|status| status.code == 419));
```

Iterating `CODES` retains catalog order; it does not generate every possible `u16` code.

## 2. Construction and lookup

Construction preserves application values; lookup only returns actual catalog members.

### 2.1. Status::new

`Status::new(code: u16, message: impl Into<String>) -> Self`

Own a custom label without catalog or transport validation.

```rust
let status = Status::new(9000, "Queued locally");
assert_eq!(status.code, 9000);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `code` | `u16` | Application code. |
| `message` | `impl Into<String>` | Owned custom label; no constructor length check. |

**Returns** An owned `Status`; this constructor is infallible. Response setters apply their own
label limit.

### 2.2. Status::from_code

`Status::from_code(code: u16) -> Self`

Use the catalog label when present and preserve unknown numeric codes.

```rust
assert_eq!(Status::from_code(9999).message, "Unknown");
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `code` | `u16` | Known or custom application code. |

**Returns** An owned status; unknown codes use the exact label `Unknown`, with no error.

### 2.3. Status::lookup

`Status::lookup(code: u16) -> Option<&'static Self>`

Look up a known code without manufacturing a catalog member.

```rust
assert_eq!(Status::lookup(200), Some(&Status::OK));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `code` | `u16` | Catalog code to find. |

**Returns** A static borrowed catalog entry, or `None` for an unknown code.

### 2.4. Status::find

`Status::find(message: &str) -> Option<&'static Self>`

Find an exact catalog message; matching is case-sensitive.

```rust
assert_eq!(Status::find("OK"), Some(&Status::OK));
assert!(Status::find("ok").is_none());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `message` | `&str` | Exact message text. |

**Returns** A static borrowed catalog entry, or `None` when no message equals the input.

## Related

[Errors guide](../guides/errors-and-diagnostics.md) · [Error](error.md) · [Response](response.md) ·
[Events](event.md) · [API index](README.md)
