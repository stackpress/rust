# View providers and supervised packaging

Enable `view` for native source configuration/render/build bridges. Supply actual render and build implementations as application services. The aggregate does not select a renderer or a Node build engine.

## Services and rendering

`register(app) -> Result<()>` claims view, queues Config/Listen/Route and uses ConfigPresence. Public `Section(pub Option<Value>)` is the transparent raw view wrapper; activation uses explicit key presence rather than Option membership. Settings contains original config, source mode (production default) and resolved cwd. Catalog contains ordered native entry identities. NativeHttp is a trusted adapter-context marker, not caller data.

`NativeFuture<T>` is an owned Send future returning Result<T>. Render is an Arc callback `(String, Value) -> NativeFuture<Option<RenderedPage>>`; Provider.render owns it. `render(action, req, res, inv) -> async Result<Flow>` normalizes status, skips redirected responses, the configured noview data flag (default json), or an existing text body. It raw-emits me into a separate response, merges view.props then response data, and supplies source data/session/request/response props to the provider. Some HTML replaces body while retaining status; None leaves it unchanged.

Native Settings, Client and provider graphs are not serialized into props. The source intentionally includes request/page session values and explicit form/enrollment values; absence of native secrets does not mean all session-shaped values are removed. See [session browser boundary](../../../framework-session/docs/guides/shared-ui.md).

`development(req,res,inv)` skips non-native HTTP calls and production mode; otherwise invokes view-http. Its literal request hook is registered during Route, after earlier equal-priority Listen request hooks. It does not permit general Route-phase `on` calls. Existing provider Flow/errors are preserved.

## Build callbacks

Build is an Arc `Fn() -> NativeFuture<Value>`. BuildProvider.create receives owned cwd/engine options and returns BuildEngine: set(String), size(), clients, assets and pages callbacks. These callbacks are compiled native functions; catalog entry strings do not dynamically import code.

`build(&Invocation) -> async Result<Vec<Value>>` loads all Catalog entries before output checks; size zero returns an empty vector. Truthy engine.clientPath/assetPath/pagePath select stages in that order, short-circuiting on the first error. Contents are truncated to 100 source UTF-16 units plus space/ellipsis; an isolated cut surrogate uses U+FFFD as a Rust string mechanics boundary.

`build_event(req,res,inv) -> async Result<Flow>` ignores prior non-OK status, awaits build, then submits optional packaging and stages 200. It does not return the build script's vector as response results. The [website guide](../guides/website.md) links a complete finite compiler proof and distinguishes renderer artifacts.

## Packaging lifecycle

These native supervision types preserve source build.ts:21 non-awaited timing. There is no built-in npm manifest writer.

| Type / method | Contract |
| --- | --- |
| PackagePaths | cwd and build PathBuf values; build defaults to cwd/.build. |
| Package | Arc callback `(PackagePaths) -> NativeFuture<()> + Send + Sync`. |
| PackageOutcome | Result<Result<()>, tokio::task::JoinError>; provider failures remain distinguishable from task failures. |
| Packaging::new(runtime, callback) | Supplies an explicit runtime Handle and callback; no spawn during construction/registration. |
| drain().await | Returns all retained outcomes; cancellation leaves unfinished jobs supervised. It does not alter the build response. |
| shutdown().await | Stops submissions, aborts pending work and reaps it, including when drain is live. |

Retain an Arc lifecycle handle and publish it in Config. No callback means no packaging work. Slow/failing callbacks do not delay or fail the source build response; failures are observable via drain. Dropping the final owner aborts unfinished work. A per-instance async completion gate serializes waiters; the synchronous task-state mutex is released before Pending and before app code.

Evidence: [view source](../../src/native/view.rs), [supervision](../../src/native/view/packaging.rs), [render/skip/build/truthiness/shutdown tests](../../tests/cases/view.rs), [Route request-hook test](../../../../server/server/tests/on_request.rs).
