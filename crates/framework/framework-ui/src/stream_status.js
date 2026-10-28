// External classic script: execute while the document may still be streaming.
// The completion-gated module alone owns replay admission and hydration.
(() => {
  const recovery = document.getElementById('stackpress-stream-recovery');
  if (!recovery || recovery.firstChild?.nodeType !== Node.TEXT_NODE) return;
  const message = recovery.firstChild;
  let unfinished = false;

  /** Release every observer, listener and timer owned by this document. */
  function dispose() {
    observer.disconnect();
    clearTimeout(watchdog);
    document.removeEventListener('readystatechange', inspect);
    document.removeEventListener('DOMContentLoaded', inspect);
    window.removeEventListener('pagehide', dispose);
  }

  /** Announce only fixed safe text; the prechecked reload link stays intact. */
  function show(text) {
    recovery.hidden = false;
    recovery.setAttribute('role', 'alert');
    message.textContent = text;
  }

  /** A late marker wins over completion; absence during parsing proves nothing. */
  function inspect() {
    if (document.getElementById('stackpress-stream-failed')) {
      show("This page didn't finish loading. ");
      dispose();
      return;
    }
    if (document.getElementById('stackpress-stream-complete')) {
      // A slow successful response may arrive after our informational watchdog.
      // Keep the notice until the independent bootstrap has actually started.
      if (unfinished) {
        recovery.setAttribute('role', 'status');
        message.textContent = 'Page received. Starting browser controls. ';
      }
      dispose();
      return;
    }
    if (document.readyState !== 'loading') {
      show("This page didn't finish loading. ");
      dispose();
    }
  }

  // Missing end events on an aborted document cannot leave an endless Loading.
  // This is elapsed-time guidance, never a claim that the server failed.
  const watchdog = setTimeout(() => {
    unfinished = true;
    show("This page hasn't finished loading. ");
  }, 6000);
  const observer = new MutationObserver(inspect);
  observer.observe(document, {childList: true, subtree: true});
  document.addEventListener('readystatechange', inspect);
  document.addEventListener('DOMContentLoaded', inspect);
  window.addEventListener('pagehide', dispose, {once: true});
  inspect();
})();
