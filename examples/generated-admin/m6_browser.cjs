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
  const result = cp.execFileSync(binary,['--request'],{input:JSON.stringify(request),encoding:'utf8',timeout:10000,killSignal:'SIGKILL',maxBuffer:8388608});
  return JSON.parse(result);
};
const url = (module, action, key) => `https://admin.invalid${config.mount}${module}/${action}${key?'?key='+encodeURIComponent(JSON.stringify(key)):''}`;
const frame = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const go = async (page,target) => {await page.goto(target);await page.waitForFunction(() => window.adminReady===true);await page.evaluate(zoom => {document.body.style.zoom=String(zoom);},config.zoom);};
const picture = async (page,name) => {
  const geometry=await page.evaluate(() => ({width:innerWidth,scroll:document.documentElement.scrollWidth,overflow:[...document.querySelectorAll('body *')].filter(node=>node.getBoundingClientRect().right>innerWidth+1).map(node=>({tag:node.tagName,id:node.id,right:node.getBoundingClientRect().right,width:node.getBoundingClientRect().width})).slice(-10)}));
  fs.mkdirSync(path.join(output,'screenshots'),{recursive:true});
  await page.screenshot({path:path.join(output,'screenshots',`${tag}-${name}.png`),fullPage:true});
  assert.ok(geometry.scroll<=geometry.width+1,JSON.stringify(geometry));
};

async function flow(page) {
  native('/', 'GET', undefined, {initialize:true});
  const create = (module,input) => assert.equal(native(`${config.mount}api/${module}/create`,'POST',{input}).status,200);
  create('profile',{id:'parent',name:'Parent'});
  create('project',{id:'j1',ownerId:'parent',title:'First project'});
  create('project',{id:'j2',ownerId:'parent',title:'Other project'});
  // Hold bootstrap to prove hydration adopts the finite SSR form without a request.
  await page.goto(url('profile','import'),{waitUntil:'commit'});
  await page.locator('#app').waitFor({state:'attached'});
  if(config.mode==='hydrate') {
    await page.locator('#admin-import-file').waitFor();
    await page.evaluate(()=>{window.beforeHydration=document.querySelector('#admin-import-file');});
  } else assert.equal(await page.locator('#app').textContent(),'');
  const initialCalls=calls.length;releaseBoot();bootPending=false;
  await page.waitForFunction(()=>window.adminReady===true);
  await page.evaluate(zoom=>{document.body.style.zoom=String(zoom);},config.zoom);
  if(config.mode==='hydrate')assert.equal(await page.evaluate(()=>window.beforeHydration===document.querySelector('#admin-import-file')),true);
  assert.equal(calls.length,initialCalls,'mount must not fetch or write');
  assert.equal(await page.locator('#import-decode').isChecked(),false);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  const file=async(csv,name='records.csv')=>{
    await page.locator('#admin-import-file').setInputFiles({name,mimeType:'text/csv',buffer:Buffer.isBuffer(csv)?csv:Buffer.from(csv)});
    await page.waitForFunction(()=>document.querySelector('#import-file-name').textContent.length>0&&!document.querySelector('#import-status').textContent.includes('Reading selected'));
  };
  const preview=async()=>{await page.locator('#import-preview').click();await page.waitForFunction(()=>document.querySelector('#import-status').textContent!=='Validating every row; nothing is being written…');};
  const confirm=async()=>{await page.locator('#import-confirm-check').check();await page.locator('#import-confirm').click();await page.waitForFunction(()=>document.querySelector('#import-status').textContent!=='Applying one atomic batch…');};
  // Display-only names cannot break UTF-8 admission or narrow layout.
  await file('id,name\nx,Name','🦀'.repeat(127)+'X🌈.csv');
  assert.equal((await page.locator('#import-file-error').textContent()).trim(),'');
  assert.equal(await page.locator('#import-file-name').textContent(),'🦀'.repeat(127)+'X');
  await file('id,name\nx,Name','a'.repeat(128)+'.csv');
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),true);
  await file('id,name\nbad,');await preview();
  assert.match(await page.locator('#import-preview-table').textContent(),/Name.*is required|must not be empty/);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  assert.equal(await page.locator('#import-file-name').textContent(),'records.csv');
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'bad'}}).status,404);
  await picture(page,'invalid-preview');
  // Unknown headers fail before a row preview, retaining the selected file.
  await file('id,private\nbad,hidden');await preview();assert.match(await page.locator('#import-status').textContent(),/header/);
  await file(Buffer.from([0xff]),'invalid.csv');await page.waitForFunction(()=>document.querySelector('#import-file-error').textContent.length>0);
  assert.match(await page.locator('#import-file-error').textContent(),/UTF-8/);
  await file(Buffer.alloc(1048577,120),'oversize.csv');await page.waitForFunction(()=>document.querySelector('#import-file-error').textContent.length>0);
  assert.match(await page.locator('#import-file-error').textContent(),/byte limit/);assert.equal(await page.locator('#import-preview').isDisabled(),true);
  // Ordinary uploads retain apostrophes; opt-in decoding changes only documented guards.
  const csv="id,name\na,'=1+1\nb,''=literal";
  await file(csv);await preview();
  let cells=await page.locator('#import-preview-table tbody tr').first().locator('td pre').allTextContents();
  assert.equal(JSON.parse(cells[0])[1],"'=1+1");assert.equal(JSON.parse(cells[1]).name,"'=1+1");
  await page.locator('#import-decode').check();await frame(page);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  await preview();cells=await page.locator('#import-preview-table tbody tr').first().locator('td pre').allTextContents();
  assert.equal(JSON.parse(cells[0])[1],"'=1+1");assert.equal(JSON.parse(cells[1]).name,'=1+1');
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'a'}}).status,404);
  await picture(page,'decoded-preview');
  await confirm();assert.match(await page.locator('#import-status').textContent(),/^Imported 2 rows/);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'a'}}).status,200);
  await picture(page,'confirmed');
  await preview();assert.match(await page.locator('#import-preview-table').textContent(),/Identity already exists/);
  await page.locator('#admin-import-cancel').click();assert.equal(await page.locator('#import-file-name').textContent(),'');
  assert.equal(await page.locator('#import-preview-table').count(),0);
  // Deep links and post requests enforce policy even if the page was rendered for an editor.
  await file('id,name\ndenied,Kept draft');actor='reader';await preview();
  assert.match(await page.locator('#import-status').textContent(),/forbidden/);assert.equal(await page.locator('#import-file-name').textContent(),'records.csv');
  assert.equal(native(`${config.mount}profile/import`).status,403);actor='editor';
  // Related create applies a missing FK; a conflicting explicit FK blocks the whole batch.
  const parent=encodeURIComponent(JSON.stringify({module:'project',key:{id:'j1'},field:'projectId',value:'j1'}));
  const related=url('task','import')+'?parent='+parent;
  await go(page,related);
  await file('id,title,done,priority\nt1,Scoped,false,0');await preview();
  assert.match(await page.locator('#import-preview-table').textContent(),/projectId/);await confirm();
  assert.equal(native(`${config.mount}api/task/get`,'POST',{key:{id:'t1'}}).status,200);
  await file('id,projectId,title\nescape,j2,Wrong');await preview();
  assert.match(await page.locator('#import-preview-table').textContent(),/Related parent cannot be replaced/);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);await picture(page,'related-preview');
  // Download follows the same scope and ignores pagination without losing any match.
  await go(page,url('profile','search')+'?state='+encodeURIComponent(JSON.stringify({skip:1,limit:1,filters:{name:'=1+1'}})));
  const downloadPromise=page.waitForEvent('download');await page.locator('#export-csv').click();const download=await downloadPromise;
  const downloadPath=path.join(output,`download-${tag}.csv`);await download.saveAs(downloadPath);
  const exported=fs.readFileSync(downloadPath,'utf8');assert.match(exported,/'=1\+1/);assert.doesNotMatch(exported,/secret|ADMIN_PRIVATE/);
  assert.match(download.suggestedFilename(),/profile-export\.csv/);
  assert.equal(exported.split('\r\n').filter(Boolean).length,2);
  // Export guards round-trip exactly only when the explicit option is enabled.
  await go(page,url('profile','import'));await file(exported);await preview();
  cells=await page.locator('#import-preview-table tbody tr').first().locator('td pre').allTextContents();assert.equal(JSON.parse(cells[1]).name,"'=1+1");
  await page.locator('#import-decode').check();await preview();
  cells=await page.locator('#import-preview-table tbody tr').first().locator('td pre').allTextContents();assert.equal(JSON.parse(cells[1]).name,'=1+1');
  // Lost acknowledgement follows an actual durable batch; this mounted form becomes terminal.
  await file('id,name\nu,Uncertain');await preview();faultNext=true;
  const before=calls.filter(call=>call.pathname.endsWith('/confirm')).length;await confirm();await frame(page);
  assert.match(await page.locator('#import-status').textContent(),/^Write outcome is unknown/);
  assert.doesNotMatch(await page.locator('#import-status').textContent(),/^Imported/);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  assert.equal(await page.locator('#import-preview').isDisabled(),true);
  assert.equal(await page.locator('#admin-import-file').isDisabled(),true);
  assert.equal(calls.filter(call=>call.pathname.endsWith('/confirm')).length,before+1);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'u'}}).status,200);
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
      const method=request.method();const headers=await request.allHeaders();
      const transfer=target.pathname.endsWith('/preview')||target.pathname.endsWith('/confirm');
      const body=transfer?{csv:request.postData()||'',decode_export_guards:headers['x-admin-decode-export-guards']==='1',confirm:headers['x-admin-confirmed']==='true'}:request.postData()?JSON.parse(request.postData()):undefined;
      calls.push({method,pathname:target.pathname});
      const fault=faultNext && method==='POST';if(fault)faultNext=false;
      delete headers['content-type'];delete headers['content-length'];delete headers['x-admin-decode-export-guards'];delete headers['x-admin-confirmed'];
      // Actor/fault/database are trusted harness inputs, separate from browser payloads.
      const result=native(target.pathname+target.search,method,body,{headers,fault});
      return route.fulfill({status:result.status,headers:result.headers,body:result.body});
    });
    await Promise.race([flow(page),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('inner browser deadline')),105000);})]);
    fs.writeFileSync(path.join(output,`browser-${tag}.json`),JSON.stringify({config,calls,errors,unexpected,expectedHttpDiagnostics,passed:true},null,2));
    console.log(JSON.stringify({config,requests:calls.length,passed:true}));
  } finally {releaseBoot();clearTimeout(timer);if(browser)await browser.close();fs.rmSync(directory,{recursive:true,force:true});}
})().catch(error=>{console.error(error);process.exitCode=1;});
