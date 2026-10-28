// Real Chromium parser streaming via an intercepted same-origin service worker.
// The worker returns a ReadableStream of actual Hyper-duplex body chunks. Node
// does not synthesize HTML, Suspense resolution, replay or completion markers.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');
const { chromium } = require(process.env.UI_STREAM_PLAYWRIGHT);
const out = process.env.UI_STREAM_OUTPUT_DIR || '/private/tmp/s8-stream';
const worker = `
self.addEventListener('install', event => self.skipWaiting());
self.addEventListener('activate', event => event.waitUntil(clients.claim()));
self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);
  if (!url.pathname.endsWith('/stream')) return;
  const token = url.searchParams.get('token');
  const encoder = new TextEncoder();
  event.respondWith(new Response(new ReadableStream({
    async pull(controller) {
      try {
        const response = await fetch('/__next?token=' + token);
        const value = await response.json();
        if (value.done && value.terminal) controller.close();
        else if (value.done) controller.error(new Error('Native HTTP body was truncated'));
        else controller.enqueue(encoder.encode(value.chunk));
      } catch (error) { controller.error(error); }
    },
    cancel() { return fetch('/__cancel?token=' + token, {method: 'POST'}).catch(() => {}); }
  }), {headers: {
    'Content-Type': 'text/html; charset=utf-8', 'Cache-Control': 'no-store',
    'Content-Security-Policy': "script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'none'"
  }}));
});`;

// The finite native child owns the real host. Stdin is its explicit cancel
// signal; normal EOF/report closes it too. Every child is awaited in finally.
function native(mode, base) {
  const child = spawn(path.join(out, 'stream-host'), [mode, base], {stdio: ['pipe', 'pipe', 'pipe']});
  const queue = [], pending = [], chunks = [];
  let report;
  const log = fs.createWriteStream(path.join(out, `native-${mode}-${base ? 'mounted' : 'root'}.log`));
  child.stderr.pipe(log);
  const deliver = value => pending.length ? pending.shift()(value) : queue.push(value);
  // Split only the NDJSON ASCII LF delimiter. Node's readline also splits
  // U+2028/U+2029, which are intentionally present inside our valid JSON text.
  let buffered = '';
  child.stdout.setEncoding('utf8');
  child.stdout.on('data', text => {
    buffered += text;
    let newline;
    while ((newline = buffered.indexOf('\n')) !== -1) {
      const line = buffered.slice(0, newline); buffered = buffered.slice(newline + 1);
      const value = JSON.parse(line);
      log.write(line + '\n');
      if (value.chunk !== undefined) chunks.push({at: performance.now(), chunk: value.chunk});
      if (value.done) { report = value; child.stdin.end(); }
      deliver(value);
    }
  });
  const exited = new Promise((resolve, reject) => {
    child.on('error', reject);
    child.on('exit', code => { if (code) reject(new Error('native host exit ' + code)); else resolve(); });
  });
  return {
    next: () => queue.length ? Promise.resolve(queue.shift()) : new Promise(resolve => pending.push(resolve)),
    cancel: () => { if (!child.stdin.writableEnded) child.stdin.end('x'); },
    close: async () => { if (!child.stdin.writableEnded) child.stdin.end('x'); await exited; log.end(); },
    chunks,
    get report() { return report; },
  };
}

async function scenario(browser, mode, base) {
  const context = await browser.newContext({viewport: {width: 1280, height: 800}});
  const page = await context.newPage();
  page.setDefaultTimeout(10000);
  const errors = [], unexpected = [];
  let session, calls = 0, cancelCalls = 0;
  page.on('pageerror', error => errors.push(error.message));
  try {
    await context.route('**/*', async route => {
      const url = new URL(route.request().url());
      if (url.origin !== 'https://stream.invalid') { unexpected.push(url.href); return route.abort(); }
      if (url.pathname === '/install') return route.fulfill({contentType: 'text/html', body: '<!doctype html><title>Installing finite stream proof</title>'});
      if (url.pathname === '/sw.js') return route.fulfill({contentType: 'application/javascript', body: worker});
      if (url.pathname === '/__next') {
        session ??= native(mode, base);
        return route.fulfill({contentType: 'application/json', body: JSON.stringify(await session.next())});
      }
      if (url.pathname === '/__cancel') { cancelCalls++; session?.cancel(); return route.fulfill({status: 204, body: ''}); }
      if (url.pathname === '/resource') { calls++; return route.fulfill({body: 'DUPLICATE RESOURCE FETCH'}); }
      const file = path.basename(url.pathname);
      if (['start.js', 'stream.js', 'stream_bg.wasm', 'stream-status.js'].includes(file)) {
        return route.fulfill({path: path.join(out, 'assets', file), contentType: file.endsWith('.wasm') ? 'application/wasm' : 'application/javascript'});
      }
      if (url.pathname === '/favicon.ico') return route.fulfill({status: 204, body: ''});
      unexpected.push(url.href); return route.abort();
    });
    await page.goto('https://stream.invalid/install');
    await page.evaluate(async () => { await navigator.serviceWorker.register('/sw.js'); await navigator.serviceWorker.ready; });
    const token = `${mode}-${base ? 'mounted' : 'root'}`;
    await page.goto(`https://stream.invalid${base}/stream?token=${token}`, {waitUntil: 'commit'});
    await page.locator('#shell').waitFor({state: 'visible'});
    assert.equal(await page.locator('#details').count(), 0, 'delayed details were already present at shell delivery');
    await page.screenshot({path: path.join(out, `${token}-shell.png`), fullPage: true});
    if (mode === 'success') {
      await page.locator('html[data-hydrated=true]').waitFor();
      assert.match(await page.locator('#details').textContent(), /<script>window.hostile=true<\/script>/);
      assert.equal(await page.evaluate(() => window.hostile), undefined);
      assert.equal(calls, 0, 'hydration fetched the initial Resource again');
      assert.equal(await page.evaluate(() => document.getElementById('app').innerHTML === window.streamBeforeHydrate), true, 'hydration repaired the streamed DOM');
      assert.equal(await page.evaluate(() => document.getElementById('counter') === window.streamInitialButton), true, 'hydration replaced the initial button');
      await page.locator('#counter').click();
      assert.equal(await page.locator('#counter').textContent(), 'Clicks: 1');
      assert.equal(await page.locator('#stackpress-stream-recovery').isVisible(), false);
      await page.screenshot({path: path.join(out, `${token}-settled.png`), fullPage: true});
      await page.setViewportSize({width: 375, height: 750});
      await page.screenshot({path: path.join(out, `${token}-375.png`), fullPage: true});
    } else if (mode === 'late') {
      await page.locator('#stackpress-stream-recovery[role=alert]').waitFor();
      assert.equal(await page.locator('#details').count(), 0);
      assert.equal(await page.locator('html').getAttribute('data-hydrated'), null);
      assert.match(await page.locator('#stackpress-stream-recovery').textContent(), /interrupted|didn't finish|hasn't finished/);
      await page.screenshot({path: path.join(out, `${token}-recovery.png`), fullPage: true});
    } else {
      // This is an actual browser navigation cancelling the worker ReadableStream.
      await page.goto('https://stream.invalid/install');
      await page.waitForTimeout(150);
      assert.ok(cancelCalls > 0, 'navigation never cancelled the worker body');
    }
    await session.close();
    assert.equal(session.report.released, true);
    assert.equal(session.report.hooks, 1);
    assert.equal(session.report.errors, 0);
    if (mode === 'success') {
      assert.match(session.report.head, /^HTTP\/1\.1 200/);
      assert.equal(session.report.delivery, 'Some(Completed)');
      const shell = session.chunks.find(value => value.chunk.includes('id="shell"'));
      const details = session.chunks.find(value => value.chunk.includes('id="details"'));
      assert.ok(details.at - shell.at > 500, 'native delayed section did not follow useful shell');
    }
    assert.deepEqual(errors, []);
    assert.deepEqual(unexpected, []);
    return {mode, base, calls, cancelCalls, chunks: session.chunks.length, report: session.report};
  } finally {
    if (session) await session.close();
    // Clear the ephemeral origin's registrations even if a proof assertion fails.
    await page.evaluate(async () => { for (const registration of await navigator.serviceWorker.getRegistrations()) await registration.unregister(); }).catch(() => {});
    await context.close();
  }
}

/** Run the existing progressive-render matrix when invoked as the entry point. */
async function main() {
  const browser = await chromium.launch({headless: true});
  const results = [];
  try {
    for (const base of ['', '/demo']) {
      for (const mode of ['success', 'late', 'pending']) results.push(await scenario(browser, mode, base));
    }
    fs.writeFileSync(path.join(out, 'browser.json'), JSON.stringify({version: browser.version(), results}, null, 2));
    console.log('PASS', results.length, 'Chromium streamed SSR cases');
  } finally { await browser.close(); }
}

// The M0 status proof reuses only the native transport and worker framing.
module.exports = {native, worker};
if (require.main === module) {
  main().catch(error => { console.error(error); process.exitCode = 1; });
}
