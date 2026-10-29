# StreamRenderer and owned bodies

Feature `stream` implies native `ssr`; it does not enable itself through preferences or page
registration. Start explicitly with `render::initialize_ssr` in Tokio. A complete consumer is
in [stream HTML](../guides/stream-html.md).

## Start a worker

`StreamRenderer::render(limits, policy, failure, work).await` creates one supervised worker.
`work: FnOnce(StreamWriter<E>) -> Fut + Send + 'static` constructs its view inside that worker;
`Fut` may remain local and returns `Result<(), RenderError<E>>`. `E` must be `Send + 'static`.

| Parameter | Contract |
| --- | --- |
| `limits: RenderLimits` | Aggregate HTML and logical replay ceilings |
| `policy: StreamLimits` | Nonzero fragment bound and absolute render timeout |
| `failure: String` | Trusted late-failure bytes; must fit the fragment bound; reserved in aggregate budget |
| `work` | Trusted yielding factory; no automatic application protocol |

**Returns** `Result<StreamBody<E>, RenderError<E>>` after opening/Head staging. `StreamBody`
implements `Stream<Item=Result<Vec<u8>,RenderError<E>>>` and is `Send`. Poll through explicit
completion; dropping staging or the body cancels and joins the worker. No worker is detached.

## Produce framed chunks

`writer.render(body, head, opening, frame, trailer).await` consumes a `Stream<Item=String>` and
metadata output. It stages opening/Head, waits for demand, reserves bridge capacity before
polling local body work, frames each body/checked-replay pair, and finishes with trailer/completion.
`frame: FnMut(String,String) -> Result<String,E>` supplies application framing.

**Returns** Unit after explicit completion, or a generic execution/caller failure. Every affected
fragment is admitted before publication. After commitment, failure bytes and a typed failure are
sent under the reserved budget; arbitrary errors are not written into HTML.

`StreamLimits::default()` uses `max_chunk_bytes=64*1024` and `render_timeout=30s`. Zero, an
unrepresentable deadline, a fragment larger than total HTML, or a failure marker larger than a
fragment is invalid. Trusted non-yielding CPU work or a blocking destructor cannot be forcibly
interrupted. The transport needs its own admission, connection and flush/delivery semantics.
