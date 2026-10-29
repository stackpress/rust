// Real browser interactions over finite intercepted artifacts; no application host/listener.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.INDEPENDENT_PLAYWRIGHT);
const output = process.env.INDEPENDENT_OUTPUT_DIR || '/private/tmp/leptos-independent';
const records = [];
const text = 'Resolved </script><script>globalThis.independentPwned=1</script>&雪';
(async () => {
  const browser = await chromium.launch({headless:true});
  const screenshots = path.join(output, 'screenshots'); fs.mkdirSync(screenshots,{recursive:true});
  try {
    for (const variant of ['base','helpers']) for (const mode of ['hydrate','csr']) {
      for (const [theme,language] of [['light','en'],['dark','fr']]) for (const width of [1280,375]) {
        const tag = `${variant}-${mode}-${theme}-${language}-${width}`;
        const context = await browser.newContext({viewport:{width,height:900}});
        try {
          const page = await context.newPage(); page.setDefaultTimeout(10000);
          const errors = [], unexpected = []; let release;
          const boot = new Promise(resolve => {release=resolve;});
          page.on('pageerror',e=>errors.push(e.message));
          page.on('console',m=>{if(m.type()==='error'||/hydration.*(mismatch|error)/i.test(m.text())) errors.push(m.text());});
          await page.route('**/*',async route=>{
            const url = new URL(route.request().url());
            if (url.origin !== 'https://independent.invalid') {unexpected.push(url.href);return route.abort();}
            if (url.pathname.startsWith('/assets/')) {
              const assets = path.resolve(output,variant,mode);
              const file = path.resolve(assets,url.pathname.slice('/assets/'.length));
              assert(file.startsWith(assets+path.sep),'artifact must remain in its mode');
              if (file.endsWith('/start.js')) await boot;
              return route.fulfill({path:file,contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript'});
            }
            assert.equal(url.pathname,'/');
            return route.fulfill({path:path.join(output,variant,`${mode}-${theme}-${language}.html`),contentType:'text/html'});
          });
          await page.goto('https://independent.invalid/',{waitUntil:'commit'});
          if (mode==='hydrate') {
            await page.locator('#independent-draft').waitFor();
            assert.equal(await page.locator('h1').textContent(),language==='en'?'Welcome':'Bienvenue');
            assert.equal(await page.locator('.sp-root').getAttribute('data-theme'),theme);
            await page.locator('#independent-draft').fill('Early < & 雪');
            await page.evaluate(()=>{
              window.beforeInput=document.querySelector('#independent-draft');
              window.beforeHeading=document.querySelector('h1');
              window.beforeInput.focus();window.beforeInput.setSelectionRange(3,7);
            });
          }
          release(); await page.waitForFunction(()=>window.independentReady===true).catch(async error=>{
            fs.writeFileSync(path.join(output,'browser-failure.json'),JSON.stringify({tag,errors,unexpected,body:await page.locator('body').innerText()},null,2));
            console.error('browser boot diagnostics',JSON.stringify({tag,errors,unexpected}));throw error;
          });
          if(mode==='hydrate') {
            assert(await page.evaluate(()=>window.beforeInput===document.querySelector('#independent-draft')&&window.beforeHeading===document.querySelector('h1')));
            assert.equal(await page.locator('#independent-draft').inputValue(),'Early < & 雪');
            assert(await page.evaluate(()=>document.activeElement===window.beforeInput&&window.beforeInput.selectionStart===3&&window.beforeInput.selectionEnd===7));
          }
          await page.locator('#echo-draft').click();
          assert.equal(await page.locator('#draft-value').textContent(),mode==='hydrate'?'Early < & 雪':'Seeded draft');
          await page.locator('#increment').click({clickCount:3});
          assert.equal(await page.locator('#count').textContent(),'3');
          await page.locator('#theme').click();
          assert.equal(await page.locator('.sp-root').getAttribute('data-theme'),theme==='light'?'dark':'light');
          await page.locator('#language').click();
          assert.equal(await page.locator('h1').textContent(),language==='en'?'Bienvenue':'Welcome');
          await page.locator('#notify').click();
          assert((await page.locator('.sp-notices [role="status"]').textContent()).includes('Action completed'));
          await page.getByRole('button',{name:'Dismiss notification'}).click();
          assert.equal(await page.locator('.sp-notices [role="status"]').count(),0);
          await page.locator('#open-popover').click();
          await page.waitForFunction(()=>document.querySelector('#independent-popover').matches(':popover-open'));
          assert.equal(await page.locator('#close-popover').evaluate(el=>el===document.activeElement),true);
          await page.keyboard.press('Escape');
          await page.waitForFunction(()=>!document.querySelector('#independent-popover').matches(':popover-open'));
          assert.equal(await page.locator('#open-popover').evaluate(el=>el===document.activeElement),true);
          if(variant==='helpers') {
            await page.waitForFunction(expected=>document.querySelector('#resource')?.textContent===expected,text);
            assert.equal(await page.evaluate(()=>window.independentLoads||0),mode==='hydrate'?0:1);
            assert.equal(await page.evaluate(()=>window.independentPwned),undefined);
            assert.equal(await page.locator('#rich .sp-rich script').count(),0);
            assert.equal(await page.locator('#rich strong').textContent(),'Checked rich sample');
            const input=page.locator('#independent-amount');
            await input.fill('12.345'); await input.press('Enter');
            assert.equal(await page.locator('#committed').textContent(),'12.34');
            await input.fill('broken');await input.blur();
            assert.equal(await input.inputValue(),'broken');
            assert.equal(await page.locator('#committed').textContent(),'12.34');
            await input.press('Escape');assert.equal(await input.inputValue(),'12.34');
            // Real DOM events enter the same IME adapter; Enter during composition cannot commit.
            await input.evaluate(el=>{
              el.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));
              el.value='7.';el.dispatchEvent(new InputEvent('input',{bubbles:true,isComposing:true}));
              el.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,isComposing:true}));
            });
            assert.equal(await page.locator('#committed').textContent(),'12.34');
            await input.evaluate(el=>{
              el.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'7.25'}));
              el.value='7.25';el.setSelectionRange(4,4);
              el.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'7.25'}));
              el.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:'7.25'}));
            });
            await page.evaluate(()=>new Promise(resolve=>setTimeout(resolve,0)));
            await input.press('Enter');assert.equal(await page.locator('#committed').textContent(),'7.25');
          }
          await page.screenshot({path:path.join(screenshots,tag+'.png'),fullPage:true});
          // Disposing the owned root removes handlers and permits a clean CSR root in place.
          await page.evaluate(()=>{window.independentDispose();window.independentDispose();});
          if(mode==='csr') {
            assert.equal(await page.locator('#increment').count(),0);
            await page.evaluate(()=>{window.independentDispose=window.independentMount('light','en','[]');});
            assert.equal(await page.locator('#count').textContent(),'0');
            assert.equal(await page.locator('.sp-root').count(),1);
            await page.locator('#increment').click();assert.equal(await page.locator('#count').textContent(),'1');
            await page.evaluate(()=>window.independentDispose());
          }
          assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
          records.push({variant,mode,theme,language,width,identity:mode==='hydrate',earlyDraftFocus:mode==='hydrate',dispose:true,errors,unexpected});
          console.log('PASS '+tag);
        } finally {await context.close();}
      }
    }
    fs.writeFileSync(path.join(output,'browser.json'),JSON.stringify({listener:false,chromium:browser.version(),records},null,2));
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
