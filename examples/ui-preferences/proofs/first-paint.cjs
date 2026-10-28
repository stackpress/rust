// Maintained port-free matrix against the actual finite example and built WASM.
const assert = require('node:assert/strict');
const cp = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require(process.env.PREFERENCES_PLAYWRIGHT);
const target = process.env.CARGO_TARGET_DIR;
const output = process.env.PREFERENCES_LOGS;
const host = path.join(target, 'debug/stackpress-example-ui-preferences-web');
const source = path.resolve(__dirname, '../../..', 'crates/framework/framework-view/src');
const asset = /BOOTSTRAP_FILE: &str = "([^"]+)"/.exec(fs.readFileSync(path.join(source, 'theme.rs'), 'utf8'))[1];
const bytes = fs.readFileSync(path.join(source, 'initial-theme.js'));
const receipts = [];

/** Run one actual HTTP adapter exchange, with a finite process deadline. */
function request(base, choice, mode) {
  const cookie = 'stackpress-locale=fr' + (choice ? `; stackpress-theme=${choice}` : '');
  const raw = cp.execFileSync(host, { input: JSON.stringify({method:'GET',path:base+'/preferences',cookie,csr:mode==='csr'}),
    encoding:'utf8', timeout:10000, env:{...process.env,PREFERENCES_BASE:base} });
  const split = raw.indexOf('\n');
  const meta = JSON.parse(raw.slice(0,split));
  assert.equal(meta.status,200);
  assert.equal(meta.headers['set-cookie'],undefined,'acquisition never writes');
  return {meta,body:raw.slice(split+1)};
}

(async () => {
  const browser = await chromium.launch({headless:true});
  try {
    const cases = [];
    for (const mode of ['hydrate','csr']) for (const base of ['', '/demo/v1']) for (const os of ['light','dark']) for (const choice of ['', 'light','dark','system']) {
      cases.push({mode,base,os,choice,bootstrap:'run'});
    }
    for (const bootstrap of ['no-js','blocked','strict-csp','missing']) for (const choice of ['', 'dark']) {
      cases.push({mode:'hydrate',base:'/demo/v1',os:'dark',choice,bootstrap});
    }
    for (const item of cases) {
      const {mode,base,os,choice,bootstrap} = item;
      const unset = !choice || choice === 'system';
      const expected = unset ? bootstrap==='run' ? os : 'light' : choice;
      const name = [mode,base?'mounted':'root',os,choice||'unset',bootstrap].join('-');
      const context = await browser.newContext({javaScriptEnabled:bootstrap!=='no-js', colorScheme:os,viewport:{width:1000,height:850}});
      const page = await context.newPage();
      page.setDefaultTimeout(10000);
      const errors = [], requests = [];
      let release;
      const gate = new Promise(resolve => { release=resolve; });
      await page.addInitScript(() => {
        window.samples=[];window.themeQueries=0;
        const query=window.matchMedia.bind(window);
        window.matchMedia=value => {if(value==='(prefers-color-scheme: dark)')window.themeQueries++;return query(value);};
        const sample=() => {if(document.querySelector('h1'))window.samples.push(getComputedStyle(document.body).backgroundColor);requestAnimationFrame(sample);};
        requestAnimationFrame(sample);
      });
      page.on('pageerror',error=>errors.push(error.message));
      page.on('console',message=>{if(/hydration.*(mismatch|error)/i.test(message.text()))errors.push(message.text());});
      const response=request(base,choice,mode);
      assert.equal(response.body.includes('data-theme-unset'),unset);
      assert.equal(response.body.includes(`<script src="${base}/assets/${asset}"></script>`),unset);
      if(bootstrap==='strict-csp')response.meta.headers['content-security-policy']=["default-src 'none'; script-src 'none'; style-src 'unsafe-inline'"];
      await page.route('**/*', async route => {
        const url = new URL(route.request().url());
        requests.push(url.pathname);
        assert.equal(url.origin,'https://preferences.invalid');
        if(url.pathname===base+'/preferences')return route.fulfill({status:200,body:response.body,headers:Object.fromEntries(Object.entries(response.meta.headers).map(([key,values])=>[key,values.join('\n')]))});
        if(url.pathname===base+'/assets/'+asset){
          if(bootstrap==='blocked')return route.abort();
          if(bootstrap==='missing')return route.fulfill({status:404,body:''});
          // Slow the classic asset: the parser cannot paint themed body content early.
          await new Promise(resolve=>setTimeout(resolve,80));
          return route.fulfill({body:bytes,contentType:'text/javascript',headers:{'cache-control':'public, max-age=31536000, immutable'}});
        }
        const relative=url.pathname.slice((base+'/assets/').length);
        if(relative==='start.js')await gate;
        const directory=path.join(output,'assets',mode),file=path.resolve(directory,relative);
        assert.equal(file.startsWith(directory+path.sep),true);
        return route.fulfill({path:file,contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript'});
      });
      try {
        await page.goto('https://preferences.invalid'+base+'/preferences',{waitUntil:'commit'});
        await page.locator('body').waitFor({state:'attached'});
        if(mode==='hydrate'){
          await page.locator('#draft').fill('Early French draft');
          await page.evaluate(()=>{window.inputBefore=document.getElementById('draft');window.inputBefore.focus();});
          assert.equal(await page.locator('h1').textContent(),'Préférences');
        }
        assert.equal(await page.locator('html').getAttribute('data-theme'),expected);
        if(mode==='hydrate')assert.equal(await page.locator('body').evaluate(node=>getComputedStyle(node).backgroundColor),expected==='dark'?'rgb(23, 23, 23)':'rgb(250, 250, 250)');
        if(name==='hydrate-mounted-dark-unset-run'||name==='hydrate-mounted-dark-unset-no-js'||name==='hydrate-mounted-dark-dark-strict-csp'){
          await page.screenshot({path:path.join(output,'screenshots','first-'+name+'-before-wasm.png'),fullPage:true});
        }
        release();
        if(bootstrap==='run'){
          await page.waitForFunction(()=>window.preferencesReady===true);
          assert.equal(await page.locator('#system').count(),0);
          if(mode==='hydrate'){
            assert.equal(await page.locator('#draft').inputValue(),'Early French draft');
            assert.equal(await page.evaluate(()=>inputBefore===document.getElementById('draft')&&document.activeElement===inputBefore),true);
          }
          await page.emulateMedia({colorScheme:os==='dark'?'light':'dark'});
          await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
          assert.equal(await page.locator('html').getAttribute('data-theme'),expected);
          assert.equal(await page.evaluate(()=>window.themeQueries),unset?1:0);
          const colors=await page.evaluate(()=>window.samples);
          assert(colors.length>0);
          assert(colors.every(color=>color===(expected==='dark'?'rgb(23, 23, 23)':'rgb(250, 250, 250)')),JSON.stringify({name,colors}));
          assert.deepEqual(errors,[]);
        }
        assert.equal((await context.cookies()).length,0,'initial acquisition never writes a cookie');
        assert.equal(requests.some(value=>value.includes('ui-preferences')),false,'no preference fetch or save');
        receipts.push({...item,expected,errors,queries:bootstrap==='run'?await page.evaluate(()=>window.themeQueries):null});
      } finally {release();await context.close();}
    }
    fs.writeFileSync(path.join(output,'first-paint.json'),JSON.stringify({listener:false,asset,bytes:bytes.length,cases:receipts},null,2)+'\n');
    console.log('PASS real example first-paint matrix: '+receipts.length+' cases; asset '+bytes.length+' bytes');
  } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
