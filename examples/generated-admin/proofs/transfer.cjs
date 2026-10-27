// Scenario assertions are separate from the shared listener-free HTTP/browser harness.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
module.exports = async function flow(h, page) {
  const { config, output, tag, calls, errors, unexpected, releaseBoot, native,
    url, frame, go, picture } = h;

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
  const initialCalls=calls.length;releaseBoot();
  await page.waitForFunction(()=>window.adminReady===true);
  await page.evaluate(zoom=>{document.body.style.zoom=String(zoom);},config.zoom);
  if(config.mode==='hydrate')assert.equal(await page.evaluate(()=>window.beforeHydration===document.querySelector('#admin-import-file')),true);
  assert.equal(calls.length,initialCalls,'mount must not fetch or write');
  assert.equal(await page.locator('#import-decode').isChecked(),false);
  assert.match(await page.locator('#admin-import').textContent(),/1 MB per file, 100 rows, 100 columns, 64 KB per cell/);
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
  assert.equal(await page.locator('#import-preview-table').getByRole('columnheader',{name:'Raw cells',exact:true}).count(),1,'column headers remain in the accessibility tree');
  if(config.width===375) {
    const labels=await page.locator('#import-preview-table td').evaluateAll(nodes=>nodes.map(node=>({label:node.dataset.label,before:getComputedStyle(node,'::before').content,display:getComputedStyle(node).display})));
    assert.deepEqual(labels.map(cell=>cell.label),['Raw cells','Decoded values','Errors']);
    assert.ok(labels.every(cell=>cell.display==='block'&&cell.before.includes(cell.label)),'stacked cells retain visible labels');
  }
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
  await file('id,name\ndenied,Kept draft');h.actor='reader';await preview();
  assert.match(await page.locator('#import-status').textContent(),/forbidden/);assert.equal(await page.locator('#import-file-name').textContent(),'records.csv');
  assert.equal(native(`${config.mount}profile/import`).status,403);h.actor='editor';
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
  // A replaced file (even identical bytes), or toggling decode away and back, needs a fresh preview.
  await file('id,name\nversioned,Submitted file');await preview();
  assert.match(await page.locator('#import-row-count').textContent(),/^1 row;/);
  await file('id,name\nversioned,Submitted file');await frame(page);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  await preview();await page.locator('#import-decode').uncheck();await page.locator('#import-decode').check();await frame(page);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  await preview();await page.locator('#import-confirm-check').check();
  const held=h.deferNext('/confirm');await page.locator('#import-confirm').click();
  assert.equal((await held.acknowledgement).status,200);
  assert.equal(await page.locator('#admin-import-file').isEnabled(),true);
  await file('id,name\nnewer,Newer unsaved file');held.release();
  await page.waitForFunction(()=>document.querySelector('#import-status').textContent.includes('Imported 1 row from the submitted file'));
  assert.match(await page.locator('#import-status').textContent(),/newer unsaved changes/);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'newer'}}).status,404);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'versioned'}}).status,200);
  await picture(page,'newer-file-after-confirm');
  // Lost acknowledgement follows an actual durable batch; this mounted form becomes terminal.
  await file('id,name\nu,Uncertain');await preview();h.faultNext=true;
  const before=calls.filter(call=>call.pathname.endsWith('/confirm')).length;
  const uncertainHold=h.deferNext('/confirm');
  await page.locator('#import-confirm-check').check();await page.locator('#import-confirm').click();
  assert.equal((await uncertainHold.acknowledgement).status,500);
  await file('id,name\nlater-uncertain,Newer unsaved file','newer-uncertain.csv');uncertainHold.release();
  await page.waitForFunction(()=>document.querySelector('#import-status').textContent.includes('Write outcome is unknown'));await frame(page);
  assert.match(await page.locator('#import-status').textContent(),/^Write outcome is unknown/);
  assert.match(await page.locator('#import-status').textContent(),/submitted file.*newer unsaved changes/);
  assert.equal(await page.locator('#import-file-name').textContent(),'newer-uncertain.csv');
  assert.doesNotMatch(await page.locator('#import-status').textContent(),/^Imported/);
  assert.equal(await page.locator('#import-confirm').isDisabled(),true);
  assert.equal(await page.locator('#import-preview').isDisabled(),true);
  assert.equal(await page.locator('#admin-import-file').isDisabled(),true);
  assert.equal(calls.filter(call=>call.pathname.endsWith('/confirm')).length,before+1);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'u'}}).status,200);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'later-uncertain'}}).status,404);
  await picture(page,'unknown');
  assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
  await page.evaluate(()=>window.adminDispose());assert.equal(await page.locator('#app').textContent(),'');

};
