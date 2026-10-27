/* Finite real Chromium proof; routes only local fixture assets under .invalid. */
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const { chromium } = require(process.env.UI_FORMAT_PLAYWRIGHT);
const out = '/private/tmp/s8-format';
const native = JSON.parse(fs.readFileSync(path.join(out, 'native.json'), 'utf8'));
// A partial feature-matrix rebuild must never be mistaken for current browser proof.
const sizes = JSON.parse(fs.readFileSync(path.join(out, 'sizes.json'), 'utf8'));
assert.equal(sizes.length,8,'complete optional-feature matrix required');
const dependencies = JSON.parse(fs.readFileSync(path.join(out,'dependencies.json'),'utf8'));
assert.equal(dependencies.find(item=>item.name==='icu_datetime').version,'2.4.0','current aligned ICU metadata required');
assert.deepEqual(JSON.parse(fs.readFileSync(path.join(out,'lock-delta.json'),'utf8')).removed,[],'existing package versions retained');
for (const {mode,selected,files} of sizes) for (const [file,record] of Object.entries(files)) {
  assert.equal(crypto.createHash('sha256').update(fs.readFileSync(path.join(out,'assets',mode+'-'+selected,file))).digest('hex'),record.sha256,'artifact hash');
}
fs.mkdirSync(path.join(out, 'screenshots'), {recursive:true});
const css = `body{font:16px/1.5 system-ui;margin:0;color:#222;background:#fff}main{max-width:70rem;margin:auto;padding:1rem}section{border-top:1px solid #ccc;padding:.5rem 0;overflow-wrap:anywhere}h1{font-size:1.6rem}h2{font-size:1rem}pre{white-space:pre-wrap}a{color:#333}a:focus-visible{outline:3px solid #555;outline-offset:3px}p{margin:.5rem 0}`;

/** Preserve real element/attribute/text structure; ignore only empty Leptos anchors. */
function canonicalDom() {
  const root = document.querySelector('#app').cloneNode(true);
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_COMMENT);
  const empty = [];
  while (walker.nextNode()) if (walker.currentNode.data === '') empty.push(walker.currentNode);
  for (const node of empty) node.remove();
  return root.innerHTML;
}

/** Inspect inserted DOM and a second parse of every safe fragment for mutation XSS. */
function inspectPolicy() {
  const allowed = new Set(['P','A','STRONG','EM','UL','OL','LI','CODE','PRE','BLOCKQUOTE','BR']);
  const violations = [];
  for (const root of document.querySelectorAll('.sp-rich')) {
    const check = node => {
      for (const element of node.querySelectorAll('*')) {
        if (!allowed.has(element.tagName) || element.namespaceURI !== 'http://www.w3.org/1999/xhtml') violations.push(element.outerHTML);
        for (const attribute of element.attributes) {
          if (element.tagName !== 'A' || !['href','title','rel'].includes(attribute.name)) violations.push(attribute.name);
        }
        if (element.tagName === 'A') {
          if (element.hasAttribute('href') && !/^https?:\/\//i.test(element.getAttribute('href'))) violations.push(element.outerHTML);
          if (element.getAttribute('rel') !== 'noopener noreferrer') violations.push('rel');
        }
      }
    };
    check(root);
    const reparsed = document.createElement('div');
    reparsed.innerHTML = root.innerHTML;
    check(reparsed);
    if (reparsed.innerHTML !== root.innerHTML) violations.push('DOM reparse changed');
  }
  return {violations, xss: window.__xss || 0, richCount: document.querySelectorAll('.sp-rich').length};
}

/** Run modes/mounts with hostile resources failing the proof even if Chromium blocks them. */
(async () => {
  let browser;
  const results = [];
  try {
    browser = await chromium.launch({headless:true});
    for (const mode of ['hydrate','csr']) for (const mount of ['','/demo/v1']) {
      // Contrasting ambient locales/timezones cannot influence these shared formatters.
      const context = await browser.newContext({viewport:{width:1280,height:900}, locale:mode==='csr'?'fr-FR':'en-US', timezoneId:mode==='csr'?'Asia/Tokyo':'America/New_York', serviceWorkers:'block'});
      const unexpected = [], errors = [], dialogs = [];
      try {
        const page = await context.newPage();
        page.setDefaultTimeout(10000);
        page.on('pageerror', error => errors.push(error.message));
        page.on('console', message => { if (message.type()==='error' || message.type()==='warning') errors.push(message.text()); });
        page.on('dialog', async dialog => { dialogs.push(dialog.message()); await dialog.dismiss(); });
        await context.route('**/*', async route => {
          const url = new URL(route.request().url());
          const prefix = mount + '/';
          if (url.origin !== 'https://format.invalid' || !url.pathname.startsWith(prefix)) {
            unexpected.push(url.href); return route.abort();
          }
          const file = url.pathname.slice(prefix.length);
          if (file === '') {
            const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Formatting proof</title><link rel="stylesheet" href="${prefix}proof.css"></head><body><div id="app">${mode==='hydrate'?native.html:''}</div></body></html>`;
            return route.fulfill({status:200,contentType:'text/html',body:html});
          }
          if (file === 'proof.css') return route.fulfill({contentType:'text/css',body:css});
          if (file === 'proof.js' || file === 'proof_bg.wasm') return route.fulfill({contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript',body:fs.readFileSync(path.join(out,'assets',mode+'-both',file))});
          unexpected.push(url.href); return route.abort();
        });
        await page.goto('https://format.invalid'+mount+'/', {waitUntil:'load'});
        let before, original;
        if (mode==='hydrate') {
          before = await page.$eval('#app', root => root.innerHTML);
          original = await page.evaluateHandle(() => document.querySelector('#format-proof'));
          const inert = await page.evaluate(inspectPolicy);
          assert.deepEqual(inert.violations, []); assert.equal(inert.xss, 0);
        }
        // No script in the page executes until after the pre-hydration inspection.
        const snapshot = await page.evaluate(async url => {
          const module = await import(url); await module.default();
          window.proofModule = module;
          const snapshot = module.snapshot();
          window.disposeProof = module.start();
          return snapshot;
        }, 'https://format.invalid'+mount+'/proof.js');
        assert.equal(snapshot, native.snapshot, 'exact native/WASM strings');
        await page.waitForSelector('#format-proof');
        if (mode==='hydrate') {
          assert.equal(await page.$eval('#app', root => root.innerHTML), before, 'exact parsed SSR/hydrate DOM');
          assert.equal(await page.evaluate(node => node === document.querySelector('#format-proof'), original), true, 'hydration retains root');
        }
        const actualDom = await page.evaluate(canonicalDom);
        if (results.length) assert.equal(actualDom, results[0].dom, 'SSR/hydrate/CSR semantic DOM equality');
        assert.deepEqual(await page.$$eval('.formatted', nodes => nodes.map(node => node.textContent)), native.rows.filter(row=>!row.rich).map(row=>row.output));
        assert.deepEqual(await page.$$eval('.sp-rich', nodes => nodes.map(node=>{
          const clone = node.cloneNode(true);
          const walker = document.createTreeWalker(clone, NodeFilter.SHOW_COMMENT);
          const empty = [];
          while (walker.nextNode()) if (walker.currentNode.data === '') empty.push(walker.currentNode);
          for (const anchor of empty) anchor.remove();
          return clone.innerHTML;
        })), native.rows.filter(row=>row.rich).map(row=>row.output));
        const policy = await page.evaluate(inspectPolicy);
        assert.deepEqual(policy.violations, []); assert.equal(policy.xss, 0); assert.equal(policy.richCount,native.rows.filter(row=>row.rich).length);
        // Repeated WASM calls must retain byte equality independently of DOM equality.
        assert.equal(await page.evaluate(()=>window.proofModule.snapshot()), native.snapshot);
        const slug = (mount?'mounted':'root')+'-'+mode;
        for (const [width,zoom] of [[1280,1],[375,1],[375,2],[1280,2]]) {
          await page.setViewportSize({width,height:900});
          await page.evaluate(zoom => {document.body.style.zoom=String(zoom);}, zoom);
          const overflow = await page.evaluate(()=>document.documentElement.scrollWidth > innerWidth + 1);
          assert.equal(overflow,false,'page-wide overflow');
          await page.screenshot({path:path.join(out,'screenshots',`${slug}-${width}-zoom${zoom}.png`)});
          await page.locator('[data-proof="fr-decimal-1234567.125-2"]').scrollIntoViewIfNeeded();
          await page.screenshot({path:path.join(out,'screenshots',`${slug}-format-${width}-zoom${zoom}.png`)});
          await page.locator('[data-proof="fr-offset-2026-11-01T01:30:00-04:00"]').scrollIntoViewIfNeeded();
          await page.screenshot({path:path.join(out,'screenshots',`${slug}-dates-${width}-zoom${zoom}.png`)});
          await page.evaluate(()=>scrollTo(0,0));
        }
        // No CSP was needed to make the corpus inert. Give resource handlers a turn.
        await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
        assert.deepEqual(unexpected,[]); assert.deepEqual(errors,[]); assert.deepEqual(dialogs,[]);
        await page.evaluate(()=>window.disposeProof());
        assert.equal(await page.locator('#format-proof').count(),0,'owned root is disposed');
        results.push({mode,mount,dom:actualDom,rows:native.rows.length,rich:policy.richCount,unexpected,errors,dialogs});
      } finally { await context.close(); }
    }
    fs.writeFileSync(path.join(out,'browser.json'),JSON.stringify({chromium:browser.version(),playwright:require(path.join(process.env.UI_FORMAT_PLAYWRIGHT,'package.json')).version,results:results.map(({dom,...rest})=>rest)},null,2));
    console.log('PASS '+results.length+' real Chromium mode/mount cases');
  } finally { if (browser) await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1;});
