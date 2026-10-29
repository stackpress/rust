# Ownership and selected render modes

Components receive explicit values, signals and callbacks. Theme/language/notifications live
inside their Leptos Owner; no global application service or persistence is acquired.

`render::Renderer` stages synchronous component output and disposes its Owner before returning
owned eager Head/HTML. `async_render::AsyncRenderer` instead retains its root through complete
buffered Resource/Suspense and reactive Head assembly; each poll restores the actual caller.
Dropping it cleans loaders and serialization queues cooperatively, with no retry or rollback.

`stream::StreamRenderer` owns one worker thread, current-thread runtime and LocalSet. Constructed
views stay local; only bounded owned byte chunks cross its two-slot channel. First demand starts
production; dropping staging or the body cancels and joins the worker. Explicit completion is
required for successful EOF. Trusted non-yielding work and blocking destructors remain outside
cooperative cancellation guarantees. Caller framing is admitted before publication, including
its reserved late-failure marker. Streaming stays opt-in.

SSR, hydrate and CSR compile separately. Hydration requires the exact same initial component
tree/data and matching artifacts; CSR constructs its tree from explicitly supplied values.
Checked generic replay contains inert encoded values, not executable scripts. Host status,
page seeds, routing catalogs and completion/recovery identities belong to the integrating app.

`navigation::LocalLocation` enforces strict bounded local paths for images; `components::safe_href`
is the broader navigation policy. `ImageSource` additionally checks raster suffix/size/dimensions.
G41 intentionally keeps view's validator/escapers independently owned, with a shared differential
regression table guarding common behavior.
