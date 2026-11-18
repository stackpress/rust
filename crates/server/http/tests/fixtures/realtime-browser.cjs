// Finite real Chromium proof; static artifacts use interception, protocols use the owned Rust host.
const assert = require('node:assert/strict');
const path = require('node:path');
const fs = require('node:fs');
const {chromium} = require(process.env.BLOG_PLAYWRIGHT);
const origin = process.env.REALTIME_ORIGIN;
const output = process.env.REALTIME_ARTIFACTS;
let browser;
const watchdog = setTimeout(async () => {setTimeout(()=>process.exit(1),1000); await browser?.close(); process.exit(1);},45000);
(async()=>{
  browser=await chromium.launch({headless:true});
  try {
    for(const mode of ['hydrate','csr']) {
      const context=await browser.newContext({permissions:['local-network-access']});
      await context.addCookies([{name:'session',value:'fixture',url:origin}]);
      const page=await context.newPage();page.setDefaultTimeout(5000);
      const errors=[];page.on('pageerror',e=>{errors.push(e.message);console.error('PAGEERROR',e.message);});page.on('console',m=>{if(m.type()==='error')console.error('BROWSER',m.text());});
      let release;const boot=new Promise(resolve=>release=resolve);
      await page.addInitScript(()=>{
        window.sockets=[];window.sources=[];window.nativeListeners=0;window.retryTimers=new Set();
        const WS=WebSocket,ES=EventSource,add=EventTarget.prototype.addEventListener,remove=EventTarget.prototype.removeEventListener;
        const registered=new WeakMap();
        EventTarget.prototype.addEventListener=function(name,fn,...args){
          if(this instanceof WS||this instanceof ES){let map=registered.get(this);if(!map){map=new Map();registered.set(this,map);}let set=map.get(name);if(!set){set=new Set();map.set(name,set);}if(!set.has(fn)){set.add(fn);window.nativeListeners++;}}
          return add.call(this,name,fn,...args);
        };
        EventTarget.prototype.removeEventListener=function(name,fn,...args){if(registered.get(this)?.get(name)?.delete(fn))window.nativeListeners--;return remove.call(this,name,fn,...args);};
        window.WebSocket=class extends WS {constructor(...args){super(...args);window.sockets.push(this);if(window.disposeNextConnection){window.disposeNextConnection=false;queueMicrotask(()=>window.realtimeDispose());}}};
        window.EventSource=class extends ES {constructor(...args){super(...args);window.sources.push(this);}};
        const set=window.setTimeout,clear=window.clearTimeout;
        window.setTimeout=function(fn,delay,...args){let id;id=set(()=>{window.retryTimers.delete(id);fn(...args);},delay);if([1000,2000,4000,8000,16000,30000].includes(delay))window.retryTimers.add(id);return id;};
        window.clearTimeout=function(id){window.retryTimers.delete(id);return clear(id);};
      });
      await page.route(origin+'/**',async route=>{
        const url=new URL(route.request().url());
        if(url.pathname.startsWith('/assets/')) {
          const base=path.resolve(output,mode),file=path.resolve(base,url.pathname.slice(8));assert(file.startsWith(base+path.sep));
          if(file.endsWith('start.js'))await boot;
          return route.fulfill({path:file,contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript'});
        }
        if(url.pathname==='/')return route.fulfill({path:path.join(output,mode+'.html'),contentType:'text/html'});
        return route.continue();
      });
      await page.goto(origin,{waitUntil:'commit'});
      if(mode==='hydrate') {await page.locator('#sse-state').waitFor();assert.equal(await page.locator('#sse-state').textContent(),'Disconnected');await page.evaluate(()=>window.originalRoot=document.querySelector('#realtime-ui'));}
      assert.equal(await page.evaluate(()=>window.sockets.length+window.sources.length),0);
      release();await page.waitForFunction(()=>window.realtimeReady);
      await page.waitForFunction(()=>document.querySelector('#ws-state')?.textContent==='Open').catch(async error=>{console.error('STATE',await page.locator('body').innerText(),await page.evaluate(()=>({sockets:window.sockets.map(s=>s.readyState),sources:window.sources.map(s=>s.readyState)})));throw error;});
      await page.waitForFunction(()=>document.querySelector('#sse-message')?.textContent==='ready|2|resumed');
      assert.equal(await page.evaluate(()=>window.sockets.length),1);assert.equal(await page.evaluate(()=>window.sources.length),1);
      if(mode==='hydrate')assert(await page.evaluate(()=>window.originalRoot===document.querySelector('#realtime-ui')));
      await page.click('#send-text');await page.waitForFunction(()=>document.querySelector('#ws-message').textContent.includes('hello'));
      await page.click('#send-binary');await page.waitForFunction(()=>document.querySelector('#ws-message').textContent.includes('Binary([0, 255])'));
      // Native bufferedAmount admission: this proof changes the observed counter, never queues test data.
      await page.evaluate(()=>Object.defineProperty(window.sockets.at(-1),'bufferedAmount',{configurable:true,get:()=>262144}));
      await page.click('#send-text');assert.equal(await page.locator('#result').textContent(),'Err(Backpressure)');
      await page.evaluate(()=>delete window.sockets.at(-1).bufferedAmount);
      const before=await page.evaluate(async()=>await (await fetch('/stats')).json());
      await page.click('#close-ws');await page.waitForFunction(()=>document.querySelector('#ws-state').textContent==='Disconnected');
      await page.click('#reconnect');await page.waitForFunction(()=>document.querySelector('#ws-state').textContent==='Open');
      const after=await page.evaluate(async()=>await (await fetch('/stats')).json());
      assert.equal(after.opens,before.opens+1);assert.equal(after.messages,before.messages);assert(after.resumes>0);assert(after.cookies>0);
      assert((await context.cookies()).some(cookie=>cookie.name==='proof'&&cookie.value==='ok'));
      // An unexpected native close starts the opt-in timer; a successful retry resets it.
      const connections=await page.evaluate(()=>window.sockets.length);
      await page.evaluate(()=>window.sockets.at(-1).close(1000,'retry proof'));
      await page.waitForFunction(count=>window.sockets.length===count+1&&window.sockets.at(-1).readyState===1,connections);
      await page.evaluate(()=>window.sockets.at(-1).close(1000,'dispose during retry'));
      await page.waitForFunction(()=>window.retryTimers.size===1);
      await page.click('#close-sse');await page.evaluate(()=>window.realtimeDispose());
      await page.waitForFunction(()=>window.nativeListeners===0&&window.retryTimers.size===0&&window.sources.every(s=>s.readyState===2)&&window.sockets.every(s=>s.readyState>=2));
      // A fresh owner creates new URLs, retaining no cursor or listeners from the old owner.
      const oldCount=await page.evaluate(()=>window.sockets.length);
      await page.evaluate(async()=>{const module=await import('/assets/realtime.js');window.realtimeClients=module.realtime_clients;window.realtimeDispose=window.realtimeClients('owner=second');});
      await page.waitForFunction(count=>window.sockets.length===count+1&&window.sockets.at(-1).readyState===1,oldCount);
      assert((await page.evaluate(()=>window.sockets.at(-1).url)).endsWith('?owner=second'));
      await page.evaluate(()=>window.realtimeDispose());
      await page.waitForFunction(()=>window.nativeListeners===0&&window.sources.every(s=>s.readyState===2)&&window.sockets.every(s=>s.readyState>=2));
      // Disposal queued by the constructor runs while the new native socket is still CONNECTING.
      await page.evaluate(()=>{window.disposeNextConnection=true;window.realtimeDispose=window.realtimeClients('owner=connecting');});
      await page.waitForFunction(()=>!window.disposeNextConnection&&window.nativeListeners===0&&window.retryTimers.size===0&&window.sources.every(s=>s.readyState===2)&&window.sockets.every(s=>s.readyState>=2));
      assert.deepEqual(errors,[]);
      await context.close();
      console.log('PASS realtime '+mode+' mount, SSE retry/id, WS text/binary/backpressure/reconnect, cookie, cleanup');
    }
    // A cross-site browser handshake is rejected by the fixture's explicit Origin guard.
    const context=await browser.newContext({permissions:['local-network-access']});const page=await context.newPage();
    const alternate=origin.replace('127.0.0.1','localhost');
    const beforeDenied=await (await fetch(origin+'/stats')).json();
    await page.route(alternate+'/**',route=>route.fulfill({body:'<!doctype html><title>Origin proof</title>',contentType:'text/html'}));
    await page.goto(alternate+'/');
    const code=await page.evaluate(url=>new Promise(resolve=>{const ws=new WebSocket(url);ws.onopen=()=>resolve('opened');ws.onclose=e=>resolve(e.code);}),origin.replace('http:','ws:')+'/ws');
    assert.equal(code,1006);const afterDenied=await (await fetch(origin+'/stats')).json();assert.equal(afterDenied.denied,beforeDenied.denied+1);await context.close();
    // An accepted cookie snapshot grants no permanent authority: the app checks current policy.
    const sessionContext=await browser.newContext({permissions:['local-network-access']});
    await sessionContext.addCookies([{name:'session',value:'fixture',url:origin}]);
    const sessionPage=await sessionContext.newPage();
    await sessionPage.route(origin+'/',route=>route.fulfill({body:'<!doctype html><title>Session proof</title>',contentType:'text/html'}));
    await sessionPage.goto(origin+'/');
    const beforeExpiry=await (await fetch(origin+'/stats')).json();
    const expired=await sessionPage.evaluate(async url=>{
      const ws=new WebSocket(url),messages=[];
      await new Promise((resolve,reject)=>{ws.onmessage=e=>{messages.push(e.data);resolve();};ws.onerror=reject;});
      await fetch('/expire',{method:'POST'});
      const closed=new Promise(resolve=>{ws.onclose=e=>resolve({code:e.code,reason:e.reason});});
      ws.send('private action');
      return {...await closed,messages};
    },origin.replace('http:','ws:')+'/ws');
    assert.deepEqual(expired,{code:1008,reason:'session expired',messages:['welcome']});
    const refused=await sessionPage.evaluate(url=>new Promise(resolve=>{const ws=new WebSocket(url);ws.onopen=()=>resolve('opened');ws.onclose=e=>resolve(e.code);}),origin.replace('http:','ws:')+'/ws');
    assert.equal(refused,1006);
    const afterExpiry=await (await fetch(origin+'/stats')).json();
    assert.equal(afterExpiry.opens,beforeExpiry.opens+1);assert.equal(afterExpiry.messages,beforeExpiry.messages);
    await sessionContext.close();
    console.log('PASS explicit app session revocation before reconnect and after upgrade');
    fs.writeFileSync(path.join(output,'browser-result.json'),JSON.stringify({modes:['hydrate','csr'],originGuard:true,sessionRevocation:true,passed:true},null,2));
  } finally {await browser.close();clearTimeout(watchdog);}
})().catch(error=>{console.error(error);process.exitCode=1;});
