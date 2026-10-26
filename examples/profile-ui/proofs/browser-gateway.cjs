// Finite browser proof of the actual gateway service. Every request uses its no-port probe.
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PROFILE_UI_PLAYWRIGHT || 'playwright');
const probe = process.env.PROFILE_UI_GATEWAY_PROBE;
const base = process.env.PROFILE_UI_BASE ?? '/demo';
if (!probe) throw Error('set PROFILE_UI_GATEWAY_PROBE to the built finite probe');
function request(method, uri, body = '') {
  const bytes = cp.execFileSync(probe, [method, uri, body], {
    env: { ...process.env, PROFILE_UI_BASE: base }, timeout: 10000, maxBuffer: 70 * 1024 * 1024
  });
  const end = bytes.indexOf(10);
  return { ...JSON.parse(bytes.subarray(0, end).toString()), body: bytes.subarray(end + 1) };
}
(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  let mode = 'normal';
  let calls = 0;
  let documents = 0;
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {
    if ((message.type() === 'error' && mode === 'normal') || /hydration.*(mismatch|error)/i.test(message.text())) errors.push(message.text());
  });
  await page.route('**/*', async route => {
    const input = route.request();
    const url = new URL(input.url());
    if (url.origin !== 'https://profiles.invalid') return route.abort();
    let uri = url.pathname + url.search;
    if (mode === 'missing-js' && url.pathname.endsWith('/profile_ui.js')) uri = `${base}/assets/missing.js`;
    if (mode === 'missing-wasm' && url.pathname.endsWith('.wasm')) uri = `${base}/assets/missing.wasm`;
    if (input.method() === 'POST') calls++;
    const reply = request(input.method(), uri, input.postData() || '');
    if (reply.headers['content-type']?.startsWith('text/html')) {
      documents++;
      let html = reply.body.toString();
      if (mode === 'seed') html = html.replace('"revision":1', '"revision":999');
      if (mode === 'build') html = html.replaceAll(/"build":"[^"]+"/g, '"build":"old-client-or-host"');
      reply.body = Buffer.from(html);
    }
    // Playwright supplies framing; the body may be changed by a deliberate drift fixture.
    delete reply.headers['content-length'];
    return route.fulfill(reply);
  });
  const ready = () => page.waitForFunction(() => window.profileUiReady === true, { timeout: 15000 });
  const rows = () => page.locator('.sp-list li:visible');
  try {
    await page.goto(`https://profiles.invalid${base}/ui/profiles?id=abc`);
    await ready();
    assert.equal(calls, 0);
    assert.equal(await rows().count(), 3);
    assert.match(await page.locator('script[type=module]').getAttribute('src'), /\/assets\/[a-f0-9]{64}\/start.js$/);
    const customized = process.env.PROFILE_UI_CUSTOMIZED === '1';
    assert.equal(await rows().first().textContent(), `${customized ? 'Demo: ' : ''}Ada Lovelace`);
    if (process.env.PROFILE_UI_SCREENSHOT) await page.screenshot({ path: `${process.env.PROFILE_UI_SCREENSHOT}-initial.png`, fullPage: true });
    for (const failure of ['seed', 'build', 'missing-js', 'missing-wasm']) {
      mode = failure;
      await page.goto(`https://profiles.invalid${base}/ui/profiles?id=abc`);
      await page.getByRole('button', { name: 'Reload page', exact: true }).waitFor();
      assert.equal(calls, 0, 'failed startup cannot issue a duplicate search');
      const recovery = page.getByRole('button', { name: 'Reload page', exact: true });
      assert.equal(await recovery.evaluate(element => element === document.activeElement), true);
      const box = await recovery.boundingBox();
      assert.ok(box.y >= 0 && box.y + box.height <= page.viewportSize().height, 'recovery action is in the viewport');
      if (failure === 'missing-wasm' && process.env.PROFILE_UI_SCREENSHOT) await page.screenshot({ path: `${process.env.PROFILE_UI_SCREENSHOT}-recovery.png`, fullPage: true });
      const count = documents;
      await page.waitForTimeout(75);
      assert.equal(documents, count, 'startup failure has no automatic reload loop');
      if (failure.startsWith('missing')) assert.equal(await rows().count(), 3, 'useful SSR remains visible');
      mode = 'normal';
      await page.getByRole('button', { name: 'Reload page', exact: true }).click();
      await ready();
      assert.equal(await page.locator('#stackpress-recovery').count(), 0);
      assert.equal(await rows().count(), 3);
    }
    await page.locator('#profile-id').fill('abc456');
    await page.locator('form button[type=submit]').click();
    await page.waitForFunction(() => document.querySelectorAll('.sp-list li').length === 1);
    assert.equal(await rows().first().textContent(), `${customized ? 'Demo: ' : ''}Grace Hopper`);
    assert.equal(calls, 1);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ base, contentAddressedAssets: true, initialCalls: 0, seedAndBuildMismatch: true, missingJsAndWasm: true, explicitRecovery: true, secondTypedQuery: true, customized, errors }));
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
