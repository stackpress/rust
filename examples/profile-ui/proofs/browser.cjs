// Finite real-Chromium hydration proof. All requests are fulfilled over browser pipes.
// No listener, test framework, package install or external network service is used.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PROFILE_UI_PLAYWRIGHT || 'playwright');
const root = path.resolve(__dirname, '../../..');
const host = process.env.PROFILE_UI_HOST;
if (!host) throw Error('set PROFILE_UI_HOST to the built finite web fixture binary');
const assets = path.join(root, 'examples/profile-ui/dist/hydrate');
const render = (hostile) => withM1Assets(cp.execFileSync(host, hostile ? ["hostile"] : [], { encoding: 'utf8', timeout: 10000 }));
/** Temporary M1 bootstrap fixture; remove when the merged host emits its own assets. */
function withM1Assets(html) {
  if (html.includes('id="stackpress-assets"')) return html;
  const data = JSON.parse(html.match(/id="stackpress-page">(.*?)<\/script>/s)[1]);
  const assets = JSON.stringify({ build: data.build, base: "", module: "/assets/start.js", stylesheet: null });
  return html.replace('</head>', '<script type="application/json" id="stackpress-assets">' + assets + '</script></head>');
}

(async () => {
  const browser = await chromium.launch({ headless: true });
  const errors = [];
  let calls = 0;
  const page = await browser.newPage();
  page.on('pageerror', e => errors.push(e.message));
  page.on('console', msg => { if (msg.type() === 'error' || /hydration.*(mismatch|error)/i.test(msg.text())) errors.push(msg.text()); });
  let mismatch = false;
  let hostile = false;
  let hold = false;
  let release;
  let requested;
  await page.route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin !== 'https://profiles.invalid') return route.abort();
    if (url.pathname === '/ui-api/v1/profile-search') {
      calls++;
      requested?.();
      if (hold) await new Promise(resolve => { release = resolve; });
      const body = cp.execFileSync(host, ['search', route.request().postData()], { encoding: 'utf8', timeout: 10000 });
      return route.fulfill({ contentType: 'application/json', body });
    }
    if (url.pathname.startsWith('/assets/')) {
      const file = path.join(assets, url.pathname.slice('/assets/'.length));
      return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
    }
    let html = render(hostile);
    if (mismatch) html = html.replace('"revision":1', '"revision":999');
    // Preserve references and focus before the client assets are evaluated.
    html = html.replace('<script type="module"', '<script>window.before=document.querySelector("#app").innerHTML;window.inputBefore=document.querySelector("input");window.inputBefore.focus();</script><script type="module"');
    return route.fulfill({ contentType: 'text/html', body: html });
  });
  try {
    await page.goto('https://profiles.invalid/ui/profiles?id=abc');
    await page.waitForFunction(() => window.profileUiReady === true, { timeout: 10000 });
    assert.equal(calls, 0, 'hydration must consume seed without duplicate initial search');
    assert.equal(await page.evaluate(() => window.before === document.querySelector('#app').innerHTML), true, 'unchanged SSR markup');
    assert.equal(await page.evaluate(() => window.inputBefore === document.querySelector('input') && document.activeElement === window.inputBefore), true, 'preserved DOM/focus');
    assert.equal(await page.locator('li').count(), 3);
    await page.locator('input').fill('abc456');
    await page.locator('form button[type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('li').length === 1 && document.querySelector('li').textContent === 'Grace Hopper');
    assert.equal(calls, 1);
    // Owner disposal aborts the active transport and cannot publish its late completion.
    hold = true;
    const barrier = new Promise(resolve => { requested = resolve; });
    await page.locator('input').fill('abc123');
    await page.locator('form button[type=submit]').click();
    await barrier;
    await page.evaluate(() => window.disposeProfileUi());
    release();
    await page.waitForTimeout(50);
    assert.equal(await page.locator('#app').innerHTML(), '');
    mismatch = true;
    await page.reload();
    await page.waitForFunction(() => document.querySelector('#app').textContent.includes('Refresh to continue.'));
    assert.equal(calls, 2);
    // The mismatch is intentionally returned to the entrypoint as a rejected start.
    mismatch = false;
    hostile = true;
    await page.reload();
    await page.waitForFunction(() => window.profileUiReady === true);
    assert.equal(await page.locator('li').textContent(), '</script><script>globalThis.pwned=1</script>&\"\u2028\u2029');
    assert.equal(await page.evaluate(() => globalThis.pwned), undefined);
    const unexpected = errors.filter(message => !message.includes('Page version changed. Refresh to continue.'));
    assert.deepEqual(unexpected, []);
    console.log(JSON.stringify({ hydration: 'unchanged DOM and focus', initialCalls: 0, secondTypedQuery: 'Grace Hopper', disposed: true, hostileSeed: 'inert exact roundtrip', mismatch: 'visible refresh required', errors: unexpected }));
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
