// Finite real SQLite/compiled HTTP CRUD proof. Every native request has its own deadline.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
if (!process.env.ADMIN_PLAYWRIGHT) throw Error('ADMIN_PLAYWRIGHT is required');
const { chromium } = require(process.env.ADMIN_PLAYWRIGHT);
const config = JSON.parse(process.env.ADMIN_BROWSER_CASE);
const output = process.env.ADMIN_BROWSER_OUTPUT;
const tag = process.env.ADMIN_BROWSER_TAG;
const artifacts = config.plugin ? path.join(output,'plugin') : output;
const binary = path.join(artifacts,'admin-web');
const directory = fs.mkdtempSync(path.join(output,'browser-fixture-'));
const database = path.join(directory,'admin.sqlite');
const calls = [], errors = [], unexpected = [];
const expectedHttpDiagnostics=[];
let releaseBoot;
const boot=new Promise(resolve=>{releaseBoot=resolve;});
let bootPending=true;
let actor = 'editor', faultNext = false;
const native = (pathname, method='GET', body, extra={}) => {
  const request = {database,mount:config.mount,actor,path:pathname,method,body,csr:config.mode==='csr',headers:{origin:'https://admin.invalid','x-admin-csrf':'finite-session-csrf'},...extra};
  const result = cp.execFileSync(binary,['--request'],{input:JSON.stringify(request),encoding:'utf8',timeout:10000,killSignal:'SIGKILL',maxBuffer:2097152});
  return JSON.parse(result);
};
const url = (module, action, key) => `https://admin.invalid${config.mount}${module}/${action}${key?'?key='+encodeURIComponent(JSON.stringify(key)):''}`;
const frame = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const snapshot = page => page.locator('#admin-seed').evaluate(node => JSON.parse(node.textContent));
const draft = page => page.locator('#admin-draft').evaluate(node => JSON.parse(node.textContent));
const status = (page,message) => page.waitForFunction(message => document.querySelector('#operation-status')?.textContent.includes(message),message);
const settled = page => page.waitForFunction(() => document.querySelector('#operation-status')?.textContent !== 'Submitting…');
const go = async (page,target) => {await page.goto(target);await page.waitForFunction(() => window.adminReady===true);await page.evaluate(zoom => {document.body.style.zoom=String(zoom);},config.zoom);};
const click = async (page,selector) => {await page.locator(selector).click();await page.waitForFunction(() => window.adminReady===true);await page.evaluate(zoom => {document.body.style.zoom=String(zoom);},config.zoom);};
const picture = async (page,name) => {
  const geometry=await page.evaluate(() => ({width:innerWidth,scroll:document.documentElement.scrollWidth,overflow:[...document.querySelectorAll('body *')].filter(node=>node.getBoundingClientRect().right>innerWidth+1).map(node=>({tag:node.tagName,id:node.id,right:node.getBoundingClientRect().right,width:node.getBoundingClientRect().width})).slice(-10)}));
  fs.mkdirSync(path.join(output,'screenshots'),{recursive:true});
  await page.screenshot({path:path.join(output,'screenshots',`${tag}-${name}.png`),fullPage:true});
  assert.ok(geometry.scroll<=geometry.width+1,JSON.stringify(geometry));
};

async function flow(page) {
  native('/', 'GET', undefined, {initialize:true});
  const create = (module,input) => assert.equal(native(`${config.mount}api/${module}/create`,'POST',{input}).status,200);
  create('profile',{id:'p1',name:'Parent'});create('profile',{id:'p2',name:'Other parent'});
  create('project',{id:'j1',ownerId:'p1',title:'Project one'});create('project',{id:'j2',ownerId:'p2',title:'Project two'});
  create('task',{id:'t1',projectId:'j1',title:'First task',done:false});create('task',{id:'t2',projectId:'j2',title:'Other task',done:false});create('task',{id:'t3',projectId:'j1',title:'Finished',done:true});
  if(config.plugin) {
    const pathname=`${config.mount}task/create`;
    assert.ok(native(pathname,'GET',undefined,{csr:false}).body.includes('Plugin number label'));
    await page.goto(url('task','create'),{waitUntil:'commit'});
    await page.locator('#app').waitFor({state:'attached'});
    let initial;
    if(config.mode==='hydrate') {await page.locator('#admin-field-amount').waitFor();initial=await page.locator('#app').innerHTML();}
    releaseBoot();bootPending=false;await page.waitForFunction(()=>window.adminReady===true);
    if(initial)assert.equal(await page.locator('#app').innerHTML(),initial);
    await page.evaluate(zoom=>{document.body.style.zoom=String(zoom);},config.zoom);
    assert.equal(await page.locator('label[for="admin-field-amount"]').textContent(),'Plugin number label');
    await page.locator('#admin-field-amount').fill('1,234.50');await page.locator('#admin-field-amount').press('Enter');await frame(page);
    assert.equal((await draft(page)).amount,1234.5);
    await picture(page,'plugin-format');assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
    await page.evaluate(()=>window.adminDispose());return;
  }
  await page.goto(url('profile','search'),{waitUntil:'commit'});
  await page.locator('#app').waitFor({state:'attached'});
  let initial;
  if(config.mode==='hydrate') {
    await page.locator('#public-table').waitFor();initial=await page.locator('#app').innerHTML();
    await page.evaluate(()=>{window.beforeHydration=document.querySelector('#admin-search-filter-name');});
  } else assert.equal(await page.locator('#app').textContent(),'');
  const initialCalls=calls.length;releaseBoot();bootPending=false;
  await page.waitForFunction(()=>window.adminReady===true);
  if(config.mode==='hydrate') {
    assert.equal(await page.locator('#app').innerHTML(),initial,'initial SSR/hydrated parsed DOM parity');
    assert.equal(await page.evaluate(()=>window.beforeHydration===document.querySelector('#admin-search-filter-name')),true);
  }
  assert.equal(calls.length,initialCalls,'no initial duplicate loader/fetch');
  await page.evaluate(zoom=>{document.body.style.zoom=String(zoom);},config.zoom);
  assert.equal(await page.locator('#search-total').textContent(),'2');
  assert.equal((await snapshot(page)).data.total,2);
  const labels=await page.locator('[aria-label=Records] a').allTextContents();
  assert.deepEqual(labels,['Parent','Other parent']);assert.ok(labels.every(label=>!/[{}]/.test(label)));
  const gap=await page.locator('.admin-actions').evaluate(node=>{
    const [a,b]=[...node.children].map(node=>node.getBoundingClientRect());
    return b.top>=a.bottom ? b.top-a.bottom : b.left-a.right;
  });assert.ok(gap>=8,`pagination action gap ${gap}`);
  const ids=await page.locator('[id]').evaluateAll(nodes=>nodes.map(node=>node.id));
  assert.equal(new Set(ids).size,ids.length);
  await click(page,'[data-admin-action="create"]');
  assert.equal(await page.locator('#admin-field-id-presence').count(),0);assert.equal(await page.locator('#admin-field-name-presence').count(),0);
  await page.locator('#admin-field-id').fill('created');
  await page.locator('#admin-field-name').fill('temporary');
  await page.locator('#admin-field-name').fill('');
  await page.locator('#admin-field-name').evaluate(node=>{window.retainedInput=node;});
  const before=calls.length;
  await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'input validation failed');
  assert.equal(calls.length,before+1,'one submission; no automatic validation replay');
  assert.equal((await draft(page)).id,'created');assert.equal((await draft(page)).name,'');
  assert.equal(await page.locator('#admin-field-name').evaluate(node=>node===window.retainedInput),true);
  assert.equal(await page.locator('#admin-field-name-error').textContent(),'Name </script> 🦀 is required');
  await picture(page,'validation');
  await page.locator('#admin-field-name').fill('Created </script> 🦀');
  await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'Saved');
  await click(page,'#saved-record');assert.equal(await page.locator('#record-detail').textContent().then(text=>text.includes('Created </script> 🦀')),true);
  assert.equal(await page.locator('script:not([src]):not([type="application/json"])').count(),0);
  await click(page,'[data-admin-action="update"]');
  assert.equal(await page.locator('#admin-field-id').count(),0,'identity is absent from update controls');
  assert.ok((await page.locator('#admin-field-name-presence').textContent()).includes('Leave unchanged'));
  await page.locator('#admin-field-name').fill('Updated');
  await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'Saved');
  await click(page,'[data-admin-action="copy"]');
  assert.deepEqual((await snapshot(page)).key,{});
  assert.equal('id' in (await snapshot(page)).data,false,'copy seed omits source identity');
  assert.equal('id' in await draft(page),false);
  assert.equal(await page.locator('#admin-field-id').inputValue(),'');
  await page.locator('#admin-field-id').fill('copied');await page.locator('#admin-field-name').fill('Copied');
  await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'Saved');
  assert.equal(JSON.parse(native(`${config.mount}api/profile/get`,'POST',{key:{id:'copied'}}).body).results.name,'Copied');
  await click(page,'#saved-record');await click(page,'[data-admin-action="remove"]');
  assert.equal(await page.getByRole('button',{name:'Confirm remove'}).isDisabled(),true);
  await page.locator('#confirm-action').check();await page.getByRole('button',{name:'Confirm remove'}).click();await status(page,'Saved');
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'copied'}}).status,404);
  await click(page,'#restore-link');await page.locator('#confirm-action').check();await page.getByRole('button',{name:'Confirm restore'}).click();await status(page,'Saved');
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'copied'}}).status,200);
  await go(page,url('project','detail',{id:'j1'}));await click(page,'[data-related="task"]');
  assert.equal(await page.locator('#search-total').textContent(),'2');
  assert.equal(await page.locator('#public-table').textContent().then(text=>text.includes('Other task')),false);
  await click(page,'[data-admin-action="create"]');
  assert.equal(await page.locator('#admin-field-projectId').isDisabled(),true);
  assert.equal((await draft(page)).projectId,'j1');
  await page.locator('#admin-field-id').fill('related');await page.locator('#admin-field-title').fill('Related created');
  await page.locator('#admin-field-note-presence').selectOption('null');
  assert.equal((await draft(page)).note,null);assert.equal((await draft(page)).priority,0);assert.equal((await draft(page)).done,false);
  assert.ok((await page.locator('#admin-field-note-presence').textContent()).includes('Clear (empty)'));
  const unchangedCalls=calls.length;
  assert.equal(await page.locator('label[for="admin-field-amount"]').textContent(),'amount');
  assert.ok(!(await page.locator('#app').textContent()).includes('Plugin number label'),'disabled plugin removed after regeneration and rebuild');
  await page.locator('#admin-field-amount').fill('-');await page.locator('#admin-field-amount').press('Enter');await frame(page);
  assert.equal('amount' in await draft(page),false);assert.equal(calls.length,unchangedCalls);
  assert.match(await page.locator('#admin-field-amount-error').textContent(),/Complete|valid/);
  assert.equal(await page.getByRole('button',{name:'Save',exact:true}).isDisabled(),true);
  await picture(page,'formatted-error');
  await page.locator('#admin-field-amount').fill('0');await page.locator('#admin-field-amount').press('Enter');await frame(page);
  assert.equal((await draft(page)).amount,0,'formatted zero is supplied');
  await page.locator('#admin-field-amount-presence').selectOption('null');await frame(page);assert.equal((await draft(page)).amount,null);
  await page.locator('#admin-field-amount-presence').selectOption('missing');await frame(page);assert.equal('amount' in await draft(page),false);
  for(const [field,raw] of [['amount','1,234.565'],['decimal','1\u202f234,565'],['date','2024-02-29'],['time','12:30'],['instant','2024-02-29T12:30:00+02:00'],['phone','2025550123']]) {
    await page.locator('#admin-field-'+field).fill(raw);await page.locator('#admin-field-'+field).press('Enter');await frame(page);
  }
  const formatted=await draft(page);
  assert.equal(formatted.amount,1234.56);assert.equal(formatted.decimal,'1234.56');assert.equal(formatted.date,'2024-02-29');assert.equal(formatted.time,'12:30:00');assert.equal(formatted.instant,'2024-02-29T10:30:00Z');assert.equal(formatted.phone,'+12025550123');
  assert.equal(await page.locator('#admin-field-time').inputValue(),'12:30:00');
  assert.ok(!(await page.locator('#admin-field-instant').inputValue()).includes('.000000000'));
  for(const [field,raw] of [['time','12:30:00.5'],['instant','2024-02-29T12:30:00.123456789+02:00']]) {
    await page.locator('#admin-field-'+field).fill(raw);await page.locator('#admin-field-'+field).press('Enter');await frame(page);
  }
  assert.equal(await page.locator('#admin-field-time').inputValue(),'12:30:00.5');
  const goodInstant=(await draft(page)).instant;
  assert.equal(goodInstant,'2024-02-29T10:30:00.123456789Z');
  const hints=await page.locator('.sp-formatted .sp-edit > p[id$="-help"]:visible').count();
  assert.equal(hints,1,'only focused field shows its hint');
  assert.equal((await page.locator('body').innerText()).split('Press Enter to apply the value, or Escape to cancel.').length-1,1,'hint is not duplicated in visible text');
  for(const field of ['amount','decimal','date','time','instant','phone']) {
    const described=await page.locator('#admin-field-'+field).getAttribute('aria-describedby');
    assert.ok(described.split(' ').includes('admin-field-'+field+'-help'));
  }
  if(config.width===1280)for(const field of ['date','time','instant','phone']) {
    const fits=await page.locator('#admin-field-'+field).evaluate(node=>{
      const style=getComputedStyle(node),canvas=document.createElement('canvas'),context=canvas.getContext('2d');context.font=style.font;
      return context.measureText(node.value).width+parseFloat(style.paddingLeft)+parseFloat(style.paddingRight)<=node.clientWidth+1;
    });assert.ok(fits,field+' is not truncated');
  }
  await page.locator('#admin-field-instant').fill('2024-02-29T12:30:00');await page.locator('#admin-field-instant').press('Enter');await frame(page);
  assert.equal((await draft(page)).instant,goodInstant);assert.notEqual(await page.locator('#admin-field-instant-error').textContent(),'');
  await page.locator('#admin-field-instant').press('Escape');await frame(page);
  await page.locator('#admin-field-phone').fill('+33');await page.locator('#admin-field-phone').press('Enter');await frame(page);
  assert.equal((await draft(page)).phone,'+12025550123');assert.notEqual(await page.locator('#admin-field-phone-error').textContent(),'');
  await page.locator('#admin-field-phone').press('Escape');await frame(page);
  assert.equal(calls.length,unchangedCalls,'formatted edits never submit a write');
  assert.equal(await page.getByRole('button',{name:'Save',exact:true}).isDisabled(),false);

  await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'Saved');
  const related=JSON.parse(native(`${config.mount}api/task/get`,'POST',{key:{id:'related'}}).body).results;
  assert.equal(related.amount,1234.56);assert.equal(related.decimal,'1234.56');assert.equal(related.phone,'+12025550123');assert.equal(related.instant,goodInstant);assert.equal(related.time,'12:30:00.5');assert.equal(related.projectId,'j1');assert.equal(related.note,null);assert.equal(related.priority,0);assert.equal(related.done,false);
  await picture(page,'related');
  await click(page,'#saved-record');assert.ok((await page.locator('#record-detail').textContent()).includes('1,234.56'));assert.ok((await page.locator('#record-detail').textContent()).includes('+1'));
  assert.ok(!(await page.locator('#record-detail').textContent()).includes('.000000000'));
  assert.equal((await page.locator('[id]').evaluateAll(nodes=>new Set(nodes.map(node=>node.id)).size)),await page.locator('[id]').count());
  await go(page,url('task','search'));
  await page.locator('#admin-search-filter-done-presence').selectOption('value');
  await page.locator('#admin-search-filter-priority-presence').selectOption('value');
  await page.locator('#admin-limit').selectOption('1');
  await page.getByRole('button',{name:'Search',exact:true}).click();await settled(page);
  assert.equal(await page.locator('#search-total').textContent(),'3','total is actual match count, not one visible row');
  let href=await page.locator('#search-deep-link').getAttribute('href');
  assert.equal(JSON.parse(new URL(href,'https://admin.invalid').searchParams.get('state')).filters.done,false);
  assert.equal(JSON.parse(new URL(href,'https://admin.invalid').searchParams.get('state')).filters.priority,0);
  await page.locator('#next-page').click();await settled(page);
  href=await page.locator('#search-deep-link').getAttribute('href');assert.equal(JSON.parse(new URL(href,'https://admin.invalid').searchParams.get('state')).skip,1);
  await go(page,new URL(href,'https://admin.invalid').href);assert.equal((await snapshot(page)).query.filters.done,false);assert.equal((await snapshot(page)).query.skip,1);
  await picture(page,'search');
  await go(page,url('profile','search'));
  for (const direction of ['desc','asc',null]) {
    await page.locator('[data-order="name"]').click();await settled(page);
    href=await page.locator('#search-deep-link').getAttribute('href');const order=JSON.parse(new URL(href,'https://admin.invalid').searchParams.get('state')).order;
    assert.equal(order?.[1]??null,direction);
  }
  actor='reader';await go(page,url('profile','search'));assert.equal(await page.locator('[data-admin-action="create"]').count(),0);
  assert.equal(await page.evaluate(target=>fetch(target).then(response=>response.status),url('profile','create')),403,'deep link enforces the same policy');
  actor='editor';await go(page,url('profile','create'));await page.locator('#admin-field-id').fill('denied');await page.locator('#admin-field-name').fill('Preserved denied draft');
  actor='reader';await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'operation is forbidden');
  assert.equal((await draft(page)).name,'Preserved denied draft');
  actor='editor';await go(page,url('profile','create'));await page.locator('#admin-field-id').fill('uncertain');await page.locator('#admin-field-name').fill('Preserved uncertain draft');
  faultNext=true;const writes=calls.filter(call=>call.method==='POST'&&call.pathname.endsWith('/create')).length;
  await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'Write outcome is unknown');await frame(page);
  assert.equal((await draft(page)).name,'Preserved uncertain draft');assert.equal(await page.getByRole('button',{name:'Save',exact:true}).isDisabled(),true);
  assert.equal(await page.locator('#operation-status').textContent().then(text=>text.includes('Saved')),false);
  assert.equal(calls.filter(call=>call.method==='POST'&&call.pathname.endsWith('/create')).length,writes+1);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'uncertain'}}).status,200,'unknown acknowledgement follows a real SQLite effect');
  await picture(page,'unknown');
  assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
  await page.evaluate(()=>window.adminDispose());assert.equal(await page.locator('#app').textContent(),'');
}

(async()=>{
  let browser,timer;
  try {
    browser=await chromium.launch({headless:true});
    const context=await browser.newContext({viewport:{width:config.width,height:1000}});
    const page=await context.newPage();page.setDefaultTimeout(10000);
    page.on('pageerror',error=>errors.push(error.message));
    page.on('console',message=>{if(message.type()==='error' && /^Failed to load resource: the server responded with a status of (400|403|404|500)/.test(message.text()))expectedHttpDiagnostics.push(message.text());else if(message.type()==='error' || /hydration.*(mismatch|error)/i.test(message.text()))errors.push(message.text());});
    await page.route('**/*',async route=>{
      const request=route.request(),target=new URL(request.url());
      if(target.origin!=='https://admin.invalid'){unexpected.push(target.href);return route.abort();}
      const prefix=config.mount+'assets/';
      if(target.pathname.startsWith(prefix)){
        if(bootPending && target.pathname===prefix+'start.js')await boot;
        const assets=path.join(artifacts,'assets',config.mode),file=path.resolve(assets,target.pathname.slice(prefix.length));
        assert.ok(file.startsWith(assets+path.sep));return route.fulfill({path:file,contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript'});
      }
      const method=request.method();const body=request.postData()?JSON.parse(request.postData()):undefined;
      calls.push({method,pathname:target.pathname});
      const fault=faultNext && method==='POST';if(fault)faultNext=false;
      const headers=await request.allHeaders();delete headers['content-type'];delete headers['content-length'];
      // Actor/fault/database are trusted harness inputs, separate from browser payloads.
      const result=native(target.pathname+target.search,method,body,{headers,fault});
      return route.fulfill({status:result.status,headers:result.headers,body:result.body});
    });
    await Promise.race([flow(page),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('inner browser deadline')),105000);})]);
    fs.writeFileSync(path.join(output,`browser-${tag}.json`),JSON.stringify({config,calls,errors,unexpected,expectedHttpDiagnostics,passed:true},null,2));
    console.log(JSON.stringify({config,requests:calls.length,passed:true}));
  } finally {releaseBoot();clearTimeout(timer);if(browser)await browser.close();fs.rmSync(directory,{recursive:true,force:true});}
})().catch(error=>{console.error(error);process.exitCode=1;});
