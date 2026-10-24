# Slice 2 design: CLI, HTTP and dependency boundaries

Adapter contract for [task 00005](tasks/00005-slice-2-server-and-cli.md), approved with G12–G16. Load for route capture precedence, native envelopes, CLI syntax/composition/output, prompt ownership and dependency features. Only M0 is released; the [task receipt](tasks/00005-slice-2-server-and-cli.md) distinguishes its probes from the later full adapters/lifecycle.

## HTTP belongs in this slice as a buffered adapter

Include `stackpress-server/http` behind feature `http`, default off, depending on the `http` value-types crate. It receives an already buffered request and returns a buffered native response/report. It starts no socket, task, runtime, listener or gateway. This proves common invocation/normalization semantics while keeping tokio/hyper out of the server's default normal dependency tree.

```rust
pub struct HttpOptions { /* trusted base URL, ParseLimits, response policy */ }
pub struct HttpAdapter { /* validated immutable options; no socket */ }
pub enum HttpOutput {
    Response(http::Response<Vec<u8>>),
    Suppressed,
    Unavailable { state: DeliveryState },
}
pub struct HttpReply {
    pub output: HttpOutput,
    pub report: HandleReport,
}
HttpAdapter::new(options: HttpOptions) -> Result<Self>
HttpAdapter::handle(&self, app: &Server, input: http::Request<Vec<u8>>)
    -> impl Future<Output = Result<HttpReply>> + Send
```

No public `action`/event override argument. Select only `request.method + URI.pathname`; never read an event selector from a URL parameter, body, header or configured arbitrary-event endpoint. Access surfaces still own authorization: the sample exposes only a harmless read capability, with no claim that the host supplies an auth policy.

The application route explicitly delegates to the shared capability (with ordinary lib imports):

```rust
async fn profile_route(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("profile-search", req, res).await
}
async fn route_profiles(start: &mut StartupContext) -> Result<Flow> {
    start.get("/profiles/:id", profile_route)?;
    Ok(Flow::Continue)
}
```

After building that application once, an in-process caller uses the HTTP feature:

```rust
let request = http::Request::builder()
    .method("GET")
    .uri("/profiles/abc123")
    .body(Vec::new())
    .map_err(Error::from_source)?;
let reply = HttpAdapter::new(HttpOptions::default())?.handle(&app, request).await?;
// Match HttpOutput explicitly; only its Response variant contains native output bytes.
```

The CLI call `stackpress emit profile-search --id abc123` reaches the same named operation. The proof compares decoded public envelopes and side-effect counters; neither path starts a listener.

Normalize checked methods, multi-value headers and cookie input into foundation values. Origin-form paths use an explicitly configured trusted base (default `http://unknownhost/`); absolute HTTP(S) URIs retain checked authority. Host/forwarded headers are preserved as data, not trusted to select the URL base. Reject non-path request targets (e.g. authority-form CONNECT or `*`) in this bounded adapter; S2-T can define real proxy/server handling separately.

Preserve path percent escapes, repeated slashes, case and exact pathname; query parsing is separate. No percent-decoding of captures, dot-segment rewrite by custom string manipulation, or inherited Node trailing-slash rewrite. Feed the URI pathname to matching directly, independent of any URL library's normalization for the informational Request.url. Tests must detect a discrepancy that would change selected routes; reject unsafe/unrepresentable normalized input rather than silently route a different target.

Enforce finite request/header/body limits before parsing/handler entry. The caller has already allocated the supplied buffer; this limit is not evidence of bounded network reading. Known JSON/urlencoded/multipart MIME types use existing strict `parse_body`, with atomic `LoadResult` application and explicit supported object-root JSON policy. Text/binary/unspecified MIME retains raw bytes without inventing POST fields; unknown structured formats are not silently parsed. Malformed supported bodies enter Input error handling with normalized method/path/headers available. A request too malformed to normalize gets a fixed public 400 and Input diagnostic before lifecycle callbacks; no fake request is created.

Response conversion validates final HTTP status (200–599 for this completed buffered response; no interim 1xx or custom application codes outside HTTP range), headers and cookie revisions before output. Objects/arrays/scalar results use the existing public envelope; explicit text/bytes preserve their bytes and MIME. Status-only responses use the public envelope except statuses that forbid a body. No developer stack is automatically disclosed. Structured serialization uses `Response::limits`, not a detached envelope's default limits.

HEAD runs matching HEAD/Any contributions without silently falling back to GET; suppress wire body while retaining representation headers where valid. Status 204/205/304 has no wire body; remove inconsistent framing. Adapter owns Content-Length for the actual buffered representation and rejects Transfer-Encoding, which needs a real transport. Preserve repeated header values, append all Set-Cookie values, and retain cookie deletion scope. Do not comma-fold Set-Cookie. Network hop-by-hop stripping is an S2-T boundary, not proof supplied by these value types.

Prepare all fallible wire conversion before the single `Response::dispatch_with` callback that publishes the native result. If conversion fails while Open, attach an Output failure and choose one fixed empty 500 fallback before the attempt; do not rerun error/response hooks. If handler already stopped output return Suppressed; if it finalized/attempted/failed without producing this adapter's native result return Unavailable with report. Never fabricate a second response from Sent. Real network adapters must define connection close/ownership for these states separately.

The adapter envelope owns native output and a private one-attempt token independently of Response, so replacing a Response cannot reset native output authority. Conversion fallback replaces the still-Open framework response with matching sanitized status-500 state before the sole attempt. Native handles/writers never enter CallContext/services automatically; callback-owned external I/O is outside this guarantee.

## Route capture injection and precedence

Host `.route`/method helpers register a wrapper around the original named Handler. For that matched route action only, before invoking user code, validate a small object of its named captures and atomically merge it into Request.data using **literal top-level keys**. Thus route `:id` overrides aggregate `id` from query, post or explicit input. Request.query/post remain unchanged. A name containing punctuation never becomes a dot path. Numeric/regex/wildcard positional captures remain `inv.captures().args`; do not insert numeric keys or reserve a caller-data `args` field.

Each route listener applies its own named captures in the existing global priority/tie order, before that action; later colliding captures win at that point. Prior mutations/captures remain in aggregate data for nested capability calls and following listeners, as in upstream. No implicit restoration/transaction on Abort/error/cancellation. A capture-injection budget failure leaves aggregate data unchanged and returns a normal annotated operation failure. Invocation captures themselves remain immutable and local.

The host does not expose a second global before hook that could override injection. Plugins use lifecycle listeners for broad policy. Existing standalone lib before/after hooks remain unchanged. Host route replacement/removal retains the wrapper/policy and original IDs/sites; do not change lib `route` or ordinary `emit` capture behavior. A raw host dispatch of an explicitly host-wrapped route still runs that wrapper; only raw lib registrations promise no request injection at all.

## CLI library and application executable

Package `stackpress-cli` lives in `tools/cli`, exposing the reusable adapter library. **G14 selects application binaries only**: no stock/empty binary or src/main.rs in tools/cli. The application compiles its own binary named **`stackpress`**, calling the library with its selected Server. The proof app in `examples/capability-host` will compile the profiles plugin once for its CLI and in-process HTTP tests after M0.

This compile-time seam is a real constraint, not a runtime `-b` plugin loader. Later Slice 3 generation may emit the application's main/registration manifest. The initial stock-binary recommendation was rejected in G14; it is no longer an implementation option.

```text
stackpress [--verbose|-v] [--force|-f] [--bootstrap|-b config.json] emit <event> [target args...]
stackpress emit profile-search --id abc123
stackpress -v -b config/test.json emit profile-search --id abc123
cargo run -p stackpress-example-capability-host --bin stackpress -- emit profile-search --id abc123
```

Only tokens before `emit` are operator options. Everything after the event belongs to that capability, including keys named `verbose`, `force` and `bootstrap`. This deliberate boundary removes ambiguous global flag stealing; unlike upstream examples, `-b` after the event is target data. The existing strict argv parser handles target tokens directly, preserving spaces, false/0/null, JSON and `--`; never join arguments or re-read process argv inside an event.

Require nonempty event; usage/invalid options/non-UTF-8 arguments return exit 2 with concise stderr. No missing-value heuristic for target long flags beyond lib's approved grammar; bare operator booleans are recognized before emit. `force` is request-local operator metadata only; it authorizes no command implicitly. `-b` reads exactly one explicitly named JSON data file, under bounded read and normal I/O errors; no extension search/executable imports. Explicit application env/code overlays can follow it in documented config order.

```rust
pub struct Terminal { /* immutable parsed command, target args/data, options */ }
pub struct TerminalOptions { /* verbose, force, config path, brand, color */ }
pub struct CliContext { /* request-local operator metadata; no I/O handles */ }
pub struct CliRun { pub exit_code: u8, pub report: HandleReport }
Terminal::parse(args: &[String], limits: ParseLimits) -> Result<Self>
Terminal::expect<T: DeserializeOwned>(&self, aliases: &[&str]) -> Result<Option<T>>
run(app: &Server, terminal: Terminal, sinks: &mut CliSinks<'_>)
    -> impl Future<Output = Result<CliRun>> + Send
```

`CliSinks<'a>` borrows `stdout` and `stderr` as `&'a mut (dyn std::io::Write + Send)` for bounded buffered writes. It has no Clone/global lock; the explicit Send bound keeps run's future movable. Synchronous caller-provided writes may block and are not implicitly cancellable. Parse/build error results are mapped by the executable wrapper; capability handling enters `Server::handle` exactly once with `terminal/arguments` MIME and parsed data. CliContext is passed in top-level host call options, held alongside the host's CallContext, never inserted into untrusted Request.data. Expose it with `ServerInvocationExt::adapter_context<T>() -> Option<&T>` and propagate it to nested calls. This adds only server context fields, not another lib callback signature.

Emit one JSON envelope/newline on stdout for ordinary Open output, including application errors/unknown events. No branding/progress/ANSI/prompt/cause text on stdout. Use stderr for optional progress and metadata diagnostics; default errors stay concise, verbose output is an explicit developer formatter. Final output preserves status/results/errors/total and uses the owning response's limits. Stopped output prints nothing; an already delivered/failed/attempted response is never retried. Writer/serialization failure yields exit 1; broken pipe does not trigger another stdout write.

Exit 0 requires final 2xx/3xx status with no public error and no terminal/delivery failure, including deliberate successful error-handler replacement. Exit 1 covers application/unknown-event/unrecovered/terminal/output failure. Abort alone is not failure; an empty aborted reply normalizes to 204 and exit 0. Explicit prompt cancellation maps to 130 at the executable boundary. These codes do not equal HTTP/application status values.

Application binary sketch, with concrete error mapping supplied by library main helper during implementation:

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    // Application owns compile-time selection, environment inputs and its server builder.
    stackpress_cli::main_with(build_application).await
}
```

`main_with<F, Fut>(build: F)` accepts `F: FnOnce(ConfigLayers) -> Fut + Send` and `Fut: Future<Output = BuildResult<Server>> + Send`. A named `async fn build_application(layers: ConfigLayers)` installs those layers on its builder, selects plugins explicitly and awaits build. The later helper reads argv/sinks once, parses data-file options before consuming that function, then calls run. M0 proves the callback/Send/config-before-build seam in a disposable consumer; it does not add a partially implemented main_with/command pipeline to the library before M3. That probe uses raw resolve explicitly and is not CLI lifecycle evidence. The smaller direct `application()` sketch in the host API needs no CLI inputs.

## Control and prompt ownership

`Control` owns brand/level/color policy and borrows an explicit writer per call; methods output/error/info/success/system/warning substitute `%s` in order and return I/O errors. `ColorMode::{Auto,Always,Never}` defaults to Auto, requiring TTY and honoring NO_COLOR. Brand/progress values cannot smuggle control sequences. Core formatting/validation is pure and tests use buffers.

`PromptOptions` and `PromptState` model default, required, Tab/Editable prefill, Keep/Clear invalid answers, optional Rust regex/error, final transformer and sync/async validator. `PromptEvent` models text, backspace, Tab, Enter, Cancel, EOF and validation completion. `Validation::{Valid,Invalid(String)}` avoids JS false/string coercion. Required/pattern checks precede user validation. Bound answers/error/render text; delete complete Unicode scalar values. Serialize edits/validation; accept cancellation while validation awaits and discard stale completion after cancellation. Validator/transform/I/O errors terminate cleanly with their causes.

The optional native TTY driver owns one terminal-input session, guarded by an explicit exclusive lease. A joinable input worker polls with a bounded interval and sends bounded events; an atomic stop signal and receiver closure break poll/send waits. It holds/restores prior raw-mode state through RAII on success/error/drop/unwind, and never holds a framework mutex across await. Normal completion awaits worker completion before join. Cancellation/drop closes the queue, signals stop and joins the framework-controlled bounded polling worker before releasing ownership; M0 must prove this exceptional synchronous cleanup bound and readiness-before-read contract. Never invoke a user validator/writer inside that worker. No detached blocked stdin thread or `spawn_blocking` read that prevents runtime shutdown.

For non-TTY output, render readable prompt/error/final text without cursor/erase/color escapes. The pure prompt driver accepts supplied buffered events/lines and supports pipe-style fixtures. Native non-TTY stdin does not start an uncancellable blocking reader: use an explicit/default answer or return NonInteractive/EOF. Full asynchronous native pipe polling can be added with its own OS proof; this limitation and its difference from arbitrary Node streams must be documented and approved. No `emit` invocation prompts automatically.

## Dependency budget and features

G16 approves this dependency budget. M0 resolved http 1.5.0 and crossterm 0.29.0 (`default-features = false`, `features = ["events"]`); exact graph/features are retained in Cargo.lock and the task receipt. No dependency beyond the approved budget was introduced. Server default and HTTP normal-edge trees contain no Tokio/hyper; crossterm's native polling/synchronization/signal dependencies belong to CLI only.

| Crate / edge | Dependencies and justification |
| --- | --- |
| server default normal | `stackpress-lib`; direct `serde` with derive for public typed config; `serde_json` with preserve_order for explicit data layers. Reuse lib's public types/parsers; standard collections/Arc/Any/atomics suffice for registries/reports. No executor, filesystem, network, tracing or logging dependency. |
| server `http` normal | Optional `http` 1.x value types (standard support, no runtime feature); Vec<u8> avoids an extra direct bytes/body/stream abstraction. Review its transitive bytes dependency normally. No hyper/tokio/tower/axum server. |
| CLI normal | `stackpress-server` default features off; direct `stackpress-lib`, serde_json for output, serde for typed lookup and regex for pure PromptOptions pattern compilation; Tokio only rt/macros/sync/time for main, validator selection and bounded worker completion. No net/process/signal/full feature. |
| CLI optional `tty` (native default) | `crossterm` with terminal/event support only, no event-stream or serde integration. Confirm exact backend feature names/platform edges before locking; avoid a new polling/line-editor dependency unless the M0 proof demonstrates need and coordinator reviews it. |
| test-only | Existing Tokio macros/rt-multi-thread/sync/time and fixtures for scheduling/deadlines; no bound ports. Prefer std process watchdog helpers already in lib tests. |
| example | CLI/server http feature + serde for typed proof config/results and Tokio for native entry; one compile-time optional experimental plugin used in selected/disabled build proofs. |
| root facade | Optional path dependency server, `server-http = [server, stackpress-server/http]`. Existing default lib dependency remains compatible. |

If HTTP or prompt work needs a new dependency beyond this budget, explain the concrete requirement and return it for review before expanding the implementation. Native networking, SSE/WebSockets, TLS, streaming, serverless, MCP, auth/session identity, Leptos and Tauri remain separately scoped. HTTP-value integration is not proof of a real listener or platform deployment.
