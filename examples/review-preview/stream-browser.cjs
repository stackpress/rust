// Actual review gateway HTTP frames reach Chromium without a listener or clean-EOF repair.
const assert = require('node:assert/strict');
const cp = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const {chromium} = require(process.env.REVIEW_PLAYWRIGHT);
const {bridge} = require('./browser.cjs');
const directory = process.env.REVIEW_DIRECTORY;
const origin = 'http://127.0.0.1:3001';

// Only document requests use the worker; all assets pass through the gateway's manifest.
const worker = `
self.addEventListener('install',event=>self.skipWaiting());
self.addEventListener('activate',event=>event.waitUntil(clients.claim()));
self.addEventListener('fetch',event=>{
 const url=new URL(event.request.url());
 if(!url.pathname.includes('/ui-stream/') || !['stream','late','pending'].includes(url.pathname.split('/').pop()))return;
 event.respondWith((async()=>{
  const head=await (await fetch('/__head')).json();
  return new Response(new ReadableStream({
   async pull(controller){
    try{
     const value=await (await fetch('/__next')).json();
     if(value.done && value.terminal)controller.close();
     else if(value.done)controller.error(new Error('Native HTTP body was truncated'));
     else controller.enqueue(Uint8Array.from(atob(value.chunk),character=>character.charCodeAt(0)));
    }catch(error){controller.error(error);}
   }
  }),{status:head.status,headers:{...head.headers,
   // The external classic script also works under a policy prohibiting inline execution.
   'Content-Security-Policy':"script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'none'"
  }});
 })());
});`;

/** Own one finite gateway process, preserving headers, frame boundaries and interruption. */
function stream(base, mode) {
  const binary = path.join(directory,'bundle/disabled/slice8-review');
  const child = cp.spawn(binary,['stream-pipe','--port','3001','--directory',directory,'--variant','disabled','--render','hydrate','--base',base],{stdio:['pipe','pipe','pipe']});
  const log = fs.createWriteStream(path.join(directory,'review',`stream-native-${mode}-${base?'mounted':'root'}.log`));
  child.stderr.pipe(log);
  const queued = [], waiting = [];
  let buffer = '', report, stopped;
  const fail = error => {stopped=error;for(const receiver of waiting.splice(0))receiver.reject(error);};
  child.stdout.setEncoding('utf8');
  child.stdout.on('data',text=>{
    buffer+=text;
    let index;
    while((index=buffer.indexOf('\n'))>=0){
      const line=buffer.slice(0,index);buffer=buffer.slice(index+1);
      try{
        const value=JSON.parse(line);
        log.write(JSON.stringify(value)+'\n');
        if(value.done)report=value;
        const receiver=waiting.shift();
        if(receiver)receiver.resolve(value);else queued.push(value);
      }catch(error){
        // Reject the awaited frame so scenario finally still closes/reaps the child.
        fail(error);
      }
    }
  });
  child.on('error',fail);child.stdin.on('error',fail);
  const exited=new Promise(resolve=>child.once('close',code=>{
    if(code || !report)fail(Error('Progressive review pipe exited '+code));
    log.end();resolve(code);
  }));
  child.stdin.end(JSON.stringify({method:'GET',path:base+'/ui-stream/'+mode,headers:{},body:''})+'\n');
  return {
    next:()=>queued.length?Promise.resolve(queued.shift()):stopped?Promise.reject(stopped):new Promise((resolve,reject)=>waiting.push({resolve,reject})),
    get report(){return report;},
    async close(){
      const timer=setTimeout(()=>child.kill('SIGTERM'),12000);
      const kill=setTimeout(()=>child.kill('SIGKILL'),14000);
      try{await exited;}finally{clearTimeout(timer);clearTimeout(kill);}
    }
  };
}

/** Show recovery from the actual bundle/gateway, with safe completion-gated hydration. */
async function scenario(browser,base,mode) {
  const binary=path.join(directory,'bundle/disabled/slice8-review');
  const child=cp.spawn(binary,['pipe','--port','3001','--directory',directory,'--variant','disabled','--render','hydrate','--base',base],{stdio:['pipe','pipe','ignore']});
  const exited=new Promise(resolve=>child.once('close',resolve));
  const request=bridge(child), native=stream(base,mode);
  const context=await browser.newContext({viewport:{width:1280,height:800}});
  const page=await context.newPage();page.setDefaultTimeout(12000);
  const errors=[],unexpected=[],assets=[];let resources=0,head;
  page.on('pageerror',error=>errors.push(error.message));
  try{
    await context.route('**/*',async route=>{
      const url=new URL(route.request().url());
      if(url.origin!==origin){unexpected.push(url.href);return route.abort();}
      if(url.pathname==='/install')return route.fulfill({contentType:'text/html',body:'<!doctype html><title>Review stream proof</title>'});
      if(url.pathname==='/sw.js')return route.fulfill({contentType:'application/javascript',body:worker});
      if(url.pathname==='/__head'){
        head=(await native.next()).head;
        assert.equal(head.status,200);
        const headers=Object.fromEntries(Object.entries(head.headers).filter(([key])=>!['transfer-encoding','connection','content-length'].includes(key)).map(([key,values])=>[key,values.join('\n')]));
        return route.fulfill({contentType:'application/json',body:JSON.stringify({status:head.status,headers})});
      }
      if(url.pathname==='/__next')return route.fulfill({contentType:'application/json',body:JSON.stringify(await native.next())});
      if(url.pathname==='/resource')resources++;
      const result=await request({method:'GET',path:url.pathname+url.search,headers:{},body:''});
      if(url.pathname.includes('/assets/'))assets.push({path:url.pathname,status:result.status});
      return route.fulfill({status:result.status,headers:Object.fromEntries(Object.entries(result.headers).filter(([key])=>!['transfer-encoding','connection','content-length'].includes(key)).map(([key,values])=>[key,values.join('\n')])),body:Buffer.from(result.body,'base64')});
    });
    await page.goto(origin+'/install');
    await page.evaluate(async()=>{await navigator.serviceWorker.register('/sw.js');await navigator.serviceWorker.ready;});
    await page.goto(origin+base+'/ui-stream/'+mode,{waitUntil:'commit'});
    await page.locator('#shell').waitFor({state:'visible'});
    const recovery=page.locator('#stackpress-stream-recovery');
    if(mode==='stream'){
      await page.locator('html[data-hydrated=true]').waitFor();
      assert.equal(await recovery.isVisible(),false);
      assert.match(await page.locator('#details').textContent(),/project summary is ready/);
      assert.match(await page.locator('#stream-safety-value').textContent(),/<script>window.hostile=true/);
      await page.locator('#counter').click();assert.equal(await page.locator('#counter').textContent(),'Clicks: 1');
    }else{
      await page.locator('#stackpress-stream-recovery[role=alert]').waitFor();
      const observable=await page.evaluate(()=>({readyState:document.readyState,failed:!!document.getElementById('stackpress-stream-failed')}));
      assert.match(await recovery.textContent(),observable.failed || observable.readyState!=='loading'?/didn't finish loading/:/hasn't finished loading/);
      assert.equal(await page.locator('#stackpress-stream-complete').count(),0);
      assert.equal(await page.locator('#details').count(),0);
      assert.equal(await page.locator('html').getAttribute('data-hydrated'),null);
      assert.equal(await page.evaluate(()=>window.streamBeforeHydrate),undefined);
      assert.equal(await recovery.locator('a').getAttribute('href'),base+'/ui-stream/'+mode);
    }
    assert.equal(resources,0);assert.equal(await page.evaluate(()=>window.hostile),undefined);
    assert.ok(assets.some(asset=>asset.path===base+'/ui-stream/assets/stream-status.js' && asset.status===200));
    await native.close();assert.equal(native.report.terminal,mode==='stream');
    const state=await page.evaluate(()=>({readyState:document.readyState,failed:!!document.getElementById('stackpress-stream-failed'),complete:!!document.getElementById('stackpress-stream-complete'),text:document.getElementById('stackpress-stream-recovery').textContent}));
    const name=`review-stream-${mode}-${base?'mounted':'root'}`;
    for(const width of [1280,375]){await page.setViewportSize({width,height:800});await page.screenshot({path:path.join(directory,'review',`${name}-${width}.png`),fullPage:true});}
    assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
    return {base,mode,state,resources,assets,report:native.report};
  }finally{
    await native.close();
    await page.evaluate(async()=>{for(const registration of await navigator.serviceWorker.getRegistrations())await registration.unregister();}).catch(()=>{});
    await context.close();child.stdin.end();
    const timer=setTimeout(()=>child.kill('SIGKILL'),12000);
    try{await exited;}finally{clearTimeout(timer);}
  }
}

/** Six finite documents cover both mounts, successful replay, late failure and deadline. */
async function main(){
  const browser=await chromium.launch({headless:true});const results=[];
  try{
    for(const base of ['', '/demo'])for(const mode of ['stream','late','pending']){
      results.push(await scenario(browser,base,mode));
      fs.writeFileSync(path.join(directory,'review/stream-browser.json'),JSON.stringify({version:browser.version(),results},null,2));
      console.log('PASS review stream',base||'root',mode);
    }
  }finally{await browser.close();}
}
main().catch(error=>{console.error(error);process.exitCode=1;});
