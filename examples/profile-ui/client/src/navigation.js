// DOM/history transport only. Rust owns catalog matching, state and admission.
let displayedPath;

export function installNavigation(receive) {
  displayedPath = location.pathname;
  const pop = () => receive(location.pathname + location.search, false);
  const click = event => {
    if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.defaultPrevented) return;
    const link = event.target.closest?.('#app a[href]');
    if (!link || link.target || link.hasAttribute('download')) return;
    const target = new URL(link.href);
    if (target.origin !== location.origin || target.hash) return;
    event.preventDefault();
    receive(target.pathname + target.search, true);
  };
  addEventListener('popstate', pop);
  document.addEventListener('click', click);
  return () => {
    removeEventListener('popstate', pop);
    document.removeEventListener('click', click);
  };
}

export function writeNavigation(location, push) {
  const nextPath = new URL(location, window.location.href).pathname;
  const changedPage = displayedPath !== nextPath;
  displayedPath = nextPath;
  if (push) history.pushState(null, '', location);
  else history.replaceState(null, '', location);
  // Canonical metadata follows the same checked URL as history.
  document.querySelector('link[rel=canonical]')?.setAttribute('href', location);
  // Focus remains in the new page's visible document order after keyboard navigation.
  if (changedPage) requestAnimationFrame(() => {
    const heading = document.querySelector('#app h1');
    if (heading) {
      heading.tabIndex = -1;
      heading.focus({ preventScroll: true });
    }
  });
}
