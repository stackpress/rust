# Values, roots and rendering

Two tabs can show different languages and themes without sharing mutable state. Give each
component root explicit initial values, then allocate its signals inside its Leptos Owner.
`ThemeState`, `LanguageContext` and `Notifications` describe presentation in that one root.
They do not read an OS preference, cookie, storage service or application identity.

## A root owns its work

A signal handle belongs to the Owner that created it. Provide root state before evaluating
children, and marshal asynchronous completions back to that root before publishing changes.
Disposal removes root-owned reactive work; it is separate from rolling back external effects.

For example, the independent consumer starts one root with Light/English and another with
Dark/French. Its counter, draft, popover and notifications are recreated for each root. The
caller chooses those values; an application's saved/unset preference policy belongs outside
the generic library. Effective UI theme has only Light and Dark.

## Choose the lifetime you need

The kernels differ in how long they retain the reactive tree, rather than in transport.

| Kernel | Root lifetime | Result |
| --- | --- | --- |
| `Renderer` | Component evaluation and synchronous staging | Owned eager Head/body; `PreparedRender::finish` checks complete output |
| `AsyncRenderer` | Through body, Resource replay and reactive Head completion | Owned `Send` future; dropping it cleans the retained root |
| `StreamRenderer` | One worker/runtime/LocalSet until completion or cancellation | `Send` owned bytes; local view state stays on the worker |

The async kernel restores the polling caller's Owner on every poll. Native retained state
must permit the future's thread migration; local `Rc`/LocalStorage state belongs in synchronous
rendering, a browser, or the stream worker. Cancellation is cooperative: trusted non-yielding
work and blocking destructors cannot be forcibly preempted.

Streaming starts production on first body demand and uses a bounded two-slot bridge. Framing,
escaped replay, trailer and a reserved failure marker all count toward output limits. Successful
EOF requires the explicit completion message. The integrating application defines what those
bytes mean; the generic kernel has no page status or recovery-element identities.

## Keep initial trees deterministic

Hydration adopts matching native markup using the same component, values and selected helper
features. Capture any allowed native draft before hydration can reconcile properties. CSR starts
with an empty mount and constructs a new tree. Compile these modes as separate WASM artifacts.

A serializable Resource can reuse checked inert replay from native rendering. Replay contains
encoded values for upstream Leptos codecs, not executable JavaScript. The consumer owns its
bootstrap and root disposal. See [hydrate and mount](guides/hydrate-and-mount.md).

## Checked values have distinct policies

`LocalLocation` accepts a bounded encoded local path/query without normalization. `ImageSource`
uses that strict policy and additionally checks raster suffix, declared bytes and dimensions.
`safe_href` supports broader ordinary navigation. Replacing the image check with it would widen
image admission. HTML/script escaping and rich sanitization also have different inputs:
escaping protects text/JSON insertion, while `SafeHtml` produces checked markup under a closed policy.
