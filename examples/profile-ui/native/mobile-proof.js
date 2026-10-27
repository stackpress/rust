// Explicit simulator fixture only. Ordinary startup installs none of this code.
(async () => {
  const errors = [];
  window.addEventListener('error', event => errors.push(String(event.message)));
  window.addEventListener('unhandledrejection', event => errors.push(String(event.reason)));
  let firstRender;
  const observe = new MutationObserver(() => {
    const root = document.querySelector('.sp-root');
    if (!firstRender && root && root.querySelector('h1')) firstRender = { heading: root.querySelector('h1').textContent, theme: root.dataset.theme };
  });
  observe.observe(document, { subtree: true, childList: true });
  const deadline = performance.now() + 60000;
  let stage = 'startup';
  const until = async predicate => {
    while (!predicate()) {
      if (performance.now() > deadline) throw Error('mobile proof timed out at ' + stage);
      await new Promise(resolve => setTimeout(resolve, 50));
    }
  };
  const assert = (value, message) => { if (!value) throw Error(message); };
  const report = phase => {
    const data = { phase, firstRender, lang: document.documentElement.lang, theme: document.querySelector('.sp-root')?.dataset.theme,
      draft: document.querySelector('input')?.value, errors, origin: location.origin };
    window.location.href = 'mobile-proof://report?data=' + encodeURIComponent(JSON.stringify(data));
  };
  const button = label => [...document.querySelectorAll('button')].find(node => node.textContent === label);
  const native = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
  try {
    await until(() => window.profileUiReady && document.querySelectorAll('.sp-list li').length === 3);
    assert(location.origin === 'tauri://localhost', 'unexpected packaged origin');
    if (window.mobileProofMode === 'restart') {
      assert(document.documentElement.lang === 'fr', 'restart lost locale');
      assert(document.querySelector('.sp-root').dataset.theme === 'dark', 'restart lost theme');
      assert(firstRender.heading === 'Profils' && firstRender.theme === 'dark', 'preferences were not applied to first CSR render');
      report('complete');
      return;
    }
    if (window.mobileProofMode === 'recovery') {
      assert(document.documentElement.lang === 'en', 'corrupt file did not select explicit defaults');
      assert(document.querySelector('.sp-root').dataset.theme === 'light', 'corrupt file theme fallback drift');
      assert(document.querySelector('#app').textContent.includes('Saved preferences could not be loaded'), 'recovery warning missing');
      report('complete');
      return;
    }
    for (const command of ['shell', 'emit']) {
      let denied = false;
      try { await native(command); } catch { denied = true; }
      assert(denied, 'forged generic command admitted');
    }
    for (const [command, args] of [
      ['profile_search', { callId: 'forged', json: '{"revision":1,"id":"abc","event":"shell"}' }],
      ['update_preferences', { json: '{"revision":1,"preferences":{"locale":"fr","theme":"dark"},"path":"/outside"}' }]
    ]) {
      let denied = false;
      try { await native(command, args); } catch { denied = true; }
      assert(denied, 'forged fields admitted');
    }
    const input = document.querySelector('input');
    input.value = 'UNSAVED-before-failure';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    stage = 'failure-barrier';
    report('failure-ready');
    await until(() => window.mobileProofControl === 'fail');
    button('Français').click();
    await until(() => document.querySelector('#app').textContent.includes('Preferences could not be saved'));
    assert(document.documentElement.lang === 'en' && input.value === 'UNSAVED-before-failure', 'failed persistence replaced UI or draft');
    assert(document.querySelector('.sp-root').dataset.theme === 'light', 'failed persistence changed theme');
    report('failed-save-kept-draft');
    await until(() => window.mobileProofControl === 'continue');
    stage = 'save-preferences';
    button('Français').click();
    await until(() => document.documentElement.lang === 'fr');
    button('Thème sombre').click();
    await until(() => document.querySelector('.sp-root').dataset.theme === 'dark');
    assert(input.value === 'UNSAVED-before-failure', 'successful preferences lost draft');
    input.value = 'abc456';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('form').dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
    stage = 'permitted-search';
    await until(() => document.querySelectorAll('.sp-list li').length === 1 && document.querySelector('.sp-list li').textContent === 'Grace Hopper');
    document.querySelector('.sp-list a').click();
    stage = 'detail';
    await until(() => document.querySelector('[data-detail-page] h2')?.textContent === 'Grace Hopper');
    history.back();
    await until(() => !document.querySelector('[data-detail-page]'));
    input.value = 'UNSAVED-mobile-background';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    window.mobileDraftRef = input;
    report('background-ready');
    stage = 'background-resume';
    await until(() => window.mobileProofControl === 'finish');
    assert(document.querySelector('input') === window.mobileDraftRef && input.value === 'UNSAVED-mobile-background', 'background/resume replaced root or draft');
    assert(document.documentElement.lang === 'fr' && document.querySelector('.sp-root').dataset.theme === 'dark', 'background/resume changed preferences');
    assert(errors.length === 0, 'unexpected JavaScript error');
    report('complete');
  } catch (error) {
    errors.push(String(error));
    report('failed');
  }
})();
