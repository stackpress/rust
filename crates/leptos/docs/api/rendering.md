# Native rendering kernels

Import `stackpress_leptos::render` with native `ssr`. Start explicitly with `initialize_ssr`
inside Tokio. See [a complete program](../getting-started.md) and [buffered rendering](../guides/render-native.md).

## Synchronous staging

`Renderer::try_prepare(limits, component, shell) -> Result<PreparedRender<E>, RenderError<E>>`
evaluates `component: FnOnce() -> Result<V,E>` in a fresh Owner, converts `V: IntoView` to HTML,
and calls `shell: FnOnce(String) -> Result<String,E>`. The Owner is cleaned before it returns.

| Input | Contract |
| --- | --- |
| `limits` | `RenderLimits`; checked before component side effects |
| `component` | Trusted synchronous factory; may fail with caller-owned `E` |
| `shell` | Trusted framing around generated component HTML; escape its untrusted interpolations |

**Returns** An owned staged result. `PreparedRender::finish(self).await` consumes eager owned
Head/body, checks executor availability and complete framed byte size, and returns `String`.
A reactive Head getter that reads disposed state is unsupported in this kernel.

## Buffered asynchronous rendering

`AsyncRenderer::try_prepare(limits, component, after_body, shell)` returns
`Result<OwnedRender<E>, RenderError<E>>`, with `E: Send + 'static`. The component is constructed
synchronously; `after_body: FnOnce() -> Result<(),E>` runs after stream creation inside its Owner.
The final `Send + 'static` shell receives `(body: String, replay: String)` after consumption.

**Returns** A concrete owned `Send` future producing `Result<String, RenderError<E>>`. Polling
restores the caller's Owner and dropping releases root state/queues. Retained native reactive
state must be safe for migration; do not retain local-storage signals across a migrating future.
Use the stream worker for local native views.

## Limits and failures

`RenderLimits` defaults to `html_bytes=2*1024*1024` and `replay_bytes=1024*1024`. Callers may lower
those ceilings, never raise them. Replay admission counts logical serialized bytes and separately
checks expanded script-safe bytes. Complete HTML includes caller framing and Head. Zero HTML
bytes admits only empty output, not an unlimited result.

| `RenderError<E>` | Meaning |
| --- | --- |
| `ExecutorUnavailable` | Startup or current native Tokio runtime is absent |
| `InvalidPolicy` | A ceiling/policy exceeds the supported bound |
| `HtmlLimit` | Complete output or an admitted fragment exceeds the selected ceiling |
| `Replay(ReplayError)` | Generic replay admission/serialization failed |
| `Component(E)` | Original caller factory/framing error |
| `Failure(Arc<dyn Error + Send + Sync>)` | Native diagnostic; inspect `Error::source` explicitly |

Trusted factory panic propagates after cleanup. Buffered assembly/Head unwind becomes a consumed
failure and is never resumed. `Display` is safe fixed text; it does not serialize native causes.
[Stream rendering](streaming.md) adds explicit worker demand, deadlines and late failure framing.
