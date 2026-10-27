// Shared finite HTTP/SQLite harness. The parent proof driver supplies the outer watchdog.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');

module.exports = async function run(flow) {
  if (!process.env.ADMIN_PLAYWRIGHT) throw Error('ADMIN_PLAYWRIGHT is required');
  const { chromium } = require(process.env.ADMIN_PLAYWRIGHT);
  const config = JSON.parse(process.env.ADMIN_BROWSER_CASE);
  const output = process.env.ADMIN_BROWSER_OUTPUT;
  const tag = process.env.ADMIN_BROWSER_TAG;
  const artifacts = config.plugin ? path.join(output, 'plugin') : output;
  const directory = fs.mkdtempSync(path.join(output, 'browser-fixture-'));
  const database = path.join(directory, 'admin.sqlite');
  const h = { config, output, tag, actor: 'editor', faultNext: false,
    calls: [], errors: [], unexpected: [], expectedHttpDiagnostics: [] };
  let release, bootPending = true, browser, timer;
  const boot = new Promise(resolve => { release = resolve; });
  h.releaseBoot = () => { bootPending = false; release(); };
  // Hold delivery after the real finite request commits, while leaving browser controls editable.
  h.deferNext = suffix => {
    let received, release;
    const acknowledgement = new Promise(resolve => { received = resolve; });
    const delivery = new Promise(resolve => { release = resolve; });
    h.deferred = { suffix, received, delivery };
    return { acknowledgement, release };
  };
  h.native = (pathname, method = 'GET', body, extra = {}) => JSON.parse(cp.execFileSync(
    path.join(artifacts, 'admin-web'), ['--request'], {
      input: JSON.stringify({ database, mount: config.mount, actor: h.actor, path: pathname,
        method, body, csr: config.mode === 'csr', headers: { origin: 'https://admin.invalid',
          'x-admin-csrf': 'finite-session-csrf' }, ...extra }),
      encoding: 'utf8', timeout: 10000, killSignal: 'SIGKILL', maxBuffer: 8388608
    }));
  h.url = (model, action, key) => `https://admin.invalid${config.mount}${model}/${action}${key ? '?key=' + encodeURIComponent(JSON.stringify(key)) : ''}`;
  h.frame = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  h.snapshot = page => page.locator('#admin-seed').evaluate(node => JSON.parse(node.textContent));
  h.draft = page => page.locator('#admin-draft').evaluate(node => JSON.parse(node.textContent));
  h.status = (page, message) => page.waitForFunction(message => document.querySelector('#operation-status')?.textContent.includes(message), message);
  h.settled = page => page.waitForFunction(() => document.querySelector('#operation-status')?.textContent !== 'Submitting…');
  const ready = async page => {
    await page.waitForFunction(() => window.adminReady === true);
    await page.evaluate(zoom => { document.body.style.zoom = String(zoom); }, config.zoom);
  };
  h.go = async (page, target) => { await page.goto(target); await ready(page); };
  h.click = async (page, selector) => { await page.locator(selector).click(); await ready(page); };
  h.picture = async (page, name) => {
    if (config.width === 375 && await page.locator('#public-table').count()) {
      const region = page.getByRole('region', { name: 'Search results table' });
      assert.equal(await region.getAttribute('tabindex'), '0');
      assert.equal(await page.locator('.table-scroll-hint').isVisible(), true);
      assert.equal(await region.evaluate(node => {
        const needsScroll = node.scrollWidth > node.clientWidth;
        node.scrollLeft = node.scrollWidth;
        const reachable = !needsScroll || node.scrollLeft > 0;
        node.scrollLeft = 0;
        return reachable;
      }), true, 'narrow result columns remain reachable inside their labelled region');
    }
    const geometry = await page.evaluate(() => ({ width: innerWidth, scroll: document.documentElement.scrollWidth }));
    fs.mkdirSync(path.join(output, 'screenshots'), { recursive: true });
    await page.screenshot({ path: path.join(output, 'screenshots', `${tag}-${name}.png`), fullPage: true });
    assert.ok(geometry.scroll <= geometry.width + 1, JSON.stringify(geometry));
  };
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: config.width, height: 1000 } });
    const page = await context.newPage();
    page.setDefaultTimeout(10000);
    page.on('pageerror', error => h.errors.push(error.message));
    page.on('console', message => {
      if (message.type() === 'error' && /^Failed to load resource: the server responded with a status of (400|403|404|500)/.test(message.text())) h.expectedHttpDiagnostics.push(message.text());
      else if (message.type() === 'error' || /hydration.*(mismatch|error)/i.test(message.text())) h.errors.push(message.text());
    });
    await page.route('**/*', async route => {
      const request = route.request(), target = new URL(request.url());
      if (target.origin !== 'https://admin.invalid') { h.unexpected.push(target.href); return route.abort(); }
      const prefix = config.mount + 'assets/';
      if (target.pathname.startsWith(prefix)) {
        if (bootPending && target.pathname === prefix + 'start.js') await boot;
        const assets = path.join(artifacts, 'assets', config.mode);
        const file = path.resolve(assets, target.pathname.slice(prefix.length));
        assert.ok(file.startsWith(assets + path.sep));
        return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
      }
      const method = request.method(), headers = await request.allHeaders();
      const transfer = target.pathname.endsWith('/preview') || target.pathname.endsWith('/confirm');
      const body = transfer ? { csv: request.postData() || '', decode_export_guards: headers['x-admin-decode-export-guards'] === '1', confirm: headers['x-admin-confirmed'] === 'true' } : request.postData() ? JSON.parse(request.postData()) : undefined;
      h.calls.push({ method, pathname: target.pathname });
      const fault = h.faultNext && method === 'POST';
      if (fault) h.faultNext = false;
      for (const name of ['content-type', 'content-length', 'x-admin-decode-export-guards', 'x-admin-confirmed']) delete headers[name];
      // Actor/fault/database belong to the harness; browser payloads cannot select them.
      const result = h.native(target.pathname + target.search, method, body, { headers, fault });
      if (h.deferred && method === 'POST' && target.pathname.endsWith(h.deferred.suffix)) {
        const deferred = h.deferred;
        h.deferred = null;
        deferred.received(result);
        await deferred.delivery;
      }
      return route.fulfill({ status: result.status, headers: result.headers, body: result.body });
    });
    await Promise.race([flow(h, page), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error('inner browser deadline')), 105000);
    })]);
    fs.writeFileSync(path.join(output, `browser-${tag}.json`), JSON.stringify({ config,
      calls: h.calls, errors: h.errors, unexpected: h.unexpected,
      expectedHttpDiagnostics: h.expectedHttpDiagnostics, passed: true }, null, 2));
    console.log(JSON.stringify({ config, requests: h.calls.length, passed: true }));
  } finally {
    h.releaseBoot(); clearTimeout(timer);
    if (browser) await browser.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
};
