// Real Chromium checks the optional native CSR boundary with a closed fixture bridge.
// Native file authority is proved separately by Rust tests and the installed iOS app.
const assert = require('node:assert/strict');
const cp = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

// A parent kills and reaps the whole finite browser child on a hang.
if (!process.argv.includes('--child')) {
  const child = cp.spawnSync(process.execPath, [__filename, '--child'], {
    env: process.env, stdio: 'inherit', timeout: 120000, killSignal: 'SIGKILL'
  });
  if (child.error) throw child.error;
  process.exit(child.status ?? 1);
}
const { chromium } = require(process.env.PROFILE_UI_PLAYWRIGHT);
const root = path.resolve(__dirname, '../../../..');
const assets = path.join(root, 'examples/profile-ui/dist/csr');
const logs = process.env.PROFILE_UI_IOS_LOGS;
if (!logs) throw Error('supply PROFILE_UI_IOS_LOGS');

(async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    for (const os of ['light', 'dark']) for (const choice of [null, 'light', 'dark']) {
      const color = choice || os;
      const page = await browser.newPage({ colorScheme: os, viewport: { width: 1000, height: 900 } });
      page.setDefaultTimeout(5000);
      const errors = [];
      page.on('pageerror', error => errors.push(String(error)));
      await page.addInitScript(({choice}) => {
        // The fixture accepts only the four named commands; it grants no generic API.
        const state = window.preferenceProof = { calls: [], hold: false, reject: false };
        let preferences = { locale: 'fr', theme: choice };
        state.queries=0;
        const match=window.matchMedia.bind(window);
        window.matchMedia=value => {if(value==='(prefers-color-scheme: dark)')state.queries++;return match(value);};
        const observer = new MutationObserver(() => {
          const root = document.querySelector('.sp-root');
          if (root && !state.first) state.first = { heading: root.querySelector('h1').textContent, theme: root.dataset.theme };
        });
        observer.observe(document, { childList: true, subtree: true });
        window.__TAURI__ = { core: { invoke: async (command, args) => {
          if (command === 'profile_search') return JSON.stringify({ revision: 1, rows: [{ id: 'abc123', name: 'Public fixture' }], total: 1 });
          if (command === 'cancel_profile_search') return;
          if (command === 'load_preferences') {
            const assertNoRoot = document.querySelector('.sp-root');
            state.rootBeforeLoad = Boolean(assertNoRoot);
            await new Promise(resolve=>setTimeout(resolve,100));
            return JSON.stringify({ revision: 1, preferences, recovered: false });
          }
          if (command !== 'update_preferences') throw Error('denied');
          const request = JSON.parse(args.json);
          state.calls.push(request);
          if (state.reject) throw Error('preferences-unavailable');
          if (state.hold) await new Promise(resolve => { state.release = resolve; });
          preferences = request.preferences;
          return JSON.stringify({ revision: 1, preferences, recovered: false });
        } } };
      }, {choice});
      await page.route('**/*', route => {
        const url = new URL(route.request().url());
        if (url.origin !== 'https://native-preferences.invalid') return route.abort();
        const relative = url.pathname === '/' ? 'index.html' : decodeURIComponent(url.pathname.slice(1));
        const file = relative === 'start.js' ? path.join(root, 'examples/profile-ui/native/native-bootstrap.js') : path.resolve(assets, relative);
        if (relative !== 'start.js' && !file.startsWith(assets + path.sep)) return route.abort();
        const contentType = file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : 'text/html';
        return route.fulfill({ body: fs.readFileSync(file), contentType });
      });
      await page.goto('https://native-preferences.invalid/');
      await page.waitForFunction(() => window.profileUiReady);
      assert.deepEqual(await page.evaluate(() => window.preferenceProof.first), { heading: 'Profils', theme: color });
      await page.emulateMedia({ colorScheme: os === 'dark' ? 'light' : 'dark' });
      assert.equal(await page.locator('.sp-root').getAttribute('data-theme'), color, 'OS changes do not track after mount');
      const input = page.locator('input');
      await input.fill('UNSAVED-native-hook');
      await page.evaluate(() => { window.draftReference = document.querySelector('input'); window.preferenceProof.hold = true; });
      await page.getByRole('button', { name: 'English', exact: true }).click();
      await page.waitForFunction(() => window.preferenceProof.calls.length === 1);
      assert.equal(await page.locator('html').getAttribute('lang'), 'fr');
      assert.equal(await input.inputValue(), 'UNSAVED-native-hook');
      await page.evaluate(() => window.preferenceProof.release());
      await page.waitForFunction(() => document.documentElement.lang === 'en');
      assert.equal(await page.evaluate(() => window.preferenceProof.calls[0].preferences.theme), choice);
      assert.equal(await page.locator('.sp-root').getAttribute('data-theme'), color);
      await page.evaluate(() => { window.preferenceProof.hold = false; window.preferenceProof.reject = true; });
      await page.getByRole('button', { name: color === 'dark' ? 'Light theme' : 'Dark theme', exact: true }).click();
      await page.waitForFunction(() => document.body.textContent.includes('Preferences could not be saved. Your draft is preserved.'));
      assert.equal(await input.inputValue(), 'UNSAVED-native-hook');
      assert.equal(await page.locator('.sp-root').getAttribute('data-theme'), color);
      assert.equal(await page.evaluate(() => document.querySelector('input') === window.draftReference), true);
      await page.screenshot({ path: path.join(logs, 'native-hook-' + (choice || 'unset') + '-os-' + os + '.png'), fullPage: true });
      assert.equal(await page.evaluate(() => window.preferenceProof.queries), choice ? 0 : 1, "locale save does not query OS again");
      assert.deepEqual(errors, []);
      await page.evaluate(() => window.disposeProfileUi());
      await page.close();
    }
    console.log(JSON.stringify({ initialModes: ['unset', 'light', 'dark'], cases: 6, firstRoot: 'host preferences', persistenceBeforePublication: true, failureKeepsDraftAndRoot: true, listener: false }));
  } finally {
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
