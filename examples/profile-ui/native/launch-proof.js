// Opt-in finite packaged-WKWebView launch proof. No server or extra IPC command.
(async () => {
  let stage = "startup";
  const deadline = performance.now() + 15000;
  const until = async predicate => {
    while (!predicate()) {
      if (performance.now() > deadline) throw Error('proof deadline at ' + stage + ' path=' + location.pathname + location.search + ' rows=' + Array.from(document.querySelectorAll('.sp-list li')).map(row => row.textContent).join(',') + ' detail=' + document.querySelector('[data-detail-page]')?.getAttribute('aria-busy'));
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
      await until(() => document.querySelector('[data-search-failure]')?.dataset.searchFailure === 'transport');
      window.location.href = 'proof://denied-ok';
      return;
    }
    const input = document.querySelector('input');
    const submit = id => {
      input.value = id;
      input.dispatchEvent(new Event('input', { bubbles: true }));
      document.querySelector('form').dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
    };
    const clickLabel = label => {
      const button = Array.from(document.querySelectorAll('button')).find(button => button.textContent === label);
      if (!button) throw Error('missing control ' + label);
      button.click();
    };
    // Packaged CSR obtains its first rows automatically over the permitted local adapter.
    await until(() => document.querySelectorAll('.sp-list li').length === 3);
    if (!document.querySelector('main').textContent.includes('Ada Lovelace')) throw Error('missing generated row');
    submit('');
    await until(() => input.getAttribute('aria-invalid') === 'true');
    if (!document.querySelector('#profile-id-error').textContent) throw Error('missing associated validation');
    submit('zzz');
    await until(() => document.querySelectorAll('.sp-list li').length === 0 && !document.querySelector('.sp-empty').hidden);
    submit('abc456');
    await until(() => document.querySelector('.sp-list li')?.textContent === 'Grace Hopper');
    if (document.querySelector('p[role=status]').textContent !== '1 profile') throw Error('English singular lost');
    // A proof-only selected native listener rejects through the real lifecycle and IPC.
    submit('host-error');
    await until(() => document.querySelector('[data-search-failure]')?.dataset.searchFailure === 'host');
    if (!document.querySelector('.sp-alert').textContent.includes('Choose another prefix.')) throw Error('host message lost');
    if (document.querySelector('#app').textContent.includes('PRIVATE_PROOF_CAUSE')) throw Error('native cause leaked');
    input.value = 'abc';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    clickLabel('Retry search');
    await until(() => document.querySelectorAll('.sp-list li').length === 3 && document.querySelector('[data-search-failure]')?.dataset.searchFailure === undefined);
    // Same catalog and retained state in packaged WKWebView, including history traversal.
    const grace = () => Array.from(document.querySelectorAll('.sp-list a')).find(link => link.textContent === 'Grace Hopper');
    stage = "detail";
    grace().click();
    await until(() => document.querySelector('[data-detail-page] h2')?.textContent === 'Grace Hopper' && document.querySelector('[data-detail-page]')?.getAttribute('aria-busy') === 'false');
    if (!location.href.includes('return=abc') || document.title !== 'Profile') throw Error('detail URL/head disagreed');
    stage = "back";
    history.back();
    await until(() => !document.querySelector('[data-detail-page]') && document.querySelectorAll('.sp-list li').length === 3);
    if (input.value !== 'abc') throw Error('back lost prefix');
    stage = "forward";
    history.forward();
    await until(() => document.querySelector('[data-detail-page] h2')?.textContent === 'Grace Hopper');
    stage += '-return';
    document.querySelector('[data-detail-page] a').click();
    await until(() => !document.querySelector('[data-detail-page]') && document.querySelectorAll('.sp-list li').length === 3);
    const missing = document.createElement('a');
    missing.href = '/ui/profiles/missing?return=abc456';
    document.querySelector('#app').append(missing);
    stage = "missing";
    missing.click();
    missing.remove();
    await until(() => document.querySelector('[data-detail-page] h2')?.textContent === 'Profile not found' && document.querySelector('[data-detail-page]')?.getAttribute('aria-busy') === 'false');
    stage += '-return';
    document.querySelector('[data-detail-page] a').click();
    await until(() => !document.querySelector('[data-detail-page]') && document.querySelector('.sp-list li')?.textContent === 'Grace Hopper');
    if (input.value !== 'abc456') throw Error('return inferred detail ID');
    submit('abc');
    await until(() => document.querySelectorAll('.sp-list li').length === 3);
    stage = 'locale';
    submit('abc456');
    await until(() => document.querySelectorAll('.sp-list li').length === 1);
    clickLabel('Français');
    await until(() => document.documentElement.lang === 'fr' && document.title === 'Profils');
    if (document.querySelector('p[role=status]').textContent !== '1 profil' || document.querySelector('[aria-label=\"About these profiles\"] p').textContent !== '1 profil public affiché.') throw Error('French singular lost');
    clickLabel('Thème sombre');
    await until(() => document.querySelector('.sp-root').dataset.theme === 'dark');
    clickLabel('Thème clair');
    submit('abc');
    await until(() => document.querySelectorAll('.sp-list li').length === 3);
    if (location.origin !== 'tauri://localhost') throw Error('not packaged origin');
    window.disposeProfileUi();
    window.location.href = 'proof://main-ok?controls=bootstrap-invalid-empty-search-native-host-error-retry-denied-transport-detail-history-return-missing-locale-theme-disposal';
  } catch (error) {
    window.location.href = 'proof://failed?message=' + encodeURIComponent(String(error));
  }
})();
