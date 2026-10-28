// M1: native duplex HTTP, real browser CSS/hydration and no listener.
const cp = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const {chromium} = require(process.env.REVIEW_PLAYWRIGHT);
const {bridge} = require('./browser.cjs');
const directory = process.env.REVIEW_DIRECTORY;
const origin = 'http://127.0.0.1:3001';

/** Read actual applied styles and overflow, rather than merely asserting classes. */
async function inspectStyles(page) {
  const result = await page.evaluate(() => {
    const root = document.querySelector('.sp-role-catalogue');
    const input = root.querySelector('input.sp-input');
    const button = root.querySelector('button:not([disabled])');
    const region = root.querySelector('[aria-label="Task table"] .sp-table-scroll');
    return {
      line: getComputedStyle(root).getPropertyValue('--sp-line').trim(),
      inputBorder: getComputedStyle(input).borderTopStyle,
      buttonBorder: getComputedStyle(button).borderTopStyle,
      overflow: getComputedStyle(region).overflowX,
      clipped: document.documentElement.scrollWidth > innerWidth + 1,
      localOverflow: region.scrollWidth > region.clientWidth,
      clientWidth: region.clientWidth, scrollWidth: region.scrollWidth
    };
  });
  assert.equal(result.line, '#c9cdd1');
  assert.equal(result.inputBorder, 'solid');
  assert.equal(result.buttonBorder, 'solid');
  assert.equal(result.overflow, 'auto');
  assert.equal(result.clipped, false);
  return result;
}

/** One compiled variant/mode/mount owns a fresh SQLite fixture and native child. */
async function run(browser, variant, mode, base) {
  const tag = `${variant}-${mode}-${base ? 'mounted' : 'root'}`;
  const scratch = fs.mkdtempSync(path.join(directory, 'review/m1-'));
  const bundle = path.join(scratch, 'bundle', variant);
  fs.cpSync(path.join(directory, 'bundle', variant), bundle, {recursive: true});
  const binary = path.join(bundle, 'slice8-review');
  cp.execFileSync(binary, ['setup', '--directory', scratch], {timeout: 30000});
  const stderr = fs.openSync(path.join(directory, 'review', `${tag}-native.log`), 'w');
  const child = cp.spawn(binary, ['pipe', '--port', '3001', '--directory', scratch,
    '--variant', variant, '--render', mode, '--base', base], {stdio: ['pipe', 'pipe', stderr]});
  const exited = new Promise(resolve => {child.once('exit', resolve); child.once('error', resolve);});
  const request = bridge(child);
  let context;
  const errors = [], unexpected = [];
  try {
    context = await browser.newContext({viewport: {width: 1280, height: 900}, serviceWorkers: 'block'});
    await context.route('**/*', async route => {
      const input = route.request(), url = new URL(input.url());
      if (url.origin !== origin) {unexpected.push(url.href); return route.abort();}
      const result = await request({method: input.method(), path: url.pathname + url.search,
        headers: await input.allHeaders(), body: (input.postDataBuffer() || Buffer.alloc(0)).toString('base64')});
      assert.equal(result.interrupted, false, 'M1 buffered requests must complete');
      const headers = Object.fromEntries(Object.entries(result.headers)
        .filter(([key]) => !['transfer-encoding', 'connection', 'content-length'].includes(key))
        .map(([key, values]) => [key, values.join('\n')]));
      return route.fulfill({status: result.status, headers, body: Buffer.from(result.body, 'base64')});
    });
    const page = await context.newPage(), panel = await context.newPage();
    for (const target of [page, panel]) {
      target.setDefaultTimeout(10000);
      target.on('pageerror', error => errors.push(error.message));
      target.on('console', message => {
        if (message.type() === 'error'
          && !/^Failed to load resource: the server responded with a status of (400|403|404|500|503)/.test(message.text())) {
          errors.push(message.text());
        }
      });
    }
    const url = suffix => origin + base + suffix;
    const action = async name => {
      await panel.goto(url('/review'));
      await panel.evaluate(async target => {await fetch(target, {method: 'POST', redirect: 'manual'});},
        url('/review/action/' + name));
    };
    // Fresh visitors have no role cookie and cannot supply authority through headers.
    assert.equal((await context.cookies()).some(cookie => cookie.name === 'slice8-review-role'), false);
    for (const model of ['profile', 'project', 'task']) {
      const response = await page.goto(url(`/generated-admin/${model}/search`));
      assert.equal(response.status(), 200);
      await page.waitForFunction(() => window.adminReady === true);
    }
    await action('denied');
    const denied = await page.goto(url('/generated-admin/profile/search'));
    assert.equal(denied.status(), 403);
    await page.getByRole('heading', {name: 'Access denied'}).waitFor();
    assert.match(await page.locator('body').textContent(), /Ask an administrator for access/);
    assert.equal(await page.locator('#admin-seed,script[type=module]').count(), 0);
    assert.equal(await page.evaluate(() => window.adminReady), undefined);
    assert.equal(await page.getByRole('link', {name: 'Return to list'}).getAttribute('href'),
      `${base}/generated-admin/profile/search`);
    await page.screenshot({path: path.join(directory, 'review', `${tag}-denied-1280.png`), fullPage: true});
    await page.setViewportSize({width: 375, height: 812});
    await page.screenshot({path: path.join(directory, 'review', `${tag}-denied-375.png`), fullPage: true});
    // API and export errors remain machine-readable even for the same browser caller.
    for (const [suffix, method] of [['/api/profile/create', 'POST'], ['/profile/export', 'GET']]) {
      const reply = await page.evaluate(async ({endpoint, method}) => {
        const response = await fetch(endpoint, {method, headers: {'x-admin-actor': 'editor',
          'x-admin-csrf': 'review-local-csrf',
          ...(method === 'POST' ? {'content-type': 'application/json'} : {})},
          ...(method === 'POST' ? {body: '{}'} : {})});
        return {status: response.status, type: response.headers.get('content-type'), body: await response.json()};
      }, {endpoint: url('/generated-admin' + suffix), method});
      assert.equal(reply.status, 403);
      assert.match(reply.type, /^application\/json/);
      assert.equal(reply.body.code, 403);
    }
    await action('editor');
    await action('failed');
    const failed = await page.goto(url('/generated-admin/profile/search'));
    assert.equal(failed.status(), 503);
    await page.getByRole('heading', {name: 'Unable to load this page'}).waitFor();
    assert.equal(await page.locator('#admin-seed,script[type=module]').count(), 0);
    await page.screenshot({path: path.join(directory, 'review', `${tag}-failed-375.png`), fullPage: true});
    if (mode === 'hydrate') await action('pause');
    await page.setViewportSize({width: 1280, height: 900});
    await page.goto(url('/generated-admin/roles'));
    let before;
    if (mode === 'hydrate') {
      await page.locator('#review-start').waitFor();
      before = await inspectStyles(page);
      await page.evaluate(() => {window.m1RoleRoot = document.querySelector('.sp-role-catalogue');});
      await page.locator('#review-start').click();
    }
    await page.waitForFunction(() => window.adminReady === true);
    if (mode === 'hydrate') assert.equal(await page.evaluate(() =>
      window.m1RoleRoot === document.querySelector('.sp-role-catalogue')), true);
    assert.equal(await page.locator('[data-admin-plugin]').count() > 0, variant === 'plugin');
    const wide = await inspectStyles(page);
    await page.screenshot({path: path.join(directory, 'review', `${tag}-roles-1280.png`)});
    await page.setViewportSize({width: 375, height: 812});
    const narrow = await inspectStyles(page);
    await page.screenshot({path: path.join(directory, 'review', `${tag}-roles-375.png`)});
    assert.equal(narrow.localOverflow, true, JSON.stringify(narrow));
    await page.keyboard.press('Tab');
    const field = page.locator('.sp-role-catalogue input.sp-input:enabled').first();
    await field.focus();
    assert.equal(await field.evaluate(element => document.activeElement === element), true);
    assert.equal(await field.evaluate(element => element.matches(':focus-visible')), true);
    assert.equal(await field.evaluate(element => getComputedStyle(element).outlineWidth), '3px');
    await page.screenshot({path: path.join(directory, 'review', `${tag}-roles-focus-375.png`)});
    assert.deepEqual(errors, []);
    assert.deepEqual(unexpected, []);
    return {variant, mode, base, before, wide, narrow, errors, unexpected};
  } finally {
    try {if (context) await context.close();}
    finally {
      child.stdin.end();
      const timer = setTimeout(() => child.kill('SIGKILL'), 5000);
      try {await exited;}
      finally {clearTimeout(timer); fs.closeSync(stderr); fs.rmSync(scratch, {recursive: true, force: true});}
    }
  }
}

/** Save each completed case so an interrupted run cannot masquerade as a full pass. */
async function main() {
  const browser = await chromium.launch({headless: true});
  const results = [];
  try {
    for (const variant of ['disabled', 'plugin']) {
      for (const mode of ['hydrate', 'csr']) {
        for (const base of ['', '/demo']) {
          results.push(await run(browser, variant, mode, base));
          fs.writeFileSync(path.join(directory, 'review/m1-browser.json'), JSON.stringify(results, null, 2));
          console.log('PASS M1', variant, mode, base || 'root');
        }
      }
    }
  } finally {await browser.close();}
}
main().catch(error => {console.error(error); process.exitCode = 1;});
