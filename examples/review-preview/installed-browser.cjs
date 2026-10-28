// Port-free packaged-byte smoke. Native/iOS behavior is verified separately.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const {chromium} = require(process.env.ADMIN_PLAYWRIGHT);

async function run() {
  const directory = path.resolve(process.argv[2]);
  const browser = await chromium.launch({headless:true});
  const results = [];
  try {
    for (const variant of ['plugin','disabled']) {
      const folder = path.join(directory,'ios-assets',variant);
      const context = await browser.newContext({viewport:{width:375,height:812}});
      const errors = [], unexpected = [];
      try {
        const page = await context.newPage();
        page.on('pageerror',error => errors.push(String(error)));
        await page.addInitScript(() => {
          window.reviewInvokes = [];
          window.__TAURI__ = {core:{invoke:async (command,args) => {
            window.reviewInvokes.push(command);
            if (command === 'profile_search' && args?.callId === 'review-denial') throw 'denied';
            throw Error('No IPC service');
          }}};
        });
        await context.route('**/*',async route => {
          const url = new URL(route.request().url());
          if (url.origin !== 'https://installed.invalid' || route.request().method() !== 'GET') {
            unexpected.push(url.href);return route.abort();
          }
          if (url.pathname === '/__controls') {
            return route.fulfill({contentType:'text/html',body:'<body><div id="app"></div><script type="module" src="/review-controls.js"></script></body>'});
          }
          const name = decodeURIComponent(url.pathname).slice(1);
          const file = path.resolve(folder,name);
          if (!file.startsWith(folder+path.sep)) {unexpected.push(name);return route.abort();}
          try {
            const body = await fs.readFile(file);
            const contentType = {'.html':'text/html','.css':'text/css','.js':'text/javascript','.wasm':'application/wasm'}[path.extname(file)];
            return route.fulfill({contentType,body});
          } catch { unexpected.push(name);return route.abort(); }
        });
        await page.goto('https://installed.invalid/admin-review.html');
        await page.waitForFunction(() => window.adminReady === true);
        assert.equal(await page.locator('label[for="profile-field-name"]').textContent(),variant === 'plugin' ? 'Plugin field label' : 'Full name');
        assert.equal(await page.locator('#admin-review-debug').getAttribute('open'),null);
        for (const id of ['proof-draft','project-draft','task-draft']) assert.equal(await page.locator('#'+id).isVisible(),false);
        // All hostile generated labels belong to the explicitly labelled safety section.
        const hostile = await page.locator('#app').evaluate(root => {
          const walker = document.createTreeWalker(root,NodeFilter.SHOW_TEXT), values = [];
          while (walker.nextNode()) if (walker.currentNode.textContent.includes('</script>'))
            values.push({text:walker.currentNode.textContent,section:walker.currentNode.parentElement.closest('section')?.id});
          return values;
        });
        assert.ok(hostile.some(label => label.text.includes('Rows </script>')));
        assert.ok(hostile.every(label => label.section === 'role-proof'));
        assert.equal(await page.locator('#role-proof h2').textContent(),'Safety samples');
        await page.locator('#profile-field-name').fill('Owner local draft');
        assert.equal(await page.locator('#profile-field-name').inputValue(),'Owner local draft');
        assert.match(await page.locator('header').textContent(),/not installed admin CRUD/);
        assert.equal(await page.locator('#admin-seed').count(),0);
        assert.deepEqual(await page.evaluate(() => window.reviewInvokes),[],'Catalog must never invoke IPC');
        await page.screenshot({path:path.join(directory,'review',`installed-${variant}-375.png`),fullPage:true});
        await page.setViewportSize({width:1280,height:900});
        await page.screenshot({path:path.join(directory,'review',`installed-${variant}-1280.png`),fullPage:true});
        // The original exact snapshots stay readable to the unchanged proof contract.
        await page.locator('#proof-field-rows-1-value').fill('Packaged exact state');
        const proof = JSON.parse(await page.locator('#proof-draft').textContent());
        assert.ok(proof.rows.includes('Packaged exact state'));
        await page.getByText('Debug state (review only)',{exact:true}).click();
        await page.locator('#admin-review-debug pre').filter({hasText:'Packaged exact state'}).waitFor();
        await page.getByText('Debug state (review only)',{exact:true}).click();
        assert.equal(await page.getByRole('link',{name:'Return to Profile UI'}).getAttribute('href'),'index.html');
        await page.evaluate(() => window.adminDispose());
        assert.equal(await page.locator('#profile-field-name').count(),0);
        await page.goto('https://installed.invalid/__controls');
        await page.getByRole('button',{name:'Check fixed denial'}).click();
        await page.getByRole('status').filter({hasText:'Denied: the fixed extra-field search'}).waitFor();
        await page.evaluate(() => {window.__TAURI__.core.invoke = () => Promise.reject('lost acknowledgement');});
        await page.getByRole('button',{name:'Check fixed denial'}).click();
        await page.getByRole('status').filter({hasText:'Denial acknowledgement unavailable; outcome uncertain'}).waitFor();
        // Closed fixed-command adapter for the browser panel only; the Rust test
        // exercises the real host ticket and cancellation future independently.
        await page.evaluate(() => {
          let reject;
          window.__TAURI__.core.invoke = (command,args) => {
            window.reviewInvokes.push([command,args]);
            if (command === 'profile_search') return new Promise((_,failure) => {reject=failure;});
            if (command === 'cancel_profile_search') {reject('cancelled');return Promise.resolve();}
            return Promise.reject('denied');
          };
        });
        await page.getByRole('button',{name:'Start cancellation check'}).click();
        await page.getByRole('button',{name:'Cancel review read'}).click();
        await page.getByRole('status').filter({hasText:'Cancelled: native read returned cancelled'}).waitFor();
        const calls = await page.evaluate(() => window.reviewInvokes);
        assert.equal(calls[1][0],'profile_search');
        assert.equal(calls[2][0],'cancel_profile_search');
        assert.equal(calls[1][1].callId,calls[2][1].callId);
        assert.equal(JSON.parse(calls[1][1].json).id,'review-wait');
        await page.evaluate(() => {
          window.__TAURI__.core.invoke = command => command === 'profile_search'
            ? new Promise(() => {}) : Promise.reject('lost acknowledgement');
        });
        await page.getByRole('button',{name:'Start cancellation check'}).click();
        await page.getByRole('button',{name:'Cancel review read'}).click();
        await page.getByRole('status').filter({hasText:'outcome uncertain'}).waitFor();
        assert.equal(await page.getByRole('button',{name:'Start cancellation check'}).isDisabled(),true);
        assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
        results.push({variant,catalog_local_draft:true,no_catalog_ipc:true,unexpected_requests:unexpected,errors,
          controls:'fixed denial, matching read/cancel IDs, uncertainty without success'});
      } finally { await context.close(); }
    }
  } finally { await browser.close(); }
  await fs.writeFile(path.join(directory,'review','installed-browser.json'),JSON.stringify({passed:true,cases:results},null,2)+'\n');
  console.log('PASS packaged CSR catalogs and fixed review controls (2 variants; no listener)');
}
run().catch(error => {console.error(error);process.exitCode=1;});
