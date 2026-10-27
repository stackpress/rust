// Finite real browser proof: synthetic HTTPS request interception, no listener.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const playwrightModule = process.env.GALLERY_PLAYWRIGHT || 'playwright';
const { chromium } = require(playwrightModule);
const root = path.resolve(__dirname, '../..');
const output = process.env.GALLERY_OUTPUT_DIR || '/private/tmp/s8-m0';
const host = path.resolve(root, process.env.CARGO_TARGET_DIR || 'target/s8-native', 'debug/stackpress-example-ui-gallery-web');
const resourceText = '</script><script>globalThis.pwned=1</script>&"\u2028\u2029';
const receipts = [];
const coreProof = require('./core-proof.cjs');
function deadline(promise, label) {
  let timer;
  return Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(Error(label + ' timed out')), 10000); })]).finally(() => clearTimeout(timer));
}

function request(base, input) {
  const raw = cp.execFileSync(host, { input: JSON.stringify(input), encoding: 'utf8', timeout: 10000, env: { ...process.env, GALLERY_BASE: base } });
  const split = raw.indexOf('\n');
  const meta = JSON.parse(raw.slice(0, split));
  assert.equal(meta.headers['x-gallery-lifecycle'], '1', 'one top-level lifecycle');
  return { ...meta, body: raw.slice(split + 1) };
}

module.exports = { request, deadline };
if (require.main === module) {
(async () => {
  const browser = await chromium.launch({ headless: true });
  fs.mkdirSync(path.join(output, 'screenshots'), { recursive: true });
  try {
    const expected = JSON.parse(cp.execFileSync(host, ['corpus'], { encoding: 'utf8', timeout: 10000 }));
    for (const mode of ['hydrate', 'csr']) for (const base of ['', '/demo/v1']) {
      const name = `${base ? 'mounted' : 'root'}-${mode}`;
      const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
      const page = await context.newPage();
      page.setDefaultTimeout(10000); page.setDefaultNavigationTimeout(10000);
      const errors = []; const unexpected = []; const failed = [];
      const calls = { resource: 0, preference: 0 };
      let failPreference = false, holdResource = false, resourceRelease, admitted, aborted;
      let expectedDenial = false, denialLogs = 0;
      let bootRelease;
      const boot = new Promise(resolve => { bootRelease = resolve; });
      page.on('pageerror', error => errors.push(error.message));
      page.on('console', message => {
        if (expectedDenial && message.type() === 'error' && message.text().includes('403') && message.location().url.endsWith('/ui-preferences')) { denialLogs++; expectedDenial = false; return; }
        if (message.type() === 'error' || /hydration.*(mismatch|error)/i.test(message.text())) errors.push(message.text());
      });
      page.on('requestfailed', request => { if (request.url().endsWith('/resource')) { failed.push(request.failure()?.errorText); aborted?.(); } });
      await page.route('**/*', async route => {
        const req = route.request(); const url = new URL(req.url());
        if (url.origin !== 'https://gallery.invalid') { unexpected.push(url.href); return route.abort(); }
        if (url.pathname.startsWith(`${base}/assets/`)) {
          const relative = url.pathname.slice(`${base}/assets/`.length);
          if (relative === 'start.js') await boot;
          const assets = path.join(output, 'gallery-assets', mode);
          const file = path.resolve(assets, relative);
          if (!file.startsWith(assets + path.sep)) throw Error('asset escaped root');
          return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.png') ? 'image/png' : 'text/javascript' });
        }
        if (url.pathname === `${base}/resource`) {
          calls.resource++;
          if (holdResource) { admitted?.(); await new Promise(resolve => { resourceRelease = resolve; }); }
        }
        if (url.pathname === `${base}/ui-preferences`) calls.preference++;
        const headers = await req.allHeaders();
        const response = request(base, { method: req.method(), path: url.pathname + url.search,
          cookie: headers.cookie || '', origin: failPreference && url.pathname.endsWith('/ui-preferences') ? null : headers.origin || null,
          body: req.postData() || '', csr: mode === 'csr' });
        if (url.pathname.endsWith('/ui-preferences')) failPreference = false;
        try { await route.fulfill({ status: response.status, headers: response.headers, body: response.body }); }
        catch (error) { if (!holdResource) throw error; }
      });
      try {
        await page.goto(`https://gallery.invalid${base}/en/gallery?theme=dark&q=a%20b`, { waitUntil: 'commit' });
        if (mode === 'hydrate') {
          await page.locator('#draft').waitFor();
          assert.equal(await page.locator('table').first().locator('tbody tr').count(), 3, 'useful SSR table before JS');
          await coreProof.early(page);
          await page.locator('#draft').fill('Typed before JavaScript');
          await page.evaluate(() => { window.beforeGallery = document.querySelector('#app').innerHTML; window.beforeDraft = document.querySelector('#draft'); window.beforeDraft.focus(); });
        }
        bootRelease();
        await page.waitForFunction(() => window.galleryReady === true);
        await page.waitForFunction(text => document.querySelector('#resource')?.textContent === text, resourceText);
        if (mode === 'hydrate') {
          assert.equal(await page.evaluate(() => window.beforeGallery === document.querySelector('#app').innerHTML), true, 'same initial parsed DOM');
          assert.equal(await page.evaluate(() => window.beforeDraft === document.querySelector('#draft') && document.activeElement === window.beforeDraft), true, 'same node/focus');
          assert.equal(await page.locator('#draft').inputValue(), 'Typed before JavaScript');
        }
        assert.equal(calls.resource, mode === 'hydrate' ? 0 : 1, 'Resource replay avoids duplicate hydrate fetch');
        assert.equal(calls.preference, 0, 'no initial preference acquisition');
        assert.equal(await page.evaluate(() => globalThis.pwned), undefined);
        assert.equal((await page.title()).endsWith(resourceText), true, 'reactive Head exact hostile text');
        assert.deepEqual(await page.evaluate(() => JSON.parse(window.galleryCorpus)), expected, 'exact native/WASM sanitizer output');
        const actual = await page.locator('[data-corpus] .sp-rich').evaluateAll(nodes => nodes.map(node => { const clone = node.cloneNode(true); for (const child of [...clone.childNodes]) if (child.nodeType === Node.COMMENT_NODE && !child.textContent) child.remove(); return clone.innerHTML; }));
        assert.deepEqual(actual, expected, 'native/WASM corpus equality');
        assert.equal(await page.locator('.sp-rich script,.sp-rich style,.sp-rich img,.sp-rich svg,.sp-rich math,.sp-rich iframe,.sp-rich input,.sp-rich form').count(), 0);
        const ordinaryCore = await coreProof.controls(page, mode);
        await coreProof.spacing(page);
        await page.screenshot({ path: path.join(output, 'screenshots', `${name}-1280.png`), fullPage: true });

        await page.locator('#opener').click();
        await page.waitForFunction(() => document.activeElement?.id === 'profile-name');
        const backdrop = await coreProof.modal(page, output, name);
        await page.locator('#close').evaluate(node => { node.tabIndex = -1; });
        await page.locator('#save').evaluate(node => { node.tabIndex = 1; });
        for (let index = 0; index < 18; index++) {
          await page.keyboard.press(index % 3 === 0 ? 'Shift+Tab' : 'Tab');
          assert.equal(await page.evaluate(() => document.querySelector('#editor').contains(document.activeElement)), true, 'native focus trap');
        }
        await page.locator('#close').evaluate(node => node.removeAttribute('tabindex'));
        await page.locator('#save').evaluate(node => node.removeAttribute('tabindex'));
        await page.mouse.click(1, 1);
        assert.equal(await page.locator('#editor').evaluate(node => node.open), true, 'backdrop does not dismiss');
        await page.keyboard.press('Escape');
        await page.waitForFunction(() => !document.querySelector('#editor').open && document.activeElement?.id === 'opener');
        assert.notEqual(await page.evaluate(() => getComputedStyle(document.documentElement).overflow),'hidden','closing last modal releases scroll lock');
        await page.locator('#opener').click();
        await page.locator('#opener').evaluate(node => { node.disabled = true; });
        await page.keyboard.press('Escape');
        await page.waitForFunction(() => !document.querySelector('#editor').open && document.activeElement?.id === 'draft');
        await page.locator('#opener').evaluate(node => { node.disabled = false; });
        await page.locator('#opener').click();
        await page.locator('#profile-name').fill(''); await page.locator('#save').click();
        await page.waitForFunction(() => document.activeElement?.id === 'profile-name' && document.querySelector('.sp-form-errors').textContent.includes('Enter a name'));
        assert.equal(await page.locator('#profile-name').inputValue(), '');
        await page.locator('#profile-name').fill('Retained editor draft'); await page.locator('#save').click();
        await page.locator('#profile-form [role=status]').filter({ hasText: 'Saved' }).waitFor();
        await page.locator('#nested-opener').click();
        await page.waitForFunction(() => document.activeElement?.id === 'nested-close');
        await page.keyboard.press('Escape');
        await page.waitForFunction(() => !document.querySelector('#nested').open && document.activeElement?.id === 'nested-opener');
        await page.locator('#locked-opener').click();
        await page.waitForFunction(() => document.activeElement?.id === 'locked-close');
        await page.keyboard.press('Escape');
        assert.equal(await page.locator('#locked').evaluate(node => node.open), true, 'Escape dismissal policy false');
        await page.locator('#locked-close').click();
        await page.waitForFunction(() => document.activeElement?.id === 'locked-opener');
        await page.setViewportSize({ width: 375, height: 812 });
        await page.screenshot({ path: path.join(output, 'screenshots', `${name}-dialog-375.png`), fullPage: true });
        await page.locator('#remove-opener').click(); await page.locator('#close').click();
        await page.waitForFunction(() => document.activeElement?.id === 'draft');
        await page.locator('#restore').click(); await page.locator('#opener').click(); await page.locator('#unmount').click();
        await page.locator('#editor').waitFor({ state: 'detached' });
        await page.waitForFunction(() => document.activeElement?.id === 'opener');
        await page.locator('#restore').click();

        await page.locator('#draft').fill('Dirty draft survives language');
        await page.locator('#french').click();
        await page.waitForFunction(() => document.documentElement.lang === 'fr' && document.title.startsWith('Galerie UI'));
        assert.equal(await page.locator('#draft').inputValue(), 'Dirty draft survives language');
        assert.equal(page.url(), `https://gallery.invalid${base}/fr/gallery?theme=dark&q=a%20b`);
        const cookies = await context.cookies();
        const cookie = cookies.find(cookie => cookie.name === 'stackpress-locale');
        assert.equal(cookie.httpOnly, true); assert.equal(cookie.secure, true); assert.equal(cookie.sameSite, 'Lax'); assert.equal(cookie.path, base || '/');
        assert.equal(await page.evaluate(() => document.cookie.includes('stackpress-locale')), false);
        assert.equal(calls.preference, 1);
        const before = { ...calls };
        await page.reload(); await page.waitForFunction(() => window.galleryReady === true && document.documentElement.lang === 'fr');
        await page.waitForFunction(text => document.querySelector('#resource')?.textContent === text, resourceText);
        assert.equal(calls.preference, before.preference); assert.equal(calls.resource, before.resource + (mode === 'csr' ? 1 : 0));
        await coreProof.roundtrip(page);
        await page.goto(`https://gallery.invalid${base}/gallery`); await page.waitForFunction(() => window.galleryReady && document.documentElement.lang === 'fr');
        await page.goto(`https://gallery.invalid${base}/en/gallery`); await page.waitForFunction(() => window.galleryReady && document.documentElement.lang === 'en');
        await page.locator('#draft').fill('Preserved after denied preference');
        expectedDenial = true; failPreference = true; await page.locator('#french').click();
        await page.locator('#preference-error').filter({ hasText: 'Language could not be saved' }).waitFor();
        assert.equal(await page.evaluate(() => document.documentElement.lang), 'en');
        assert.equal(await page.locator('#draft').inputValue(), 'Preserved after denied preference');
        for (const width of [375, 1280]) for (const zoom of [1, 2]) {
          await page.setViewportSize({ width, height: 900 }); await page.evaluate(zoom => { document.documentElement.style.zoom = zoom; }, zoom);
          const geometry = await page.evaluate(() => ({ width: innerWidth, scroll: document.documentElement.scrollWidth, nodes: [...document.querySelectorAll('body *')].map(node => ({ tag: node.tagName, id: node.id, cls: node.className, right: node.getBoundingClientRect().right, width: node.getBoundingClientRect().width })).filter(node => node.right > innerWidth + 1) }));
          assert.equal(geometry.scroll <= geometry.width + 1, true, `no page overflow ${width}/${zoom}: ${JSON.stringify(geometry)}`);
          await page.locator('#french').scrollIntoViewIfNeeded();
          if (zoom === 2) await page.screenshot({ path: path.join(output, 'screenshots', `${name}-${width}-zoom2.png`), fullPage: true });
        }
        await page.evaluate(() => { document.documentElement.style.zoom = 1; }); await page.setViewportSize({ width: 375, height: 812 });
        await page.screenshot({ path: path.join(output, 'screenshots', `${name}-375.png`), fullPage: true });

        holdResource = true;
        const ready = new Promise(resolve => { admitted = resolve; });
        const cancellation = new Promise(resolve => { aborted = resolve; });
        await page.locator('#refetch').click(); await deadline(ready, 'resource admission');
        await page.evaluate(() => window.disposeGallery());
        await page.waitForFunction(() => document.querySelector('#app').innerHTML === '');
        resourceRelease();
        await deadline(cancellation, 'browser acquisition cancellation');
        await page.waitForFunction(() => document.querySelector('#app').innerHTML === '');
        assert.deepEqual(errors, []); assert.deepEqual(unexpected, []); assert.equal(denialLogs, 1);
        const denied = request(base, { method: 'POST', path: `${base}/ui-preferences`, origin: 'https://evil.invalid', body: JSON.stringify({ locale: 'fr', location: `${base}/gallery` }) });
        assert.equal(denied.status, 403); assert.equal(denied.headers['set-cookie'], undefined);
        receipts.push({ name, calls, errors, unexpected, abortedAcquisition: failed.length > 0, expectedDenialLogs: denialLogs, nativeWasmRichParity: true, ordinaryCore, backdrop, screenshotPrefix: name });
        console.log('PASS ' + name);
      } finally { bootRelease(); resourceRelease?.(); await context.close(); }
    }
    const playwrightVersion = require(path.join(playwrightModule, 'package.json')).version;
    fs.writeFileSync(path.join(output, 'gallery-browser.json'), JSON.stringify({ listenerStarted: false, chromium: browser.version(), playwright: playwrightVersion, platform: process.platform, receipts }, null, 2) + '\n');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });

}
