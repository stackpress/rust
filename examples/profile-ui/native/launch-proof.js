// Opt-in finite packaged-WKWebView launch proof. No server or extra IPC command.
(async () => {
  let stage = "startup";
  // Fail in JS before the native observer watchdog so its receipt retains the failing stage.
  const deadline = performance.now() + 18000;
  const named = name => window.profileProofCustomization ? 'Demo: ' + name : name;
  const errors = [];
  window.addEventListener('error', event => errors.push(String(event.message)));
  window.addEventListener('unhandledrejection', event => errors.push(String(event.reason)));
  const until = async predicate => {
    while (!predicate()) {
      if (performance.now() > deadline) throw Error('proof deadline at ' + stage + ' path=' + location.pathname + location.search + ' rows=' + Array.from(document.querySelectorAll('.sp-list li')).map(row => row.textContent).join(',') + ' detail=' + document.querySelector('[data-detail-page]')?.getAttribute('aria-busy') + ' entry=' + window.profileHangEntered + ' alert=' + document.querySelector('.sp-alert')?.textContent + ' input=' + document.querySelector('input')?.value + ' lang=' + document.documentElement.lang + ' errors=' + errors.join('|'));
      // WKWebView may throttle background timers. Observe actual UI/native progress instead.
      await new Promise(resolve => {
        const events = ['securitypolicyviolation', 'profile-proof-progress', 'popstate', 'error', 'unhandledrejection'];
        const finish = () => {
          observer.disconnect();
          events.forEach(event => window.removeEventListener(event, finish));
          clearTimeout(timer);
          resolve();
        };
        const observer = new MutationObserver(finish);
        const timer = setTimeout(finish, Math.max(1, deadline - performance.now()));
        observer.observe(document, { subtree: true, childList: true, attributes: true, characterData: true });
        events.forEach(event => window.addEventListener(event, finish));
      });
    }
  };
  // Account for the local WASM bootstrap separately; every data-path fetch must be IPC.
  const fetches = { packagedAssets: 0, ipc: 0, nonIpc: 0 };
  const nativeFetch = window.fetch.bind(window);
  window.fetch = (input, ...options) => {
    const url = new URL(input instanceof Request ? input.url : String(input), location.href);
    if (url.protocol === 'tauri:' && url.hostname === 'localhost' && url.pathname.endsWith('/profile_ui_bg.wasm')) fetches.packagedAssets++;
    else if (url.protocol === 'ipc:' || url.origin === 'http://ipc.localhost') fetches.ipc++;
    else fetches.nonIpc++;
    return nativeFetch(input, ...options);
  };
  const blocked = new Set();
  document.addEventListener('securitypolicyviolation', event => {
    if (event.blockedURI.includes('blocked.invalid')) blocked.add(event.effectiveDirective);
  });
  const audit = () => {
    if (errors.length) throw Error('packaged JavaScript error: ' + errors.join('|'));
    if (fetches.nonIpc !== 0) throw Error('unexpected non-IPC fetch');
    if (!blocked.has('img-src') || !blocked.has('connect-src')) throw Error('external CSP probe was not blocked');
    return 'non_ipc_fetches=0&packaged_asset_fetches=' + fetches.packagedAssets + '&csp_image=blocked&csp_connect=blocked';
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
    // These deliberate resource attempts are blocked by CSP before network acquisition.
    const image = document.createElement('img');
    image.src = 'https://blocked.invalid/proof.png';
    document.body.append(image);
    try { const socket = new WebSocket('wss://blocked.invalid/proof'); socket.onerror = () => {}; } catch (_) {}
    await until(() => blocked.has('img-src') && blocked.has('connect-src'));
    image.remove();
    const input = document.querySelector('input');
    const submit = id => {
      input.value = id;
      input.dispatchEvent(new Event('input', { bubbles: true }));
      const event = new Event('submit', { bubbles: true, cancelable: true });
      document.querySelector('form').dispatchEvent(event);
      if (!event.defaultPrevented || !input.isConnected) throw Error('search handler detached at ' + stage);
    };
    const clickLabel = label => {
      const button = Array.from(document.querySelectorAll('button')).find(button => button.textContent === label);
      if (!button) throw Error('missing control ' + label);
      button.click();
    };
    // Packaged CSR obtains its first rows automatically over the permitted local adapter.
    await until(() => document.querySelectorAll('.sp-list li').length === 3);
    const expectedNames = ['Ada Lovelace', 'Grace Hopper', 'Margaret Hamilton'].map(named);
    const reply = JSON.parse(await window.__TAURI__.core.invoke('profile_search', { callId: 'customization-parity', json: '{"revision":1,"id":"abc"}' }));
    if (JSON.stringify(reply.rows.map(row => row.name)) !== JSON.stringify(expectedNames)) throw Error('native customization names disagree');
    if (JSON.stringify(Array.from(document.querySelectorAll('.sp-list li'), row => row.textContent)) !== JSON.stringify(expectedNames)) throw Error('shared UI and IPC names disagree');
    if (window.profileProofMode !== 'navigation') {
      window.location.href = 'proof://audit?' + audit();
      await until(() => window.profileAuditAccepted === true);
      submit(window.profileProofMode);
      return; // Native entry barrier triggers the actual close/quit event.
    }
    // Unknown commands and malformed/forged payloads are rejected before domain admission.
    for (const command of ['emit', 'shell']) {
      let denied = false;
      try { await window.__TAURI__.core.invoke(command, {}); } catch (_) { denied = true; }
      if (!denied) throw Error('unregistered command admitted');
    }
    for (const json of ['{', '{"revision":1,"id":"abc","event":"shell"}', 'x'.repeat(65537)]) {
      let denied = false;
      try { await window.__TAURI__.core.invoke('profile_search', { callId: 'bad', json }); }
      catch (error) { denied = String(error) === 'denied'; }
      if (!denied) throw Error('malformed native input admitted');
    }

    submit('');
    await until(() => input.getAttribute('aria-invalid') === 'true');
    if (!document.querySelector('#profile-id-error').textContent) throw Error('missing associated validation');
    submit('zzz');
    await until(() => document.querySelectorAll('.sp-list li').length === 0 && !document.querySelector('.sp-empty').hidden);
    submit('abc456');
    await until(() => document.querySelector('.sp-list li')?.textContent === named('Grace Hopper'));
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
    const grace = () => Array.from(document.querySelectorAll('.sp-list a')).find(link => link.textContent === named('Grace Hopper'));
    stage = "detail";
    grace().click();
    await until(() => document.querySelector('[data-detail-page] h2')?.textContent === named('Grace Hopper') && document.querySelector('[data-detail-page]')?.getAttribute('aria-busy') === 'false');
    if (!location.href.includes('return=abc') || document.title !== 'Profile') throw Error('detail URL/head disagreed');
    stage = "back";
    history.back();
    await until(() => !document.querySelector('[data-detail-page]') && document.querySelectorAll('.sp-list li').length === 3);
    if (input.value !== 'abc') throw Error('back lost prefix');
    stage = "forward";
    history.forward();
    await until(() => document.querySelector('[data-detail-page] h2')?.textContent === named('Grace Hopper'));
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
    await until(() => !document.querySelector('[data-detail-page]') && document.querySelector('.sp-list li')?.textContent === named('Grace Hopper'));
    if (input.value !== 'abc456') throw Error('return inferred detail ID');
    submit('abc');
    await until(() => document.querySelectorAll('.sp-list li').length === 3);
    // Native monitor acknowledges real suspended-handler entry; no sleep guesses admission.
    stage = 'deadline';
    submit('deadline');
    await until(() => window.profileHangEntered === 1);
    await until(() => document.querySelector('.sp-alert').textContent.includes('Search timed out.'));
    stage = 'native-cancel';
    submit('hang');
    await until(() => window.profileHangEntered === 2);
    clickLabel('Cancel search');
    await until(() => document.querySelector('.sp-alert').textContent.includes('Search cancelled.'));
    // Require a different published row; retained rows cannot satisfy this recovery barrier.
    submit('abc123');
    await until(() => document.querySelectorAll('.sp-list li').length === 1 && document.querySelector('.sp-list li').textContent === named('Ada Lovelace') && location.search === '?id=abc123');
    stage = 'locale';
    submit('abc456');
    await until(() => document.querySelectorAll('.sp-list li').length === 1 && document.querySelector('.sp-list li').textContent === named('Grace Hopper') && location.search === '?id=abc456');
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
    window.location.href = 'proof://main-ok?controls=bootstrap-invalid-empty-search-native-host-error-retry-denied-transport-detail-history-return-missing-native-deadline-native-cancel-locale-theme-disposal&' + audit();
  } catch (error) {
    window.location.href = 'proof://failed?message=' + encodeURIComponent(String(error));
  }
})();
