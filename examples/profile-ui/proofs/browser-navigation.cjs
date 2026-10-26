// Finite real-Chromium navigation and initial-settings proof; no listening socket.
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PROFILE_UI_PLAYWRIGHT || 'playwright');
const host = process.env.PROFILE_UI_HOST;
const base = process.env.PROFILE_UI_BASE || '';
const assets = path.resolve(__dirname, '../dist/hydrate');
if (!host) throw Error('set PROFILE_UI_HOST');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  const errors = [];
  let calls = 0;
  let display = 'en-light';
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error' || /hydration.*(mismatch|error)/i.test(message.text())) errors.push(message.text()); });
  await page.route('**/*', async route => {
    const request = route.request();
    const url = new URL(request.url());
    if (url.origin !== 'https://profiles.invalid') return route.abort();
    if (url.pathname.startsWith(`${base}/assets/`)) {
      const file = path.join(assets, url.pathname.slice(`${base}/assets/`.length));
      return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
    }
    const env = { ...process.env, PROFILE_UI_DISPLAY: display };
    if (url.pathname === `${base}/ui-api/v1/profile-search`) {
      calls++;
      return route.fulfill({ contentType: 'application/json', body: cp.execFileSync(host, ['search', request.postData()], { env, encoding: 'utf8', timeout: 10000 }) });
    }
    let html = cp.execFileSync(host, [url.pathname + url.search], { env, encoding: 'utf8', timeout: 10000 });
    html = html.replace('<script type="module"', '<script>window.before=document.querySelector("#app").innerHTML;window.initialLang=document.documentElement.lang;window.initialTheme=document.querySelector(".sp-root").dataset.theme;window.initialTitle=document.title;</script><script type="module"');
    return route.fulfill({ contentType: 'text/html', body: html });
  });
  const ready = () => page.waitForFunction(() => window.profileUiReady === true);
  const visibleRows = () => page.locator('.sp-list li:visible');
  const detail = () => page.locator('[data-detail-page]');
  try {
    await page.goto(`https://profiles.invalid${base}/ui/profiles?id=abc`);
    await ready();
    assert.equal(calls, 0);
    // Reuse Grace's stable row key across two successful prefixes before navigating.
    for (const prefix of ['abc456', 'abc']) {
      await page.locator('input').fill(prefix);
      await page.locator('form button[type=submit]').click();
      await page.waitForFunction(prefix => new URL(location.href).searchParams.get('id') === prefix && document.querySelector('.sp-results').getAttribute('aria-busy') === 'false', prefix);
    }
    const href = `${base}/ui/profiles/abc456?return=abc`;
    assert.equal(await page.getByRole('link', { name: 'Grace Hopper', exact: true }).getAttribute('href'), href);
    // Native anchor keyboard activation travels through the shared catalog adapter.
    await page.getByRole('link', { name: 'Grace Hopper', exact: true }).focus();
    await page.keyboard.press('Enter');
    await detail().getByRole('heading', { name: 'Grace Hopper', exact: true }).waitFor();
    assert.equal(new URL(page.url()).pathname + new URL(page.url()).search, href);
    await page.waitForFunction(() => document.activeElement === document.querySelector('#app h1'));
    assert.equal(await page.title(), 'Profile');
    assert.equal(await page.locator('meta[name=description]').getAttribute('content'), 'Public profile detail');
    await page.goBack();
    await visibleRows().filter({ hasText: 'Ada Lovelace' }).waitFor();
    assert.equal(await visibleRows().count(), 3);
    assert.equal(await page.locator('input').inputValue(), 'abc');
    const beforeHistory = calls;
    await page.goForward();
    await detail().getByRole('heading', { name: 'Grace Hopper', exact: true }).waitFor();
    assert.equal(calls, beforeHistory, 'settled history restores rows without a read');
    await page.reload();
    await ready();
    await detail().getByRole('heading', { name: 'Grace Hopper', exact: true }).waitFor();
    assert.equal(calls, beforeHistory, 'detail hydration consumes native seed');
    assert.equal(await page.evaluate(() => window.before === document.querySelector('#app').innerHTML), true);
    await detail().getByRole('link', { name: 'Return to search' }).click();
    await visibleRows().filter({ hasText: 'Ada Lovelace' }).waitFor();
    assert.equal(await visibleRows().count(), 3);
    assert.equal(await page.locator('input').inputValue(), 'abc');
    await page.reload();
    await ready();
    assert.equal(await visibleRows().count(), 3);
    await page.goto(`https://profiles.invalid${base}/ui/profiles/missing?return=abc456`);
    await ready();
    await detail().getByRole('heading', { name: 'Profile not found' }).waitFor();
    const snapshot = await page.locator('#stackpress-page').textContent();
    assert.equal(JSON.parse(snapshot).status.code, 404);
    assert.equal(await detail().getByRole('link', { name: 'Return to search' }).getAttribute('href'), `${base}/ui/profiles?id=abc456`);
    await detail().getByRole('link', { name: 'Return to search' }).click();
    await visibleRows().filter({ hasText: 'Grace Hopper' }).waitFor();
    assert.equal(await visibleRows().count(), 1);
    assert.equal(await page.locator('input').inputValue(), 'abc456');
    display = 'fr-dark';
    const beforeFrench = calls;
    await page.goto(`https://profiles.invalid${base}/ui/profiles?id=abc`);
    await ready();
    assert.equal(calls, beforeFrench);
    assert.deepEqual(await page.evaluate(() => ({ same: window.before === document.querySelector('#app').innerHTML, before: [window.initialLang, window.initialTheme, window.initialTitle], after: [document.documentElement.lang, document.querySelector('.sp-root').dataset.theme, document.title] })), { same: true, before: ['fr', 'dark', 'Profils'], after: ['fr', 'dark', 'Profils'] });
    await page.getByRole('link', { name: 'Grace Hopper', exact: true }).click();
    await detail().getByRole('heading', { name: 'Grace Hopper', exact: true }).waitFor();
    assert.equal(await page.title(), 'Profil');
    assert.equal(await page.locator('.sp-root').getAttribute('data-theme'), 'dark');
    await detail().getByRole('link', { name: 'Retour à la recherche' }).click();
    await visibleRows().filter({ hasText: 'Ada Lovelace' }).waitFor();
    assert.equal(await page.title(), 'Profils');
    for (const selector of ['title', 'meta[name=description]', 'style#stackpress-ui', 'link[rel=canonical]']) assert.equal(await page.locator(selector).count(), 1, selector);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ base, detail: true, keyboard: true, history: true, refresh: true, returnPrefixAndRows: true, native404: true, frenchDarkEquality: true, navigationHeadAndTheme: true, errors }));
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
