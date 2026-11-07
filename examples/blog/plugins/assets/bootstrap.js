// One root owns its browser mount and disposer. No watcher or development listener starts.
const mode = new URL(location.href).searchParams.get('renderer') === 'csr' ? 'csr' : 'hydrate';
const { default: init, start } = await import(`/assets/${mode}/blog.js`);
await init();
const dispose = start();
window.__blogDispose = dispose;
window.addEventListener('pagehide', () => dispose(), { once: true });
document.documentElement.dataset.blogReady = mode;
