// Finite real-Chromium interaction proof using intercepted requests over pipes.
// No listener, dependency install, test framework or external service is used.
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PROFILE_UI_PLAYWRIGHT || 'playwright');
const root = path.resolve(__dirname, '../../..');
const host = process.env.PROFILE_UI_HOST;
if (!host) throw Error('set PROFILE_UI_HOST to the finite web fixture executable');
const assets = path.join(root, 'examples/profile-ui/dist/hydrate');
const render = () => cp.execFileSync(host, [], { encoding: 'utf8', timeout: 10000 });


(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, reducedMotion: 'reduce' });
  const errors = [];
  const calls = [];
  const held = new Map();
  let mode = 'normal';
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {
    if (/hydration.*(mismatch|error)|already been disposed/i.test(message.text())) errors.push(message.text());
  });
  await page.route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin !== 'https://profiles.invalid') return route.abort();
    if (url.pathname === '/ui-api/v1/profile-search') {
      const json = route.request().postData();
      const id = JSON.parse(json).id;
      calls.push(id);
      if (mode === 'transport') return route.abort('failed');
      if (mode === 'host-error') {
        const invalid = JSON.stringify({ revision: 999, id });
        const body = cp.execFileSync(host, ['search', invalid], { encoding: 'utf8', timeout: 10000 });
        return route.fulfill({ status: 400, contentType: 'application/json', body });
      }
      if (mode === 'hold') await new Promise(resolve => held.set(id, resolve));
      const body = cp.execFileSync(host, ['search', json], { encoding: 'utf8', timeout: 10000 });
      // An aborted fetch can already be detached when its held response is released.
      try { await route.fulfill({ contentType: 'application/json', headers: { 'x-proof-request': id }, body }); }
      catch (error) { if (!/already handled|closed|cancel|abort/i.test(String(error))) throw error; }
      return;
    }
    if (url.pathname.startsWith('/assets/')) {
      const file = path.join(assets, url.pathname.slice('/assets/'.length));
      return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
    }
    const html = render().replace('<script type="module"', '<script>window.before=document.querySelector("#app").innerHTML;window.inputBefore=document.querySelector("input");window.inputBefore.focus();</script><script type="module"');
    return route.fulfill({ contentType: 'text/html', body: html });
  });
  const input = page.locator('#profile-id');
  const submit = async id => { await input.fill(id); await input.press('Enter'); };
  const rows = () => page.locator('.sp-list li');
  const release = id => { const resolve = held.get(id); assert.ok(resolve, `missing barrier ${id}`); held.delete(id); resolve(); };
  try {
    await page.goto('https://profiles.invalid/ui/profiles?id=abc');
    await page.waitForFunction(() => window.profileUiReady === true, { timeout: 10000 });
    assert.equal(calls.length, 0);
    assert.equal(await page.evaluate(() => window.before === document.querySelector('#app').innerHTML), true);
    assert.equal(await page.evaluate(() => window.inputBefore === document.activeElement), true);
    assert.equal(await rows().count(), 3);
    await input.press('Tab');
    assert.equal(await page.locator('form button[type=submit]').evaluate(element => element === document.activeElement), true);
    assert.notEqual(await page.locator('form button[type=submit]').evaluate(element => getComputedStyle(element).outlineStyle), 'none');
    await page.keyboard.press('Shift+Tab');
    assert.equal(await input.evaluate(element => element === document.activeElement), true);

    await submit('');
    assert.equal(calls.length, 0);
    assert.equal(await input.getAttribute('aria-invalid'), 'true');
    assert.match(await page.locator('#profile-id-error').textContent(), /Enter a profile ID prefix/);
    assert.equal(await rows().count(), 3);
    await submit('zzz');
    await page.waitForFunction(() => document.querySelectorAll('.sp-list li').length === 0);
    assert.equal(await page.locator('.sp-empty').isVisible(), true);
    assert.equal(await input.getAttribute('aria-invalid'), 'false');

    mode = 'transport';
    await submit('abc123');
    await page.getByRole('button', { name: 'Retry search', exact: true }).waitFor();
    assert.match(await page.locator('.sp-alert').textContent(), /Search unavailable/);
    assert.equal(await page.locator('[data-search-failure]').getAttribute('data-search-failure'), 'transport');
    assert.doesNotMatch(await page.locator('#app').textContent(), /private diagnostics/);
    await page.getByRole('button', { name: 'Dismiss notification' }).click();
    assert.equal(await page.locator('.sp-notice').count(), 0);
    mode = 'host-error';
    await page.getByRole('button', { name: 'Retry search', exact: true }).click();
    await page.waitForFunction(() => document.querySelector('[data-search-failure]')?.dataset.searchFailure === 'host');
    assert.match(await page.locator('.sp-alert').textContent(), /Search rejected: Page version changed. Refresh to continue./);
    mode = 'normal';
    await page.getByRole('button', { name: 'Retry search', exact: true }).click();
    await page.waitForFunction(() => document.querySelector('.sp-list li')?.textContent === 'Ada Lovelace');

    mode = 'hold';
    await submit('abc456');
    await page.waitForFunction(() => document.querySelector('.sp-results').getAttribute('aria-busy') === 'true');
    assert.equal(await rows().textContent(), 'Ada Lovelace');
    assert.equal(await page.locator('.sp-loader').textContent(), 'Searching…');
    await page.getByRole('button', { name: 'Cancel search', exact: true }).click();
    assert.match(await page.locator('.sp-alert').textContent(), /cancelled/);
    release('abc456');
    assert.equal(await page.locator('.sp-results').getAttribute('aria-busy'), 'false');

    // Suppress best-effort abort only for this race, forcing a real late successful A.
    await page.evaluate(() => {
      window.raceReads = [];
      window.originalAbort = AbortController.prototype.abort;
      window.originalResponseText = Response.prototype.text;
      AbortController.prototype.abort = function () {};
      Response.prototype.text = function () {
        const id = this.headers.get('x-proof-request');
        return window.originalResponseText.call(this).then(text => {
          // A timer turn observes completion after the consumer's Promise/microtask chain.
          setTimeout(() => window.raceReads.push(id), 0);
          return text;
        });
      };
    });
    await submit('abc123');
    await page.getByRole('button', { name: 'Cancel search', exact: true }).waitFor();
    await submit('abc456');
    await page.waitForFunction(() => document.querySelector('input').value === 'abc456');
    release('abc456');
    await page.waitForFunction(() => document.querySelector('.sp-list li')?.textContent === 'Grace Hopper');
    release('abc123');
    await page.waitForFunction(() => window.raceReads.includes('abc123'));
    assert.deepEqual(await page.evaluate(() => window.raceReads), ['abc456', 'abc123']);
    assert.equal(await rows().textContent(), 'Grace Hopper');
    await page.evaluate(() => {
      AbortController.prototype.abort = window.originalAbort;
      Response.prototype.text = window.originalResponseText;
    });

    await page.getByRole('button', { name: 'Français', exact: true }).click();
    await page.waitForFunction(() => document.documentElement.lang === 'fr' && document.title === 'Profils');
    assert.equal(await page.getByLabel('Préfixe de l’identifiant').count(), 1);
    assert.equal(await page.locator('meta[name=description]').count(), 1);
    assert.equal(await page.locator('meta[name=description]').getAttribute('content'), 'Rechercher des profils publics');
    assert.equal(await page.locator('style#stackpress-ui').count(), 1);
    await page.getByRole('button', { name: 'Thème sombre', exact: true }).click();
    assert.equal(await page.locator('.sp-root').getAttribute('data-theme'), 'dark');
    await page.getByRole('button', { name: 'Thème clair', exact: true }).click();
    assert.equal(await page.locator('.sp-root').getAttribute('data-theme'), 'light');

    const geometry = [];
    for (const width of [375, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      for (const zoom of [1, 2]) {
        await page.evaluate(zoom => { document.documentElement.style.zoom = zoom; }, zoom);
        assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), true, `horizontal overflow ${width}/${zoom}`);
        geometry.push({ width, cssZoom: zoom, fits: true });
      }
      await page.evaluate(() => { document.documentElement.style.zoom = 1; });
      await page.screenshot({ path: `/private/tmp/stackpress-s5-b-m2-${width}.png`, fullPage: true });
    }
    // Disposal revokes active transport and safely ignores its eventual response.
    await submit('abc789');
    await page.getByRole('button', { name: 'Annuler la recherche', exact: true }).waitFor();
    await page.evaluate(() => window.disposeProfileUi());
    release('abc789');
    assert.equal(await page.locator('#app').innerHTML(), '');
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ initialCalls: 0, hydration: 'same DOM and focus', keyboard: true, invalid: true, empty: true, transportFailure: true, hostErrorAndRetry: true, loadingRetainsRows: true, cancellation: true, newestWinsWithLateSuccess: true, localeAndHead: true, theme: true, geometry, disposed: true, errors }));
  } finally {
    for (const resolve of held.values()) resolve();
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
