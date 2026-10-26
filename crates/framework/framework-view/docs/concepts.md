# From a capability to a public page

A page combines work done by a host with data a browser may see. Treat the loader as the
boundary between them: it borrows the live invocation, then returns owned public data.
The [quick start](getting-started.md) shows the resulting envelope independently of any renderer.

## Loading is separate from rendering

A named async loader returns `Result<PageLoad<T>>`. `Ready(T)` supplies owned data;
`Abort` cooperatively stops the operation and skips rendering. An error follows the existing
server error lifecycle. A loader can use `Invocation::delegate` for nested raw dispatch;
starting another top-level handle would duplicate lifecycle work.

The native extension constructs a snapshot from that data, the renderer's `PageContract`,
the checked public location, and the response's explicitly public status/error. It gives
`PageRenderer<T>` no native output authority. A complete `RenderedPage` is staged through
the existing response; [UI rendering](../../framework-ui/docs/concepts.md) owns Leptos cleanup.

## A snapshot is a disclosure decision

Only the fields of `PageSnapshot<T>` and the application-owned `T` are serialized. Requests,
cookies, sessions, services, config, and developer reports are never forwarded implicitly.
Keep `T` small and public; serde derives alone do not decide which information is appropriate.

For example, a search page can disclose rows, a total of zero, and the validated query that
produced them. It should not serialize the request object used to obtain those rows. The
default public location includes the actual pathname only. A `PageSpec::location` projector
can deliberately reconstruct a public query from `T`.

## Compatibility and limits

Revision identifies the app's wire schema; build identifies matching assets. Both must match
before hydration. Locale and theme are explicit initial settings, so the client recreates the
same first view. `PublicStatus` is a page/HTTP status, independent of Completed/Aborted/Failed
dispatch outcomes.

The standard limits are 1 MiB logical snapshot JSON and 2 MiB complete HTML. Applications may
lower either, including to zero. Escaping can expand JSON, so decoding also caps encoded input
at six times the snapshot limit and rechecks logical serialized size. Final HTML is checked
after Head, seed, and assets are assembled. These are output limits, not a CPU sandbox.

## Routes describe compiled choices

`PageRoute` describes literal segments and required parameters; `RouteCatalog` adds an explicit
mount. A catalog supplies native patterns and a [Leptos path adapter](../../framework-ui/docs/api/routing.md).
It never loads code by caller-supplied names or replaces native all-matching route dispatch.

Locations retain encoded parameter text. Applications own decoding and query policy;
the catalog does not canonicalize a query or resolve overlapping route definitions.
`BasePath` is trusted app metadata, never a prefix inferred from request headers.

## Lifecycle remains the server's

The extension is an ordinary GET listener inside one existing lifecycle. Its observable table
is covered by [native lifecycle tests](../tests/lifecycle.rs).

| Condition | Page behavior | Lifecycle consequence |
| --- | --- | --- |
| Request stage Abort/error | Loader and renderer do not run. | Request Abort skips later stages; error is offered to `error`. |
| Earlier listener finalized output | Page listener skips loading/rendering. | Existing delivery state is preserved. |
| Loader returns `Abort` | Renderer does not run; current response remains. | Operation Abort skips response hooks; finalization still follows host policy. |
| Loader returns error | Renderer does not run. | Operation failure is offered to `error`. |
| Loader returns `Ready` after stop/send | Renderer does not run. | Existing Stopped/Sent state is preserved; ordinary host handling continues. |
| Renderer returns error | No partial page is staged. | Operation failure is offered to `error`. |
| Render succeeds | HTML and public status are staged. | Response hooks run unless another normal lifecycle rule stops them. |
| Response hook changes results/data | HTML stays as rendered. | No implicit rerender. A hook may deliberately replace output. |
| Response hook Abort/error | No rerender or retry. | Response Abort/failure uses the existing server policy. |
| Error hook aborts/fails | Original failure remains recorded. | No recursive error offer; host fallback/finalization rules apply. |

Page identity duplicates fail startup even across plugins. Duplicate native route listeners
remain allowed. After registration errors, discard the builder: queuing page claims and routes
is not a transaction. See [server lifecycle and recovery](../../../server/docs/guides/errors-and-recovery.md).

You now have the boundaries needed to choose a public projection, retain one lifecycle, and
share a compiled route. The [API index](api/README.md) provides the exact types and limits.
