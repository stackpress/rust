# Design: data, request/response, and error contract

Spec file of [00001 Stackpress Rust foundation](index.md). Load for Slice 1 data/parsing rules, request/response/session values, and the error/diagnostic model (design sections B5–B7). Moved losslessly from `docs/trial-3/slice-1-design.md` lines 447–579 at commit `e230359`; headings are demoted one level and links rebased.

#### B5. Data and boundary parsing

Following [00005's lib redesign boundary](../../references/00005-discussion-feature-feasibility.md), `Value` is richer than JSON. Owned variants are null, bool, validated finite JSON number, string, array, insertion-ordered object, **DateTime**, **Bytes**, **File**, and **Shared**. Missing is represented by absent lookup, never by a null variant.

| Non-JSON variant | Ownership and representation | Boundary behavior |
| --- | --- | --- |
| `DateTime(Timestamp)` | A distinct checked UTC instant, proposed signed epoch seconds + nanoseconds; no implicit conversion to a string/number | A caller selects a wire representation explicitly. Calendar/time-zone presentation is outside this primitive. |
| `Bytes(Vec<u8>)` | Owned bytes distinct from strings and files | Preserve arbitrary bytes, including invalid UTF-8; no implicit lossy text/base64 conversion. |
| `File(FileData)` | Owned bytes plus filename/MIME metadata | Empty files remain files; never write the untrusted filename to disk. |
| `Shared(SharedNative)` | Explicitly shared `Arc<dyn Any + Send + Sync>` with typed construction and borrowed downcast | Cloning shares only this named variant. Payload access is immutable; users explicitly choose any interior synchronization in their payload type. |

Default JSON conversion of DateTime/Bytes/File/Shared returns a path-qualified unsupported-value error. A caller may deliberately construct a JSON date/byte/file envelope before output. Shared native payloads never serialize through Debug or an inferred type name, and cannot enter an external request from JSON. No recursive user-defined wire codec framework, arbitrary JS object coercion, NaN/Infinity or executable function reflection is introduced. Owned Value containers are acyclic; a native payload may have application-defined internal sharing/cycles and the framework never traverses it.

`Value` and `Nest` implement Clone: scalar strings, arrays/objects, dates, bytes and files clone independently; only `Value::Shared(SharedNative)` shares its immutable native payload. `SharedNative` names that sharing in the public type and exposes no unchecked mutable access. No ordinary data node/controller aliases through Clone. Generic application payloads may deliberately contain synchronization or internal sharing; that is explicit at Shared construction.

Query/post→data initialization can clone uniformly, including explicitly Shared native values, removing the rejected `try_copy`/native-copy limitation. Request and Response themselves will **not** implement Clone: load/delivery attempt state and output ownership must not become a second independently usable invocation. Their data-bearing members are clonable values, so callers can deliberately construct a fresh Request from cloned data without duplicating an attempted native read/send. Invocation/registries/queues/owned error causes likewise have no blanket Clone.

Key data signatures:

```rust
Nest::new() -> Nest
Nest::from_object(value: serde_json::Value) -> Result<Nest>
Nest::get(&self, key: &str) -> Option<&Value>
Nest::has(&self, key: &str) -> bool
Nest::set(&mut self, key: &str, value: impl Into<Value>) -> Result<&mut Self>
Nest::remove(&mut self, key: &str) -> Result<Option<Value>>
Nest::path<'p>(&self, path: impl IntoPath<'p>) -> Option<&Value>
Nest::path_as<'p, T: DeserializeOwned>(&self, path: impl IntoPath<'p>) -> Result<Option<T>>
Nest::has_path<'p>(&self, path: impl IntoPath<'p>) -> bool
Nest::set_path<'p>(&mut self, path: impl IntoPath<'p>, value: impl Into<Value>) -> Result<&mut Self>
Nest::remove_path<'p>(&mut self, path: impl IntoPath<'p>) -> Result<Option<Value>>
Nest::merge(&mut self, other: Nest) -> Result<&mut Self>
Value::to_json_with(&self, limits: &ParseLimits) -> Result<serde_json::Value>

SharedNative::new<T: Any + Send + Sync>(value: T) -> SharedNative
SharedNative::downcast_ref<T: Any>(&self) -> Option<&T>
Value::as_str(&self) -> Option<&str>                // analogous checked scalar accessors
DataPath::dot(text: &str) -> Result<DataPath>
DataPath::separated(text: &str, separator: char) -> Result<DataPath>
DataPath::from_segments(segments: impl IntoIterator<Item = PathSegment>) -> Result<DataPath>
pub enum PathSegment { Key(String), Index(usize), Append }

parse_query(text: &str, limits: &ParseLimits) -> Result<Nest>
parse_args(args: &[String], skip: usize, limits: &ParseLimits) -> Result<Nest>
parse_arg_string(text: &str, limits: &ParseLimits) -> Result<Nest>
parse_json_object(text: &str, limits: &ParseLimits) -> Result<Nest>
parse_multipart(body: &[u8], boundary: &str, limits: &ParseLimits) -> Result<Nest>
parse_body(content_type: &str, body: &[u8], limits: &ParseLimits) -> Result<Nest>
```

Mutation prepares the full path and structural changes read-only, measures incoming/replaced subtrees, and checks a delta against cached node/byte totals before committing in place. Unrelated payloads are not cloned or revalidated. Container normalization/deletion may scan or move the affected container. All returned errors leave the tree unchanged. Replaced/cleared native values are dropped only after tree and totals agree, preserving invariants through a user destructor panic. Missing is `None`; present null is `Some(Value::Null)`. Empty objects remain objects. `merge` is an incoming-wins **top-level overlay**, matching `Nest.set(object)`; it is not a speculative recursive config merge. Intermediate scalar/null/date/byte/file/shared-native traversal returns a path/type error rather than silently replacing data or panicking. Append uses the next available canonical numeric key, never object key count where that could overwrite an existing sparse index. Nonempty dense canonical numeric children normalize in numeric order; sparse/noncanonical/negative keys remain objects without huge index-based allocation. Removing an array element preserves remaining index identities by converting that container to a numeric-keyed object, rather than shifting values or equating a hole with null. Empty path is rejected for mutation; root read/clear are distinct methods. `path_as` checks conversions; it does not stringify shared-native/date/byte data to pretend deserialization succeeded.

Ordered `DataMap`/`DataSet` may use documented aliases/re-exports of `indexmap` collections, with Rust iterator equivalents for filter/find/map and explicit conversion examples. No duplicate bespoke collection hierarchy is necessary. Borrowed entries/keys/values and `len` replace controllers/`size`; Rust's `Option<usize>` replaces a set-search `-1` sentinel. A sequential async nested visitor preserves upstream early-stop behavior, with empty/missing collections producing an explicit empty outcome. JSON object key order is retained by the configured serializer; no promise of byte-identical whitespace except explicit pretty/compact modes.

Approved parser rules (D3), including the deliberate differences below:

| Boundary | Proposed contract |
| --- | --- |
| Common limits | Finite defaults: 1 MiB buffered input, 64 nesting levels, 10,000 entries/parts, 8 KiB multipart part headers. Count before allocation/insertion and bound outputs; callers can pass explicit alternatives. Depth is checked for supplied JSON and already-parsed values too. No unbounded recursive parsing or sparse index allocation. |
| Query / URL-encoded | Optional leading `?`; empty input gives empty Nest; split first `=` only; decode percent escapes once, `+` to space for both names and values; reject malformed encoding/UTF-8. Bracket paths allow explicit indices and append; repeated ordinary keys replace, `[]` appends. Parse finite decimal numbers, exact true/false/null and valid object/array JSON values; malformed JSON-like scalar text stays text. Empty values stay empty strings. |
| Args | Parse supplied argv directly; never join it into a query string. Support tested long-key/value, grouped short flags, numeric positional keys, bracket paths and skip offset. Also support explicit `--key=value` without allowing `&`, `=`, `+` or `%` inside a value to become new fields. Ignore lone `-`; bare `--` ends option parsing (a final bare marker adds nothing). A dangling long option is an error. Values immediately following a long key are taken as values, including a leading minus. |
| Argument string | Small documented whitespace/quote tokenizer, single/double quotes and quoted empty values; no expansion, commands, environment access or claim to full shell syntax. Unclosed quotes are errors. Share only scalar/path policy with query parsing, not delimiter interpretation. |
| JSON object | Validate JSON after permitted leading whitespace; require an object root, preserve false/0/null, reject malformed/non-object JSON. Use the same depth/entry/input limits. |
| Multipart | Caller supplies parsed Content-Type boundary; recognize delimiters only in valid positions, require final termination, preserve exact file bytes (including CR/LF and boundary-like content), validate text UTF-8, ignore parts without a usable field disposition as upstream tests expect, apply nested names/scalar policy to text only. Empty named files remain files. Support quoted boundary and repeated/append fields; reject truncation/invalid limits without partially mutating an existing Nest. No disk spooling/streaming support claim. |
| MIME dispatch | Parse MIME essence/parameters case-insensitively; support JSON, URL-encoded and multipart deliberately, including JSON charset parameters. Unsupported media types return a typed error rather than silently manufacturing empty data. |

#### B6. Request/response/session values

Request sketch (loader signatures return borrowing futures whose output is `Result<()>`):

```rust
Request::new() -> Request
Request::builder() -> RequestBuilder
RequestBuilder::build(self) -> Result<Request>
Request::load(&mut self) -> Result<()>             // finalize with no loader
Request::load_with(&mut self, loader: /* one-shot callback producing a Send future of LoadResult */)
    -> /* borrowing Future<Output = Result<()>> */
pub struct LoadResult { pub body: Option<Body>, pub post: Option<Nest> }
pub enum LoadState { Unloaded, Attempted, Loaded, Failed }
```

Builder inputs include URL, method, headers, body/MIME, query override, post, explicit data, initial session and parse limits. Defaults preserve GET, `http://unknownhost/`, text/plain and empty body/collections. Prefer explicit `Body` variants rather than JS truthiness/type probing; bytes unify Buffer/Uint8Array. `Body::Empty` and `Body::Value(Null)` remain distinct. URL parsing does not trust forwarded headers or grant network access. Explicit initial session wins over cookies; explicit query wins over URL query. Parsed object values are not re-coerced. Initialization overlays query → post → explicit data; successful load later overlays loaded post into post/data, matching upstream sequence. Later direct edits of query/post do not magically resynchronize data.

`load_with` accepts an owned, isolated `RequestView` data snapshot and returns staged data: `F: FnOnce(RequestView) -> Fut + Send`, `Fut: Future<Output = Result<LoadResult>> + Send`. The view is Clone but contains no load-attempt state; ordinary snapshot data is independent, with sharing only through Value::Shared. This permits named async functions and closures borrowing a caller-owned Send native reader without a static callback bound. Snapshot edits cannot partially mutate the original. Exclusive mutable borrowing prevents simultaneous loads. Successful load/no-loader finalization occurs once. Before awaiting, record Attempted; success atomically installs validated output and records Loaded; error records Failed. A cancelled/dropped attempt stays visibly Attempted, and a later load returns an incomplete-attempt error rather than repeating a potentially consumed native reader. No implicit retry; the owner accepted this one-attempt contract (D4). Native transport request/stream holders remain outside this value in a later adapter-owned envelope; typed in-process values use Value::Shared. Successful load clones staged post values into aggregate data according to the explicit Shared semantics. Final validation enforces the request's selected budgets even if a caller replaced its public Nest stores.

Response sketch:

```rust
Response::new() -> Response
Response::status(&self) -> Option<&Status>          // None replaces code 0/empty message
Response::status_code(&mut self, code: u16, message: Option<String>) -> Result<&mut Self>
Response::set(&mut self, mime: &str, body: Body) -> Result<&mut Self>
Response::html(&mut self, text: impl Into<String>) -> Result<&mut Self>
Response::xml(&mut self, text: impl Into<String>) -> Result<&mut Self>
Response::json<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self>
Response::results<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self>
Response::results_value(&mut self, value: Value) -> Result<&mut Self>
Response::rows<T: Serialize + ?Sized>(&mut self, rows: &T, total: u64) -> Result<&mut Self>
Response::redirect(&mut self, location: &str) -> Result<&mut Self>
Response::set_error(&mut self, error: PublicError) -> Result<&mut Self>
Response::apply_error(&mut self, error: Error) -> Result<&mut Self>
Response::into_error(self) -> Error                // retains an owned original cause when present
Response::apply_envelope(&mut self, envelope: ResponseEnvelope) -> Result<&mut Self>
Response::to_envelope(&self) -> ResponseEnvelope    // infallible rich-value clone
Response::into_envelope(self) -> ResponseEnvelope  // move owned in-process values
ResponseEnvelope::try_to_json(&self) -> Result<serde_json::Value>
Response::dispatch_with(&mut self, dispatcher: /* one-shot callback producing a Send future of () */)
    -> /* borrowing Future<Output = Result<()>> */
Response::dispatch(&mut self) -> Result<()>         // explicit no-output finalization
Response::stop(&mut self) -> Result<()>
pub enum DeliveryState { Open, Attempted, Sent, Stopped, Failed }
```

HTML/XML/JSON/results/rows/set default to OK; redirect defaults to Found and sets validated Location. `json` stores serialized text, while results/rows retain structured JSON; results sets total=1 and rows uses the supplied total including zero. Raw JSON text, if needed, is explicit text plus MIME rather than double-guessing strings. Status/total overrides use explicit setters after a convenience method. Headers are validated, case-insensitive and multi-value; never comma-fold Set-Cookie. Mutable state is private behind checked methods/accessors so finalization cannot be bypassed by public field writes.

Envelope application distinguishes absent fields from present zero/false/empty/null. It stages validation before mutation, and does not copy delivery state/native transport resources. `results_value` supports richer in-process results, including dates/bytes/explicitly Shared native values. `into_envelope` moves those results without Clone; borrowed `to_envelope` clones ordinary data independently and retains only explicitly Shared native identity. In the envelope, Empty becomes absent results, Text becomes a string value, Bytes stays a byte value, and Value moves unchanged. The envelope is a result/status view, not a full copy of headers/session/MIME/body delivery state. The explicit JSON conversion rejects non-JSON values with their exact path, without silently erasing type distinctions. `apply_error` owns and retains the original Error for developer inspection/consuming `into_error`; only its explicit public view reaches an envelope. If no original cause was supplied, `into_error` constructs an error from the response's status/public fields and does not invent a source chain. The upstream status catalog includes custom application codes; an unknown `u16` code retains its value with `Unknown` unless a custom label is supplied. A later HTTP adapter must validate native status legality rather than assume every application status is HTTP-valid. `Status::lookup`/message lookup return Option; custom 309 remains a legacy catalog entry, never the event abort result.

`dispatch_with` passes an owned, isolated `ResponseView` data snapshot to a supplied async callback: `F: FnOnce(ResponseView) -> Fut + Send`, `Fut: Future<Output = Result<()>> + Send`. Like RequestView it is Clone without delivery/native state or owned private error causes. Native ownership/results stay with the caller, including a borrowed Send native writer. It revalidates the response budgets before snapshot/output, then records Attempted before awaiting, Sent on success, Failed on error. Repeated success is a no-op; failed/interrupted attempts are not automatically retried. `dispatch()` finalizes without native output. `stop()` records Stopped without invoking a dispatcher. All terminal/attempted states reject subsequent value mutation; no error path claims bytes were never partly sent. `sent()` retains the legacy “finalized” convenience for Sent/Stopped, while `delivery_state()` is authoritative. Drop cancellation preserves Attempted. These are the accepted D4 choices, not guarantees inferred from upstream's bool flag.

Session contains ordered string/string-list values, per-key validated cookie options and an ordered latest-revision-per-key map. Initial values produce no outgoing revisions. Value-only set retains that key's options; explicit options replace them. Delete/clear remove options from live state but **copy their prior domain/path scope into the removal revision**, allowing the later adapter to expire the same cookie instead of an unrelated default-path cookie. Removing a missing name still creates a tombstone. Arrays require an explicit adapter serialization policy; the foundation will not silently collapse a string list.

Cookie parsing keeps upstream first-value-wins, whitespace trimming, ignored malformed fragments and raw fallback for undecodable values; query decoding remains strict. Checked serialization supports domain/path, expiry, signed integer max-age, httpOnly, secure, partitioned, typed Priority and SameSite, and optional custom codecs whose output is revalidated. Validate option combinations separately from application security policy; no “session” value is proof of authentication. Session/Request Debug output must avoid dumping credential values.

#### B7. Error and diagnostic model

`Error` implements `Display` and `std::error::Error`, owns an `ErrorKind`, developer message, optional application Status/public message, validation errors, optional bounded byte `SourceSpan`, origin/registration locations, an optional boxed `Send + Sync + 'static` original cause and invocation breadcrumbs. It does not implement Clone or serialize itself implicitly. Known framework/parser conversions support `?`; arbitrary external errors use `map_err(Error::from_source)?` without a conflicting blanket `From<E>` implementation.

Typed kinds distinguish invalid path/type/encoding, parser/limit failures, invalid matcher/registration, incomplete load/delivery, finalized response mutation, dispatch-depth exhaustion and handler/domain failure. Returned ordinary failures are not panics. Native backtrace capture is optional and original-capture preserving; absence does not remove useful reports. No tracing subscriber, raw request/session logging or V8 stack parsing is needed for baseline diagnostics.

For every failed phase, report the actual emitted event, matcher/captures' identity (not automatic sensitive values), listener ID/label, plugin when supplied, priority, original registration location, phase and complete nested event/listener path. Preserve the deepest failure and cause while outer propagation adds context without duplicating the same path. Anonymous listeners are identified by ID/site; do not infer stable function names from erased Rust types. Caller/source locations describe what was actually captured, not an invented precise failing expression inside arbitrary third-party futures.

Public error/envelope conversion requires an intentional public message/field-error view and excludes paths, plugin internals, causes, native backtraces and registration locations by default. A separate developer diagnostic view renders those fields. Transport correlation and lifecycle stage enrichment are later host responsibilities. Panics propagate normally; this slice neither promises panic recovery nor turns aborting panics into an error event. Application-owned locks may still deadlock/poison; the engine does not acquire or unwrap them.
