// Listener-free browser smoke against the same native HTTP host over a persistent pipe.
const cp = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const {chromium} = require(process.env.REVIEW_PLAYWRIGHT || '/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright');
const directory = process.env.REVIEW_DIRECTORY || '/private/tmp/s8-review';
const origin = 'http://127.0.0.1:3001';

/** Frame only ASCII LF: resource text may contain Unicode line separators. */
function bridge(child) {
  let buffer = Buffer.alloc(0), pending, stopped;
  child.stdout.on('data', chunk => {
    buffer = Buffer.concat([buffer,chunk]);
    let index;
    while ((index=buffer.indexOf(10))>=0) {
      const line=buffer.subarray(0,index);buffer=buffer.subarray(index+1);
      const receiver=pending;pending=undefined;
      if(receiver) {try {receiver.resolve(JSON.parse(line));}catch(error){receiver.reject(error);}}
    }
  });
  const failed=error=>{stopped=error;pending?.reject(error);};
  child.on('error',failed);child.stdin.on('error',failed);
  child.on('exit',code=>failed(Error('Native pipe exited '+code)));
  let queue=Promise.resolve();
  return input => {
    const next=queue.then(()=>new Promise((resolve,reject)=>{
      if(stopped){reject(stopped);return;}
      const timer=setTimeout(()=>reject(Error('Native pipe deadline')),10000);
      pending={resolve:result=>{clearTimeout(timer);resolve(result);},reject:error=>{clearTimeout(timer);reject(error);}};
      child.stdin.write(JSON.stringify(input)+'\n');
    }));
    queue=next.catch(()=>{});return next;
  };
}

/** Run one compiled mode/mount/variant and always close browser/native resources. */
async function run(variant,mode,base) {
  const scratch=fs.mkdtempSync(path.join(directory,'review/smoke-'));
  fs.cpSync(path.join(directory,'bundle'),path.join(scratch,'bundle'),{recursive:true});
  const binary=path.join(scratch,'bundle',variant,'slice8-review');
  cp.execFileSync(binary,['setup','--directory',scratch],{timeout:30000});
  const stderr=fs.openSync(path.join(directory,'review',`pipe-${variant}-${mode}-${base?'mounted':'root'}.log`),'w');
  const child=cp.spawn(binary,['pipe','--port','3001','--directory',scratch,'--variant',variant,'--render',mode,'--base',base],{stdio:['pipe','pipe',stderr]});
  const exited=new Promise(resolve=>{child.once('exit',resolve);child.once('error',resolve);});
  const request=bridge(child);let browser;
  const errors=[],unexpected=[],calls=[];
  try {
    browser=await chromium.launch({headless:true,timeout:10000});
    const context=await browser.newContext({viewport:{width:1280,height:900},serviceWorkers:'block'});
    await context.route('**/*',async route=>{
      const browserRequest=route.request(),url=new URL(browserRequest.url());
      if(url.origin!==origin) {unexpected.push(url.href);return route.abort();}
      const result=await request({method:browserRequest.method(),path:url.pathname+url.search,headers:await browserRequest.allHeaders(),body:(browserRequest.postDataBuffer()||Buffer.alloc(0)).toString('base64')});
      calls.push({path:url.pathname,status:result.status,interrupted:result.interrupted});
      const headers=Object.fromEntries(Object.entries(result.headers).filter(([key])=>!['transfer-encoding','connection','content-length'].includes(key)).map(([key,values])=>[key,values.join('\n')]));
      await route.fulfill({status:result.status,headers,body:Buffer.from(result.body,'base64')});
    });
    const page=await context.newPage(),panel=await context.newPage();
    for(const target of [page,panel]){
      target.setDefaultTimeout(10000);target.setDefaultNavigationTimeout(10000);
      target.on('pageerror',error=>errors.push(error.message));
      target.on('console',message=>{if(message.type()==='error'&&!/^Failed to load resource: the server responded with a status of (400|403|404|500|503)/.test(message.text()))errors.push(message.text());});
    }
    const url=value=>origin+base+value;
    // Keep fixture POST redirects manual: intercepted navigation redirect chains may
    // escape routing. Chromium still applies the actual native Set-Cookie header.
    const action=async name=>{
      await panel.goto(url('/review'));
      await panel.evaluate(async target=>{await fetch(target,{method:'POST',redirect:'manual'});},url('/review/action/'+name));
      await panel.goto(url('/review'));
    };
    await action('editor');
    assert.equal((await context.cookies()).find(cookie=>cookie.name==='slice8-review-role')?.value,'editor');
    await panel.screenshot({path:path.join(directory,'review',`panel-${variant}-${mode}.png`),fullPage:true});
    if(mode==='hydrate')await action('pause');
    await page.goto(url('/ui-gallery/gallery'));
    await page.locator('#core-text').fill('Owner before JavaScript');
    if(mode==='hydrate'){
      assert.equal(await page.locator('#resource').count(),1);
      await page.locator('#review-start').click();
    }
    await page.waitForFunction(()=>window.galleryReady===true);
    if(mode==='hydrate')assert.equal(await page.locator('#core-text').inputValue(),'Owner before JavaScript');
    await page.locator('#opener').click();await page.locator('#nested-opener').click();await page.locator('#nested-close').click();await page.locator('#close').click();
    await action('automatic');
    await page.goto(url('/ui-preferences/preferences'));await page.waitForFunction(()=>window.preferencesReady);
    await page.locator('#draft').fill('Keep this draft');await page.locator('#french').click();
    await page.waitForFunction(()=>document.documentElement.lang==='fr');assert.equal(await page.locator('#draft').inputValue(),'Keep this draft');
    await page.locator('#dark').click();await page.waitForFunction(()=>document.documentElement.dataset.theme==='dark');
    await page.reload();await page.waitForFunction(()=>window.preferencesReady);assert.equal(await page.locator('html').getAttribute('lang'),'fr');assert.equal(await page.locator('html').getAttribute('data-theme'),'dark');
    await action('preferences');await page.locator('#draft').fill('Failed save draft');await page.locator('#english').click();await page.locator('#save-error').filter({hasText:'could not be saved'}).waitFor();assert.equal(await page.locator('#draft').inputValue(),'Failed save draft');
    await action('flash');await page.goto(url('/ui-preferences/preferences'));await page.waitForFunction(()=>window.preferencesReady);await page.locator('#dismiss').click();await page.reload();await page.waitForFunction(()=>window.preferencesReady);assert.equal(await page.locator('#flash').count(),0);
    await page.goto(url('/generated-admin/profile/create'));await page.waitForFunction(()=>window.adminReady);
    const id='smoke-'+variant+'-'+mode+(base?'mounted':'root');
    await page.locator('#admin-field-id').fill(id);await page.locator('#admin-field-name').fill('Review smoke');
    await page.locator('button[type=submit]').first().click();await page.waitForFunction(()=>document.getElementById('operation-status')?.textContent==='Saved');
    await action('reader');await page.goto(url('/generated-admin/profile/search'));await page.waitForFunction(()=>window.adminReady);assert.equal(await page.locator('[data-admin-action="create"]').count(),0);
    await action('editor');await page.goto(url('/generated-admin/roles'));await page.waitForFunction(()=>window.adminReady);
    const pluginCount=await page.locator('[data-admin-plugin]').count();assert.equal(pluginCount>0,variant==='plugin');
    await page.goto(url('/ui-format/'));await page.waitForFunction(()=>window.formatReady);assert.ok(await page.locator('#format-proof').count());
    await page.goto(url('/ui-format/edit/fr'));await page.waitForFunction(()=>window.formatReady);await page.locator('#edit-number').fill('12,5');await page.locator('#edit-number').press('Enter');assert.equal(await page.locator('input[name=decimal]').inputValue(),'12.50');
    await page.goto(url('/ui-stream/stream'));await page.waitForFunction(()=>document.documentElement.dataset.hydrated==='true');assert.match(await page.locator('#details').textContent(),/project summary is ready/);assert.match(await page.locator('#stream-safety-value').textContent(),/<script>window.hostile=true/);assert.equal(calls.filter(call=>call.path==='/resource').length,0);await page.locator('#counter').click();await page.locator('#counter').filter({hasText:'Clicks: 1'}).waitFor();
    await page.goto(url('/ui-stream/late'));await page.locator('#stackpress-stream-recovery[role=alert]').waitFor();assert.match(await page.locator('#stackpress-stream-recovery').textContent(),/This page didn't finish loading/);assert.equal(await page.locator('#stackpress-stream-complete').count(),0);
    await page.screenshot({path:path.join(directory,'review',`smoke-${variant}-${mode}.png`),fullPage:true});
    assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
    return {variant,mode,base,passed:true,calls,errors,unexpected};
  } finally {
    // Native cleanup must still run if closing a failed browser itself rejects.
    try {if(browser)await browser.close();}
    finally {
      child.stdin.end();
      const timer=setTimeout(()=>child.kill('SIGKILL'),5000);
      try {await exited;}
      finally {clearTimeout(timer);fs.closeSync(stderr);fs.rmSync(scratch,{recursive:true,force:true});}
    }
  }
}

/** Preserve the full original smoke when used as the executable entry point. */
async function main(){
  const results=[];
  for(const [variant,mode,base] of [['disabled','hydrate',''],['plugin','hydrate',''],['disabled','csr','/demo']]){results.push(await run(variant,mode,base));fs.writeFileSync(path.join(directory,'review/browser-partial.json'),JSON.stringify(results,null,2)+'\n');}
  fs.writeFileSync(path.join(directory,'review/browser.json'),JSON.stringify(results,null,2)+'\n');
  console.log('PASS review browser smoke '+results.length+' cases');
}
module.exports={bridge};
if(require.main===module)main().catch(error=>{console.error(error);process.exitCode=1;});
