// Opt-in finite packaged-WKWebView launch proof. No server or extra IPC command.
(async () => {
  const deadline = performance.now() + 15000;
  const until = async predicate => {
    while (!predicate()) {
      if (performance.now() > deadline) throw Error('proof deadline');
      await new Promise(resolve => setTimeout(resolve, 20));
    }
  };
  try {
    await until(() => window.__TAURI__?.core && window.profileUiReady);
    if (window.profileProofWindow === 'ungranted') {
      let denied = false;
      try { await window.__TAURI__.core.invoke('profile_search', { callId: 'denied', json: '{"revision":1,"id":"abc"}' }); }
      catch (error) { denied = /not allowed/i.test(String(error)); }
      if (!denied) throw Error('ungranted window was admitted');
      window.location.href = 'proof://denied-ok';
      return;
    }
    const input = document.querySelector('input');
    input.value = 'abc';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('form').dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
    await until(() => document.querySelectorAll('li').length === 3);
    if (!document.querySelector('main').textContent.includes('Ada Lovelace')) throw Error('missing generated row');
    if (location.origin !== 'tauri://localhost') throw Error('not packaged origin');
    window.disposeProfileUi();
    window.location.href = 'proof://main-ok';
  } catch (error) {
    window.location.href = 'proof://failed?message=' + encodeURIComponent(String(error));
  }
})();
