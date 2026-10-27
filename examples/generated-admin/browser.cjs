// Finite .invalid interception of native SSR and rebuilt WASM; no socket listener.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
if (!process.env.ADMIN_PLAYWRIGHT) throw Error('ADMIN_PLAYWRIGHT must name the installed module');
const { chromium } = require(process.env.ADMIN_PLAYWRIGHT);
const output = process.env.ADMIN_BROWSER_OUTPUT || path.join(process.env.ADMIN_PROOF_ROOT || '/private/tmp/s8-admin','roles');

// Exercise the adapters' actual draft bridge, including controller-originated mutations.
async function catalog(page) {
  const draft = () => page.locator('#proof-draft').evaluate(node => JSON.parse(node.textContent));
  const settle = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  assert.equal((await draft()).score, 0);
  await page.locator('#proof-field-zero-0').press('End'); await settle();
  assert.equal((await draft()).zero, 10);
  await page.locator('#proof-field-zero-0').press('Home'); await settle();
  assert.equal((await draft()).zero, 0);
  assert.equal(await page.locator('#proof-field-zero-presence').count(),0,'nonnullable creation control hides presence');
  await page.locator('#proof-filter-text-presence').selectOption('missing'); await settle();
  assert.equal(await page.locator('#proof-filter-text-presence').inputValue(),'missing','filters retain explicit omission');
  // A native input event at the same displayed number retains concrete zero.
  await page.locator('#proof-field-zero-0').evaluate(node => node.dispatchEvent(new Event('input',{bubbles:true}))); await settle();
  assert.equal((await draft()).zero, 0);
  await page.locator('#proof-field-zero-0').press('ArrowRight'); await settle();
  assert.equal((await draft()).zero, 1);
  await page.locator('#proof-field-score-5').check(); await settle();
  assert.equal((await draft()).score, 5);
  await page.locator('.sp-rating-input').getByRole('button', {name:'Clear rating'}).click(); await settle();
  assert.equal('score' in await draft(), false);
  await page.locator('#proof-field-score-presence').selectOption('null'); await settle();
  assert.equal((await draft()).score, null);
  await page.locator('.sp-rating-input').getByRole('button', {name:'Clear rating'}).click(); await settle();
  assert.equal('score' in await draft(), false, 'Clear while visually unrated selects omission');
  await page.locator('#proof-field-score-0').check(); await settle();
  assert.equal((await draft()).score, 0);
  await page.locator('#proof-field-slug').fill('Hello World!'); await settle();
  assert.equal((await draft()).slug, 'Hello World!', 'typing is verbatim');
  await page.getByRole('button',{name:'Apply slug',exact:true}).click(); await settle();
  assert.equal((await draft()).slug, 'hello-world');
  await page.locator('#proof-field-slug-presence').selectOption('null'); await settle();
  assert.equal((await draft()).slug, null, 'clearing the slug presentation does not echo an empty string');
  assert.equal(await page.locator('#proof-field-slug').inputValue(), '');
  await page.locator('#proof-field-rows-1-value').fill('</script> "quotes" 🦀');
  await page.locator('#proof-field-rows-1-value').evaluate(node => { window.retainedRow = node; });
  await page.locator('#proof-field-rows-1-down').click(); await settle();
  assert.deepEqual((await draft()).rows, ['0','</script> "quotes" 🦀']);
  assert.equal(await page.locator('#proof-field-rows-1-value').evaluate(node => node===window.retainedRow), true);
  await page.locator('#proof-field-rows-2-remove').click();
  await page.locator('#proof-field-rows-add').click(); await settle();
  assert.deepEqual((await draft()).rows, ['</script> "quotes" 🦀','']);
  assert.equal(await page.locator('#proof-field-rows-3-value').count(), 1, 'removed identities are never reused');
  await page.locator('#proof-field-rows-presence').selectOption('null'); await settle();
  assert.equal((await draft()).rows, null, 'reset does not echo an empty array');
  await page.locator('#proof-field-rows-add').click(); await settle();
  assert.deepEqual((await draft()).rows, ['']);
  await page.locator('#proof-field-metadata-1-value').fill(''); await settle();
  assert.deepEqual((await draft()).metadata, {initial:''});
  await page.locator('#proof-field-metadata-add').click(); await settle();
  const accepted = (await draft()).metadata;
  await page.locator('#proof-field-metadata-add').click(); await settle();
  assert.deepEqual((await draft()).metadata, accepted, 'duplicate keys do not overwrite a value');
  assert.equal(await page.locator('#proof-field-metadata-error').textContent(), 'Metadata keys must be unique');
  await page.locator('#proof-field-metadata-3-key').fill('new'); await settle();
  assert.deepEqual((await draft()).metadata, {initial:'','':'',new:''});
  assert.equal(await page.locator('#proof-field-metadata-error').textContent(), '');
  await page.locator('#project-field-state').fill('cl');
  await page.locator('#project-field-state').press('ArrowDown');
  await page.locator('#project-field-state').press('Enter'); await settle();
  assert.equal(await page.locator('#project-draft').evaluate(node => JSON.parse(node.textContent).state), 'closed');
  assert.equal(await page.locator('#project-field-state').getAttribute('aria-expanded'), 'false');
  await page.locator('#project-field-tags-1-value').fill(''); await settle();
  assert.deepEqual(await page.locator('#project-draft').evaluate(node => JSON.parse(node.textContent).tags),['','0']);
  assert.equal(await page.locator('table caption').textContent(),'Task');
  assert.deepEqual(await page.locator('thead th[scope=col]').allTextContents(),['title','done','note','priority']);
  assert.equal(await page.locator('tbody tr').count(),1);
  assert.equal(await page.evaluate(() => typeof window.pwned), 'undefined');
}

(async () => {
  const browser = await chromium.launch({ headless: true });
  const results = [];
  fs.mkdirSync(path.join(output, 'screenshots'), { recursive: true });
  let timer;
  try {
    await Promise.race([(async () => {
    for (const variant of ['base', 'plugin', 'disabled']) {
      for (const mode of ['hydrate', 'csr']) {
       for (const mount of ['/', '/admin/']) {
        for (const layout of [{width:1280,zoom:1},{width:375,zoom:2}]) {
        const context = await browser.newContext({ viewport: { width: layout.width, height: 1000 } });
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
          if (url.pathname.startsWith(mount+'assets/')) {
            if (url.pathname === mount+'assets/start.js') await boot;
            const assets = path.join(output, 'assets', variant, mode);
            const file = path.resolve(assets, url.pathname.slice((mount+'assets/').length));
            if (!file.startsWith(assets + path.sep)) throw Error('asset escaped output');
            return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
          }
          if (url.pathname !== mount) { unexpected.push(url.href); return route.abort(); }
          const body = cp.execFileSync(path.join(output, `${variant}-web`), mode === 'csr' ? ['--csr'] : [], { encoding: 'utf8', timeout: 10000 });
          return route.fulfill({ body, contentType: 'text/html' });
        });
        try {
          await page.goto('https://admin.invalid'+mount, { waitUntil: 'commit' });
          await page.locator('#app').waitFor({state:'attached'});
          await page.evaluate(zoom => { document.body.style.zoom=String(zoom); },layout.zoom);
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
            assert.equal(await page.locator('[data-admin-plugin="tags"]').count(), 2, 'field and span aliases use the replaced tags definition');
            assert.equal(await page.locator('#project-field-tags legend').textContent(),'Plugin tags label');
          } else {
            assert.equal(await page.locator('label[for="profile-field-name"]').textContent(), 'Name </script> 🦀');
            assert.equal(await page.locator('#profile-field-name').getAttribute('placeholder'), 'Idea hint');
          }
          assert.equal(await page.evaluate(() => typeof window.pwned), 'undefined');
          const ids = await page.locator('[id]').evaluateAll(nodes => nodes.map(node => node.id));
          assert.equal(new Set(ids).size, ids.length, 'instance/role/field IDs are unique');
          assert.equal(await page.locator('label[for="task-field-title"]').count(), 1);
          assert.equal(await page.getByRole('form').count(), 3, 'one field form per workspace model');
          assert.equal(await page.locator('ul[aria-label$=" list"]').count(), 3, 'real list structure per model');
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
          assert.equal(await page.locator('#task-field-title-presence').count(),0,'required create fields have no value-state selector');
          await page.locator('#task-field-title').fill('');
          await page.locator('#task-field-title').fill('restored');
          await page.waitForFunction(() => JSON.parse(document.querySelector('#task-draft').textContent).title === 'restored');
          await catalog(page);
          const geometry=await page.evaluate(() => ({width:innerWidth,scroll:document.documentElement.scrollWidth,overflow:[...document.querySelectorAll('body *')].filter(node=>node.getBoundingClientRect().right>innerWidth+1).map(node=>({tag:node.tagName,id:node.id,class:node.className,right:node.getBoundingClientRect().right,width:node.getBoundingClientRect().width})).slice(-20)}));
          await page.screenshot({ path: path.join(output, 'screenshots', `${variant}-${mode}-${mount==='/'?'root':'admin'}-${layout.width}-${layout.zoom}.png`), fullPage: true });
          assert.ok(geometry.scroll <= geometry.width+1, JSON.stringify(geometry));
          assert.deepEqual(errors, []); assert.deepEqual(unexpected, []);
          await page.evaluate(() => window.adminDispose());
          assert.equal(await page.locator('#app').textContent(), '');
          results.push({ variant, mode, mount, layout, geometry, ssrHydrationParity: mode === 'hydrate', uniqueIds: ids.length, errors, unexpected });
        } finally { release(); await context.close(); }
      }
      }
      }
    }
    })(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('inner role-proof deadline')),105000);})]);
    fs.writeFileSync(path.join(output, 'browser.json'), JSON.stringify(results, null, 2));
    console.log(JSON.stringify(results));
  } finally { clearTimeout(timer); await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
