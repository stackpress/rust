// Finite .invalid interception of native SSR and rebuilt WASM; no socket listener.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
if (!process.env.ADMIN_PLAYWRIGHT) throw Error('ADMIN_PLAYWRIGHT must name the installed module');
const { chromium } = require(process.env.ADMIN_PLAYWRIGHT);
const output = '/private/tmp/s8-admin';

(async () => {
  const browser = await chromium.launch({ headless: true });
  const results = [];
  fs.mkdirSync(path.join(output, 'screenshots'), { recursive: true });
  try {
    for (const variant of ['base', 'plugin', 'disabled']) {
      for (const mode of variant === 'base' ? ['hydrate', 'csr'] : ['hydrate']) {
        const context = await browser.newContext({ viewport: { width: 1280, height: 1000 } });
        const page = await context.newPage();
        page.setDefaultTimeout(10000);
        const errors = [];
        const unexpected = [];
        page.on('pageerror', error => errors.push(error.message));
        page.on('console', message => { if (message.type() === 'error' || /hydration.*(mismatch|error)/i.test(message.text())) errors.push(message.text()); });
        let release;
        const boot = new Promise(resolve => { release = resolve; });
        await page.route('**/*', async route => {
          const url = new URL(route.request().url());
          if (url.origin !== 'https://admin.invalid') { unexpected.push(url.href); return route.abort(); }
          if (url.pathname.startsWith('/assets/')) {
            if (url.pathname === '/assets/start.js') await boot;
            const assets = path.join(output, 'assets', variant, mode);
            const file = path.resolve(assets, url.pathname.slice('/assets/'.length));
            if (!file.startsWith(assets + path.sep)) throw Error('asset escaped output');
            return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
          }
          const body = cp.execFileSync(path.join(output, `${variant}-web`), mode === 'csr' ? ['--csr'] : [], { encoding: 'utf8', timeout: 10000 });
          return route.fulfill({ body, contentType: 'text/html' });
        });
        try {
          await page.goto('https://admin.invalid/', { waitUntil: 'commit' });
          let before;
          if (mode === 'hydrate') {
            await page.locator('#task-field-title').waitFor();
            before = await page.locator('#app').innerHTML();
            assert.equal(await page.locator('#task-field-done').isChecked(), false);
            assert.equal(await page.locator('[data-admin-plugin]').count() > 0, variant === 'plugin');
            await page.evaluate(() => { window.beforeNode = document.querySelector('#task-field-title'); });
          }
          release();
          await page.waitForFunction(() => window.adminReady === true);
          if (mode === 'hydrate') {
            assert.equal(await page.locator('#app').innerHTML(), before, 'identical initial parsed SSR/hydrated DOM');
            assert.equal(await page.evaluate(() => window.beforeNode === document.querySelector('#task-field-title')), true, 'hydration preserves native node');
          }
          assert.equal(await page.locator('[data-admin-plugin]').count() > 0, variant === 'plugin');
          if (variant === 'plugin') {
            assert.equal(await page.locator('label[for="profile-field-name"]').textContent(), 'Plugin field label');
            assert.equal(await page.locator('#profile-field-name').getAttribute('placeholder'), 'Plugin fixed hint');
          } else {
            assert.equal(await page.locator('label[for="profile-field-name"]').textContent(), 'Name </script> 🦀');
            assert.equal(await page.locator('#profile-field-name').getAttribute('placeholder'), 'Idea hint');
          }
          assert.equal(await page.evaluate(() => typeof window.pwned), 'undefined');
          const ids = await page.locator('[id]').evaluateAll(nodes => nodes.map(node => node.id));
          assert.equal(new Set(ids).size, ids.length, 'instance/role/field IDs are unique');
          assert.equal(await page.locator('label[for="task-field-title"]').count(), 1);
          assert.equal(await page.getByRole('form').count(), 3, 'one field form per workspace model');
          assert.equal(await page.getByRole('list').count(), 3, 'real list structure per model');
          assert.equal(await page.locator('#proof-field-text').inputValue(), 'fixture default');
          assert.equal(await page.locator('#proof-field-flag').isChecked(), false);
          assert.equal(await page.locator('#role-proof').textContent().then(text => text.includes('0')), true);
          assert.equal(await page.locator('[id*="sensitive"], [id*="hostOnly"]').count(), 0);
          assert.equal(await page.locator('#app').innerHTML().then(text => text.includes('ADMIN_PRIVATE_DEFAULT_SENTINEL')), false);
          await page.locator('#proof-field-text-presence').selectOption('null');
          await page.waitForFunction(() => JSON.parse(document.querySelector('#proof-draft').textContent).text === null);
          await page.locator('#proof-field-text').fill('');
          await page.locator('#proof-field-text').fill('edited');
          await page.locator('#proof-field-text').fill('');
          await page.waitForFunction(() => JSON.parse(document.querySelector('#proof-draft').textContent).text === '');
          await page.locator('#task-field-title').fill('Edited in Chromium');
          await page.waitForFunction(() => document.querySelector('#task-draft').textContent.includes('Edited in Chromium'));
          await page.locator('#task-field-done').check();
          await page.waitForFunction(() => JSON.parse(document.querySelector('#task-draft').textContent).done === true);
          await page.locator('#task-field-done').uncheck();
          await page.waitForFunction(() => JSON.parse(document.querySelector('#task-draft').textContent).done === false);
          await page.locator('#task-field-title-presence').selectOption('missing');
          await page.waitForFunction(() => !('title' in JSON.parse(document.querySelector('#task-draft').textContent)));
          await page.locator('#task-field-title').fill('');
          await page.locator('#task-field-title').fill('restored');
          await page.waitForFunction(() => JSON.parse(document.querySelector('#task-draft').textContent).title === 'restored');
          await page.screenshot({ path: path.join(output, 'screenshots', `${variant}-${mode}.png`), fullPage: true });
          assert.deepEqual(errors, []); assert.deepEqual(unexpected, []);
          await page.evaluate(() => window.adminDispose());
          assert.equal(await page.locator('#app').textContent(), '');
          results.push({ variant, mode, ssrHydrationParity: mode === 'hydrate', uniqueIds: ids.length, errors, unexpected });
        } finally { release(); await context.close(); }
      }
    }
    fs.writeFileSync(path.join(output, 'browser.json'), JSON.stringify(results, null, 2));
    console.log(JSON.stringify(results));
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
