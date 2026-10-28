// M0: real native chunks, strict CSP and erroring browser document streams.
// The worker errors on missing native terminal framing; no failure becomes EOF.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.UI_STREAM_PLAYWRIGHT);
const {native, worker} = require('./browser.cjs');
const out = process.env.UI_STREAM_OUTPUT_DIR;

/** Exercise one document with independently owned browser/native lifetimes. */
async function scenario(browser, mode, base) {
  const context = await browser.newContext({viewport: {width: 1280, height: 800}});
  const page = await context.newPage();
  page.setDefaultTimeout(12000);
  const pageErrors = [], unexpected = [], failed = [], assetRequests = [];
  let session, resources = 0;
  page.on('pageerror', error => pageErrors.push(error.message));
  page.on('requestfailed', request => failed.push({url: request.url(), error: request.failure()}));
  try {
    await context.route('**/*', async route => {
      const url = new URL(route.request().url());
      if (url.origin !== 'https://stream.invalid') {
        unexpected.push(url.href);
        return route.abort();
      }
      if (url.pathname === '/install') {
        return route.fulfill({contentType: 'text/html', body: '<!doctype html><title>M0 status proof</title>'});
      }
      if (url.pathname === '/sw.js') {
        return route.fulfill({contentType: 'application/javascript', body: worker});
      }
      if (url.pathname === '/__next') {
        const nativeMode = mode === 'truncated' ? 'disconnect'
          : ['held-pending', 'delayed-status', 'blocked-status'].includes(mode) ? 'pending'
          : mode === 'slow-success' ? 'success' : mode;
        session ??= native(nativeMode, base);
        const next = await session.next();
        // An errored ReadableStream discards queued chunks. Acknowledge parsed
        // useful bytes before testing a *post-shell* interruption, rather than
        // accidentally testing navigation failure before the shell is received.
        if (mode === 'truncated' && next.done) {
          await page.locator('#shell').waitFor({state: 'visible'});
        }
        // Keep parsing pending past the script's guidance threshold. The native
        // report remains unchanged, proving this is browser delivery evidence.
        if ((mode === 'held-pending' && next.done)
          || (mode === 'slow-success' && next.chunk?.includes('id="details"'))) {
          await new Promise(resolve => setTimeout(resolve, 7000));
        }
        return route.fulfill({contentType: 'application/json', body: JSON.stringify(next)});
      }
      if (url.pathname === '/__cancel') {
        session?.cancel();
        return route.fulfill({status: 204, body: ''});
      }
      if (url.pathname === '/resource') {
        resources++;
        return route.fulfill({body: 'DUPLICATE RESOURCE FETCH'});
      }
      const file = path.basename(url.pathname);
      if (['start.js', 'stream.js', 'stream_bg.wasm', 'stream-status.js'].includes(file)) {
        assetRequests.push(file);
        if (file === 'stream-status.js' && mode === 'blocked-status') return route.abort('blockedbyclient');
        if (file === 'stream-status.js' && mode === 'delayed-status') {
          await new Promise(resolve => setTimeout(resolve, 2800));
        }
        return route.fulfill({path: path.join(out, 'assets', file),
          contentType: file.endsWith('.wasm') ? 'application/wasm' : 'application/javascript'});
      }
      if (url.pathname === '/favicon.ico') return route.fulfill({status: 204, body: ''});
      unexpected.push(url.href);
      return route.abort();
    });
    await page.goto('https://stream.invalid/install');
    await page.evaluate(async () => {
      await navigator.serviceWorker.register('/sw.js');
      await navigator.serviceWorker.ready;
    });
    const name = `${mode}-${base ? 'mounted' : 'root'}`;
    const began = performance.now();
    await page.goto(`https://stream.invalid${base}/stream?token=${name}`, {waitUntil: 'commit'});
    const recovery = page.locator('#stackpress-stream-recovery');
    await page.locator('#shell').waitFor({state: 'visible'});
    // An actual inserted executable script must be refused by document CSP.
    await page.evaluate(() => {
      const element = document.createElement('script');
      element.textContent = 'window.m0InlineExecuted = true';
      document.head.append(element);
    });
    assert.equal(await page.evaluate(() => window.m0InlineExecuted), undefined);
    let pendingState;
    if (mode === 'blocked-status') {
      await page.waitForTimeout(6500);
      assert.match(await recovery.textContent(), /Loading or interrupted/);
      assert.match(await recovery.textContent(), /Reload this page/);
    } else if (mode === 'success') {
      await page.locator('html[data-hydrated=true]').waitFor();
      // A disposed status watchdog must not revive an already completed notice.
      await page.waitForTimeout(6500);
      assert.equal(await recovery.isVisible(), false);
    } else {
      await recovery.locator('xpath=self::*[@role="alert"]').waitFor();
      const text = await recovery.textContent();
      assert.match(text, /This page (didn't finish|hasn't finished) loading/);
      pendingState = await page.evaluate(() => document.readyState);
      if (['held-pending', 'slow-success'].includes(mode)) {
        assert.match(text, /hasn't finished/);
        assert.equal(pendingState, 'loading', 'watchdog must run before parsing ends');
      }
      await page.screenshot({path: path.join(out, `${name}-recovery.png`), fullPage: true});
    }
    if (mode === 'slow-success') {
      await page.locator('html[data-hydrated=true]').waitFor();
      assert.equal(await recovery.isVisible(), false);
    }
    const succeeded = ['success', 'slow-success'].includes(mode);
    assert.equal(await page.locator('html').getAttribute('data-hydrated'), succeeded ? 'true' : null);
    assert.equal(resources, 0, 'initial Resource must never be fetched by partial-page recovery');
    assert.equal(await page.evaluate(() => window.hostile), undefined);
    if (succeeded) {
      await page.locator('#counter').click();
      assert.equal(await page.locator('#counter').textContent(), 'Clicks: 1');
    } else {
      assert.equal(await page.locator('#stackpress-stream-complete').count(), 0);
      assert.equal(await page.locator('#details').count(), 0);
      assert.equal(await page.evaluate(() => window.streamBeforeHydrate), undefined);
    }
    assert.equal(await recovery.locator('a').getAttribute('href'), `${base}/stream`);
    assert.ok(assetRequests.includes('stream-status.js'));
    await page.setViewportSize({width: 375, height: 750});
    await page.screenshot({path: path.join(out, `${name}-375.png`), fullPage: true});
    await session.close();
    assert.equal(session.report.terminal, succeeded);
    assert.equal(session.report.released, true);
    assert.equal(session.report.cleanups, 1);
    assert.equal(session.report.hooks, 1);
    assert.equal(session.report.errors, 0);
    assert.equal(session.report.delivery === 'Some(Completed)', succeeded);
    assert.deepEqual(pageErrors, []);
    assert.deepEqual(unexpected, []);
    return {mode, base, seconds: (performance.now() - began) / 1000, pendingState,
      resources, assetRequests, failed, report: session.report};
  } finally {
    // Close stdin and reap even when navigation, assertions or asset loading fail.
    if (session) await session.close();
    await page.evaluate(async () => {
      for (const registration of await navigator.serviceWorker.getRegistrations()) await registration.unregister();
    }).catch(() => {});
    await context.close();
  }
}

/** Keep the feasibility matrix bounded by the parent's kill/reap watchdog. */
async function main() {
  const browser = await chromium.launch({headless: true});
  const results = [];
  try {
    for (const base of ['', '/demo']) {
      for (const mode of ['success', 'truncated', 'pending', 'deadline', 'late', 'held-pending', 'delayed-status']) {
        const result = await scenario(browser, mode, base);
        results.push(result);
        fs.writeFileSync(path.join(out, 'status-browser.json'), JSON.stringify({version: browser.version(), results}, null, 2));
        console.log('PASS M0', mode, base || 'root');
      }
    }
    for (const mode of ['slow-success', 'blocked-status']) results.push(await scenario(browser, mode, ''));
    fs.writeFileSync(path.join(out, 'status-browser.json'), JSON.stringify({version: browser.version(), results}, null, 2));
  } finally {
    await browser.close();
  }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
