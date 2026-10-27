// Finite real Chromium proof, serving only .invalid requests through interception.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const modulePath = process.env.PREFERENCES_PLAYWRIGHT;
if (!modulePath) throw Error('PREFERENCES_PLAYWRIGHT must identify the installed package');
const { chromium } = require(modulePath);
const root = path.resolve(__dirname, '../..');
// Keep defaults while allowing the closeout parent to cap its shared target root.
const output = process.env.PREFERENCES_LOGS || '/private/tmp/s8-lane-b';
const nativeTarget = process.env.PROFILE_UI_NATIVE_TARGET || path.join(root, 'target/s8-native');
const host = path.join(nativeTarget, 'debug/stackpress-example-ui-preferences-web');
const hostile = '</script><script>globalThis.pwned=1</script> & public notice';
const receipts = [];

/** Execute the actual adapter once, bounded by a child deadline. */
function request(base, input, policy = false) {
  const env = { ...process.env, PREFERENCES_BASE: base };
  delete env.PREFERENCES_TEST_POLICY;
  if (policy) env.PREFERENCES_TEST_POLICY = 'explicit-fixture';
  const raw = cp.execFileSync(host, { input: JSON.stringify(input), encoding: 'utf8', timeout: 10000, env });
  const split = raw.indexOf('\n');
  const meta = JSON.parse(raw.slice(0, split));
  assert.deepEqual(meta.headers['x-preferences-lifecycle'], ['1/1']);
  assert.deepEqual(meta.headers['cache-control'], ['no-store']);
  return { ...meta, body: raw.slice(split + 1) };
}

(async () => {
  const browser = await chromium.launch({ headless: true });
  fs.mkdirSync(path.join(output, 'screenshots'), { recursive: true });
  try {
    for (const mode of ['hydrate', 'csr']) for (const base of ['', '/demo/v1']) {
      const name = `${base ? 'mounted' : 'root'}-${mode}`;
      const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, colorScheme: 'dark' });
      const page = await context.newPage();
      page.setDefaultTimeout(10000);
      page.setDefaultNavigationTimeout(10000);
      const errors = [], unexpected = [];
      const calls = { writes: 0, confirms: 0 };
      // Only the final link-navigation phase opts into explicit fixture admission.
      let allowProtected = false;
      let failure = '', gateRelease;
      const gate = new Promise(resolve => { gateRelease = resolve; });
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', msg => {
        if (msg.type() === 'error' && /Failed to load resource/.test(msg.text()) && /ui-preferences/.test(msg.location().url)) return;
        if (msg.type() === 'error' || /hydration.*(mismatch|error)/i.test(msg.text())) errors.push(msg.text());
      });
      await page.addInitScript(() => {
        window.themeMutations = [];
        new MutationObserver(records => {
          for (const record of records) if (record.attributeName === 'data-theme') window.themeMutations.push(document.documentElement.dataset.theme);
        }).observe(document, { subtree: true, attributes: true });
        document.addEventListener('DOMContentLoaded', () => {
          window.firstColors = { theme: document.documentElement.dataset.theme, background: getComputedStyle(document.body).backgroundColor };
        }, { once: true });
      });
      await page.route('**/*', async route => {
        const req = route.request();
        const url = new URL(req.url());
        if (url.origin !== 'https://preferences.invalid') { unexpected.push(url.href); return route.abort(); }
        if (url.pathname.startsWith(`${base}/assets/`)) {
          const relative = url.pathname.slice(`${base}/assets/`.length);
          if (relative === 'start.js') await gate;
          const assets = path.join(output, 'assets', mode);
          const file = path.resolve(assets, relative);
          if (!file.startsWith(assets + path.sep)) throw Error('asset path escaped');
          return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
        }
        const headers = await req.allHeaders();
        const write = url.pathname === `${base}/ui-preferences`;
        const confirm = url.pathname === `${base}/ui-preferences/confirm`;
        if (write) calls.writes++;
        if (confirm) calls.confirms++;
        const response = request(base, { method: req.method(), path: url.pathname + url.search,
          cookie: headers.cookie || '', origin: write && failure === 'origin' ? null : headers.origin || null,
          body: req.postData() || '', csr: mode === 'csr', caller: allowProtected ? 'allowed' : '' }, allowProtected);
        const outgoing = Object.fromEntries(Object.entries(response.headers).map(([key, values]) => [key, values.join('\n')]));
        if (write && failure === 'blocked-cookie') delete outgoing['set-cookie'];
        if (write && failure === 'ack') response.body = JSON.stringify({ preferences: { locale: 'fr', theme: 'dark' }, location: '/elsewhere/fr/preferences' });
        await route.fulfill({ status: response.status, headers: outgoing, body: response.body });
        if (write && failure !== 'blocked-cookie') failure = '';
        if (confirm) failure = '';
      });
      try {
        const flash = Buffer.from(JSON.stringify({ id: 'notice-1', message: hostile })).toString('hex');
        await context.addCookies([{ name: 'stackpress-flash', value: flash, domain: 'preferences.invalid', path: base || '/', secure: true, httpOnly: true, sameSite: 'Lax' }]);
        await page.goto(`https://preferences.invalid${base}/en/preferences?q=a%2Fb&return=%252F`, { waitUntil: 'commit' });
        await page.locator('body').waitFor({ state: 'attached' });
        assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), 'system');
        assert.equal(await page.evaluate(() => getComputedStyle(document.body).backgroundColor), 'rgb(23, 23, 23)', 'CSS resolves system dark before JS');
        if (mode === 'hydrate') {
          await page.locator('#draft').fill('Typed before JavaScript');
          await page.evaluate(() => { window.beforeDOM = document.querySelector('#app').innerHTML; window.beforeDraft = document.querySelector('#draft'); window.beforeDraft.focus(); });
          assert.equal(await page.locator('#flash span').textContent(), hostile);
        }
        gateRelease();
        await page.waitForFunction(() => window.preferencesReady === true);
        if (mode === 'hydrate') {
          assert.equal(await page.evaluate(() => window.beforeDOM === document.querySelector('#app').innerHTML), true);
          assert.equal(await page.evaluate(() => window.beforeDraft === document.querySelector('#draft') && document.activeElement === window.beforeDraft), true);
          assert.equal(await page.locator('#draft').inputValue(), 'Typed before JavaScript');
        }
        assert.deepEqual(calls, { writes: 0, confirms: 0 }, 'no initial preference acquisition');
        assert.equal(await page.locator('#flash').count(), 1);
        assert.equal(await page.locator('#flash span').textContent(), hostile);
        assert.equal(await page.evaluate(() => globalThis.pwned), undefined);
        assert.equal(await page.locator('#app script').count(), 0);
        assert.equal(await page.locator('#private-link').getAttribute('href'), null);
        assert.equal(await page.locator('#private-mutation').isDisabled(), true);
        await page.screenshot({ path: path.join(output, 'screenshots', `${name}-flash.png`), fullPage: true });
        await page.locator('#dismiss').click();
        assert.equal(await page.locator('#flash').count(), 0);
        await page.locator('#draft').fill('Dirty draft survives preferences');
        await page.locator('#french').click();
        await page.waitForFunction(() => document.documentElement.lang === 'fr' && document.title === 'Préférences');
        assert.equal(await page.locator('#draft').inputValue(), 'Dirty draft survives preferences');
        assert.equal(page.url(), `https://preferences.invalid${base}/fr/preferences?q=a%2Fb&return=%252F`);
        assert.equal(await page.locator('#home').getAttribute('href'), `${base}/fr/preferences`);
        assert.match(await page.locator('#samples').textContent(), /2 janvier 2026 · 1\u202f234,50 · 0 article/);
        await page.locator('#dark').click();
        await page.waitForFunction(() => document.documentElement.dataset.theme === 'dark');
        await page.screenshot({ path: path.join(output, 'screenshots', `${name}-fr-dirty-dark.png`), fullPage: true });
        const cookies = await context.cookies();
        for (const key of ['stackpress-locale', 'stackpress-theme']) {
          const cookie = cookies.find(value => value.name === key);
          assert.equal(cookie.path, base || '/'); assert.equal(cookie.httpOnly, true); assert.equal(cookie.secure, true); assert.equal(cookie.sameSite, 'Lax');
        }
        assert.equal(cookies.some(value => value.name === 'stackpress-flash'), false, 'same-scope flash was consumed');
        assert.equal(await page.evaluate(() => document.cookie.includes('stackpress-')), false);
        const before = { ...calls };
        await page.reload();
        await page.waitForFunction(() => window.preferencesReady === true && window.firstColors);
        assert.deepEqual(await page.evaluate(() => window.firstColors), { theme: 'dark', background: 'rgb(23, 23, 23)' });
        assert.deepEqual(await page.evaluate(() => window.themeMutations.filter(value => value !== 'dark')), [], 'no initial light theme transition');
        assert.equal(await page.locator('#flash').count(), 0);
        assert.deepEqual(calls, before, 'refresh does not fetch preferences');
        await page.goto(`https://preferences.invalid${base}/preferences`);
        await page.waitForFunction(() => window.preferencesReady && document.documentElement.lang === 'fr');
        await page.goto(`https://preferences.invalid${base}/en/preferences`);
        await page.waitForFunction(() => window.preferencesReady && document.documentElement.lang === 'en');
        await page.goBack();
        await page.waitForFunction(() => window.preferencesReady && document.documentElement.lang === 'fr');
        await page.goForward();
        await page.waitForFunction(() => window.preferencesReady && document.documentElement.lang === 'en');
        await page.locator('#draft').fill('Draft after denied or blocked persistence');
        for (const reason of ['origin', 'blocked-cookie', 'ack']) {
          failure = reason;
          await page.locator('#light').click();
          await page.locator('#save-error').filter({ hasText: 'Preferences could not be saved' }).waitFor();
          assert.equal(await page.locator('#draft').inputValue(), 'Draft after denied or blocked persistence');
          assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), 'dark');
          assert.equal(await page.evaluate(() => document.documentElement.lang), 'en');
          assert.equal(page.url(), `https://preferences.invalid${base}/en/preferences`);
          if (reason === 'blocked-cookie') await page.screenshot({ path: path.join(output, 'screenshots', `${name}-failed-save.png`), fullPage: true });
        }
        await page.locator('#system').click();
        await page.waitForFunction(() => document.documentElement.dataset.theme === 'system');
        await page.emulateMedia({ colorScheme: 'light' });
        assert.equal(await page.evaluate(() => getComputedStyle(document.body).backgroundColor), 'rgb(250, 250, 250)');
        await page.emulateMedia({ colorScheme: 'dark' });
        assert.equal(await page.evaluate(() => getComputedStyle(document.body).backgroundColor), 'rgb(23, 23, 23)');
        for (const width of [375, 1280]) for (const zoom of [1, 2]) {
          await page.setViewportSize({ width, height: 900 });
          await page.evaluate(value => { document.documentElement.style.zoom = value; }, zoom);
          assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), true, 'no page-wide clipping');
          const wholeWords = await page.locator('h1').evaluate(heading => {
            const walker = document.createTreeWalker(heading, NodeFilter.SHOW_TEXT);
            for (let text = walker.nextNode(); text; text = walker.nextNode()) {
              for (const word of text.textContent.matchAll(/\S+/g)) {
                const range = document.createRange();
                range.setStart(text, word.index);
                range.setEnd(text, word.index + word[0].length);
                // A word broken across lines produces more than one text rectangle.
                if (range.getClientRects().length !== 1) return false;
              }
            }
            return heading.scrollWidth <= heading.clientWidth + 1;
          });
          assert.equal(wholeWords, true, `whole heading words fit at ${width}/${zoom}`);
          const actionGap = await page.locator('.actions').evaluate(group => parseFloat(getComputedStyle(group).gap));
          assert.equal(actionGap, 8, 'protected actions use the shared half-rem gap');
          await page.screenshot({ path: path.join(output, 'screenshots', `${name}-${width}-zoom${zoom}.png`), fullPage: true });
        }
        const denied = request(base, { method: 'GET', path: `${base}/private` });
        assert.equal(denied.status, 403); assert.deepEqual(denied.headers['x-protected-calls'], ['0']);
        const forged = request(base, { method: 'POST', path: `${base}/private`, caller: 'allowed', origin: 'https://evil.invalid', csrf: 'fixture-csrf' }, true);
        assert.equal(forged.status, 403); assert.deepEqual(forged.headers['x-protected-calls'], ['0']);
        await page.evaluate(() => window.disposePreferences());
        assert.equal(await page.locator('#app').innerHTML(), '');

        // A fresh allowed projection must expose a usable link at the same mount.
        allowProtected = true;
        await page.goto(`https://preferences.invalid${base}/en/preferences`);
        await page.waitForFunction(() => window.preferencesReady === true);
        const link = page.locator('#private-link');
        await link.waitFor({ state: 'visible' });
        assert.equal(await link.getAttribute('aria-disabled'), 'false');
        assert.equal(await link.getAttribute('href'), `${base}/private`);
        await page.screenshot({ path: path.join(output, 'screenshots', `${name}-protected-link.png`), fullPage: true });

        // Click the real anchor; the real adapter must report one protected call.
        const destination = `https://preferences.invalid${base}/private`;
        const [admitted] = await Promise.all([
          page.waitForResponse(response => response.url() === destination && response.request().isNavigationRequest()),
          link.click()
        ]);
        await page.waitForURL(destination);
        assert.equal(admitted.status(), 200);
        assert.equal(admitted.headers()['x-protected-calls'], '1');
        assert.deepEqual(await admitted.json(), { public: 'Explicit fixture admission' });
        assert.deepEqual(errors, []); assert.deepEqual(unexpected, []);
        receipts.push({ name, calls, errors, unexpected, beforePaintSystem: true, refreshNoFlash: true, blockedCookieDenied: true, flashConsumed: true, dirtyDraftPreserved: true, protectedLink: { destination, status: admitted.status(), calls: 1 } });
        console.log('PASS ' + name);
      } finally { gateRelease(); await context.close(); }
    }
    fs.writeFileSync(path.join(output, 'browser.json'), JSON.stringify({ listenerStarted: false, chromium: browser.version(), playwright: require(path.join(modulePath, 'package.json')).version, receipts }, null, 2) + '\n');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
