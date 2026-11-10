// A deferred CSR build must not delay ordinary hydration pages or reload them twice.
// The URL is authoritative before the browser entry has populated data-blog-ready.
const blogRenderer = new URL(window.location.href).searchParams.get("renderer") === "csr" ? "csr" : "hydrate";
if (msg.all || msg.blog_reload === blogRenderer) window.location.reload();
