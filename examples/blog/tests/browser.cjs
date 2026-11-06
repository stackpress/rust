// Finite real-HTTP/SQLite browser proof. Cached Playwright and in-process routing only.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const readline = require('node:readline');
const { once } = require('node:events');
const blog = process.env.BLOG_BROWSER_PROJECT || path.resolve(__dirname, '..');
const mode = process.env.BLOG_BROWSER_MODE || 'hydrate';
const width = Number(process.env.BLOG_BROWSER_WIDTH || 1280);
const zoom = Number(process.env.BLOG_BROWSER_ZOOM || 1);
const publicOnly = process.argv.includes('--public-only');
const { chromium } = require(process.env.BLOG_PLAYWRIGHT);
const output = path.join(blog, '.build', 'browser-proof');
fs.mkdirSync(output, { recursive: true });
const directory = fs.mkdtempSync(path.join(output, 'fixture-'));
const fixtureArgs = [directory];
if (process.env.BLOG_BROWSER_SEED) {
  fs.copyFileSync(process.env.BLOG_BROWSER_SEED, path.join(directory, 'blog.sqlite'));
  fixtureArgs.push('--existing');
}
const child = cp.spawn(process.env.BLOG_BROWSER_PROBE || path.join(process.env.CARGO_TARGET_DIR, 'debug', 'probe'), fixtureArgs,
  { cwd: blog, stdio: ['pipe', 'pipe', 'pipe'] });
const pending = [];
const calls = [], errors = [], diagnostics = [], remoteImages = [];
let stderr = '', browser;
child.stderr.on('data', data => { stderr += data; });
readline.createInterface({ input: child.stdout }).on('line', line => {
  const next = pending.shift();
  if (next) { try { next.resolve(JSON.parse(line)); } catch (error) { next.reject(error); } }
});
child.on('exit', code => { for (const next of pending.splice(0)) next.reject(Error(`native fixture exited ${code}: ${stderr}`)); });
function native(request) {
  return new Promise((resolve, reject) => { pending.push({ resolve, reject }); child.stdin.write(JSON.stringify(request) + '\n'); });
}
/** Route every Chromium request, including redirect continuations, without a listener. */
async function attachRouting(context, page) {
  // Playwright's high-level routing may bypass redirect continuations. CDP Fetch keeps
  // each real 302/Location/Set-Cookie intact and intercepts the next request as well.
  const session = await context.newCDPSession(page);
  await session.send('Fetch.enable', { patterns: [{ urlPattern: '*' }] });
  session.on('Fetch.requestPaused', event => {
    const input = {
      url: () => event.request.url,
      method: () => event.request.method,
      resourceType: () => event.resourceType.toLowerCase(),
      allHeaders: async () => event.request.headers,
      postDataBuffer: () => event.request.postData ? Buffer.from(event.request.postData) : null
    };
    route({ request: () => input, fulfill: async result => {
      const bytes = result.path ? await fs.promises.readFile(result.path)
        : Buffer.isBuffer(result.body) ? result.body : Buffer.from(result.body || '');
      const headers = { ...result.headers };
      if (result.contentType) headers['content-type'] = result.contentType;
      const responseHeaders = Object.entries(headers).flatMap(([name, value]) =>
        name.toLowerCase() === 'set-cookie'
          ? value.split('\n').map(value => ({ name, value })) : [{ name, value }]);
      await session.send('Fetch.fulfillRequest', { requestId: event.requestId,
        responseCode: result.status || 200, responseHeaders, body: bytes.toString('base64') });
    } }).catch(async error => {
      errors.push(error.message);
      await session.send('Fetch.failRequest', { requestId: event.requestId, errorReason: 'Failed' }).catch(() => {});
    });
  });
}
async function route(request) {
  const target = new URL(request.request().url());
  if (target.origin !== 'http://blog.test') {
    // Original seed banners are external; fulfill locally rather than performing network IO.
    assert.equal(request.request().resourceType(), 'image', target.href);
    remoteImages.push(target.href);
    return request.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="600" height="260"><rect width="600" height="260" fill="#d2e6cf"/></svg>' });
  }
  const input = request.request();
  // Use the same compiled bytes directly, avoiding a 23MiB WASM's decimal JSON-array copy.
  // The real native asset allowance is asserted separately by the HTTP adapter regression.
  if (/^\/assets\/(hydrate|csr)\/(blog\.js|blog_bg\.wasm)$/.test(target.pathname)) {
    return request.fulfill({ path: path.join(blog, '.build/browser', target.pathname.slice('/assets/'.length)),
      contentType: target.pathname.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
  }
  const result = await native({ method: input.method(), path: target.pathname + target.search,
    headers: Object.entries(await input.allHeaders()), body: Array.from(input.postDataBuffer() || []) });
  calls.push({ path: target.pathname, method: input.method(), status: result.status });
  const headers = {};
  for (const [name, value] of result.headers) headers[name] = headers[name] ? headers[name] + '\n' + value : value;
  delete headers['content-length'];
  return request.fulfill({ status: result.status, headers, body: Buffer.from(result.body) });
}
function observe(page) {
  page.on('pageerror', error => errors.push(error.message));
  page.on('requestfailed', request => diagnostics.push(`${request.method()} ${request.url()}: ${request.failure()?.errorText}`));
  page.on('console', message => {
    const text = message.text();
    if (message.type() === 'error' && /^Failed to load resource: the server responded with a status of (401|404)/.test(text)) diagnostics.push(text);
    else if (message.type() === 'error' || /hydration.*(mismatch|error)|already disposed/i.test(text)) errors.push(text);
  });
}
async function ready(page) {
  await page.waitForFunction(mode => document.documentElement.dataset.blogReady === mode, mode);
  await page.evaluate(zoom => { document.body.style.zoom = String(zoom); }, zoom);
}
async function go(page, target) {
  const url = new URL(target, 'http://blog.test');
  url.searchParams.set('renderer', mode);
  const response = await page.goto(url.href);
  await ready(page);
  return response;
}
async function picture(page, name) {
  const dimensions = await page.evaluate(() => ({ width: innerWidth, scroll: document.documentElement.scrollWidth }));
  await page.screenshot({ path: path.join(output, `${mode}-${width}-${zoom}-${name}.png`), fullPage: true });
  assert.ok(dimensions.scroll <= dimensions.width + 1, `${name}: ${JSON.stringify(dimensions)}`);
}
async function main() {
  try {
    browser = await chromium.launch({ headless: true });
    // No-JS verifies real native first paint, without manufacturing or patching a seed.
    const staticContext = await browser.newContext({ javaScriptEnabled: false, viewport: { width, height: 900 } });
    const staticPage = await staticContext.newPage();
    await attachRouting(staticContext, staticPage);
    await staticPage.goto('http://blog.test/');
    assert.match(await staticPage.locator('h1').innerText(), /structured ideas/);
    await picture(staticPage, 'no-js');
    await staticContext.close();
    const context = await browser.newContext({ viewport: { width, height: 900 } });
    const page = await context.newPage();
    await attachRouting(context, page);
    observe(page);
    page.setDefaultTimeout(10000);
    await go(page, '/');
    assert.equal(await page.title(), 'The Blog');
    const seed = await page.locator('#stackpress-page').evaluate(node => JSON.parse(node.textContent));
    assert.equal(seed.data.data.length, 2);
    assert.ok(seed.data.data.every(row => row.status === 'PUBLISHED'));
    assert.equal(await page.locator('.knowledge-feature-title').innerText(), seed.data.data[0].title);
    await picture(page, 'home');
    // Native links remain reachable with a keyboard at every viewport/zoom.
    await page.keyboard.press('Tab');
    assert.equal(await page.evaluate(() => document.activeElement?.tagName), 'A');
    await go(page, '/articles/' + seed.data.data[0].slug);
    assert.equal(await page.locator('.article-title').innerText(), seed.data.data[0].title);
    assert.ok(await page.locator('.article-prose p').count() > 0);
    assert.ok(await page.locator('.article-comment-card').count() > 0);
    await picture(page, 'article');
    const missing = await go(page, '/articles/absent-browser-proof');
    assert.equal(missing.status(), 404);
    assert.equal(await page.locator('h1').innerText(), 'Not Found');
    await go(page, '/auth/signin/username');
    assert.equal(await page.locator('input[name=username]').count(), 1);
    await picture(page, 'signin');
    // Existing source theme controls exercise live hydration/CSR rather than static HTML alone.
    const toggle = page.getByRole('button', { name: /theme/i });
    if (await toggle.count()) {
      await toggle.click();
      assert.equal(await page.locator('.sp-root').getAttribute('data-theme'), 'dark');
    }
    assert.deepEqual(errors, [], errors.join('\n'));
    if (!publicOnly) await require('./flows.cjs').run(page, go, seed.data.data[0], mode, process.env.BLOG_NICKNAME === '1');
    await page.evaluate(() => { window.__blogDispose(); window.__blogDispose(); });
    assert.equal(await page.locator('#app').innerHTML(), '');
    assert.deepEqual(errors, [], errors.join('\n'));
    console.log(JSON.stringify({ mode, width, zoom, publicOnly, requests: calls.length, passed: true }));
  } finally {
    fs.writeFileSync(path.join(output, `${mode}-${width}-${zoom}-${publicOnly ? 'public' : 'full'}.json`), JSON.stringify({ mode, width, zoom, publicOnly, calls, errors, diagnostics, remoteImages, stderr }, null, 2));
    if (browser) await browser.close();
    child.stdin.end();
    if (child.exitCode === null) await once(child, 'exit');
    // The child closed its own SQLite and hooks; remove only this owned .build fixture.
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
