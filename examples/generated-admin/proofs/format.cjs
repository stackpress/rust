// Scenario assertions are separate from the shared listener-free HTTP/browser harness.
const assert = require('node:assert/strict');
module.exports = async function flow(h, page) {
  const { config, calls, errors, unexpected, releaseBoot, native,
    url, frame, snapshot, draft, status, settled, go, click, picture } = h;

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
    releaseBoot();await page.waitForFunction(()=>window.adminReady===true);
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
  const initialCalls=calls.length;releaseBoot();
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
  const gaps=await page.locator('.admin-actions').evaluate(node=>{
    const boxes=[...node.children].map(node=>node.getBoundingClientRect());
    return boxes.slice(1).map((b,index)=>{const a=boxes[index];return b.top>=a.bottom ? b.top-a.bottom : b.left-a.right;});
  });assert.ok(gaps.every(gap=>gap>=8),`pagination action gaps ${gaps}`);
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
  assert.equal(await page.locator('#admin-field-name-error').textContent(),'Submitted version: Name </script> 🦀 is required');
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
  // Retain the original omitted-slider bridge proof in its current filter placement.
  await page.locator('#admin-search-filter-priority-presence').selectOption('missing');await frame(page);
  await page.getByRole('button',{name:'Search',exact:true}).click();await settled(page);
  let zeroLink=await page.locator('#search-deep-link').getAttribute('href');
  assert.equal('priority' in JSON.parse(new URL(zeroLink,'https://admin.invalid').searchParams.get('state')).filters,false);
  await page.locator('#admin-search-filter-priority-0').evaluate(node=>node.dispatchEvent(new Event('input',{bubbles:true})));await frame(page);
  await page.getByRole('button',{name:'Search',exact:true}).click();await settled(page);
  zeroLink=await page.locator('#search-deep-link').getAttribute('href');
  assert.equal(JSON.parse(new URL(zeroLink,'https://admin.invalid').searchParams.get('state')).filters.priority,0,'a native edit at displayed zero supplies a concrete value');
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
  h.actor='reader';await go(page,url('profile','search'));assert.equal(await page.locator('[data-admin-action="create"]').count(),0);
  assert.equal(await page.evaluate(target=>fetch(target).then(response=>response.status),url('profile','create')),403,'deep link enforces the same policy');
  h.actor='editor';await go(page,url('profile','create'));await page.locator('#admin-field-id').fill('denied');await page.locator('#admin-field-name').fill('Preserved denied draft');
  h.actor='reader';await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'operation is forbidden');
  assert.equal((await draft(page)).name,'Preserved denied draft');
  h.actor='editor';await go(page,url('profile','create'));await page.locator('#admin-field-id').fill('uncertain');await page.locator('#admin-field-name').fill('Preserved uncertain draft');
  h.faultNext=true;const writes=calls.filter(call=>call.method==='POST'&&call.pathname.endsWith('/create')).length;
  await page.getByRole('button',{name:'Save',exact:true}).click();await status(page,'Write outcome is unknown');await frame(page);
  assert.equal((await draft(page)).name,'Preserved uncertain draft');assert.equal(await page.getByRole('button',{name:'Save',exact:true}).isDisabled(),true);
  assert.equal(await page.locator('#operation-status').textContent().then(text=>text.includes('Saved')),false);
  assert.equal(calls.filter(call=>call.method==='POST'&&call.pathname.endsWith('/create')).length,writes+1);
  assert.equal(native(`${config.mount}api/profile/get`,'POST',{key:{id:'uncertain'}}).status,200,'unknown acknowledgement follows a real SQLite effect');
  await picture(page,'unknown');
  // Local validation must not release Pending/Unknown or replace an in-flight version.
  for(const outcome of ['saved','unknown']) {
    await go(page,url('task','create'));
    const identity='formatted-guard-'+outcome;
    await page.locator('#admin-field-id').fill(identity);
    await page.locator('#admin-field-projectId').fill('j1');
    await page.locator('#admin-field-title').fill('Submitted formatted draft');
    await page.locator('#admin-field-amount').fill('1.00');await page.locator('#admin-field-amount').press('Enter');await frame(page);
    const held=h.deferNext('/task/create');h.faultNext=outcome==='unknown';
    const before=calls.filter(call=>call.pathname.endsWith('/task/create')).length;
    await page.getByRole('button',{name:'Save',exact:true}).click();
    assert.equal((await held.acknowledgement).status,outcome==='saved'?200:500);
    try {
      assert.equal(await page.locator('#admin-field-amount').isEnabled(),true);
      await page.locator('#admin-field-amount').fill('-');await frame(page);
      await page.locator('#admin-form').dispatchEvent('submit');
      const pendingStatus=await page.locator('#operation-status').textContent();
      await page.locator('#admin-field-amount').fill('2.00');await page.locator('#admin-field-amount').press('Enter');await frame(page);
      await page.locator('#admin-form').dispatchEvent('submit');held.release();await settled(page);await frame(page);
      const message=await page.locator('#operation-status').textContent();
      const stored=JSON.parse(native(`${config.mount}api/task/get`,'POST',{key:{id:identity}}).body).results;
      console.log(JSON.stringify({formattedGuard:outcome,pendingStatus,submitted:1,current:(await draft(page)).amount,stored:stored.amount,message}));
      await picture(page,'formatted-guard-'+outcome);
      assert.equal(pendingStatus,'Submitting…');
      assert.equal(stored.amount,1);assert.equal((await draft(page)).amount,2);
      if(outcome==='saved')assert.equal(message,'Saved the submitted version. You have newer unsaved changes.');
      else {
        assert.match(message,/unknown for the submitted version.*newer unsaved changes/);
        await page.locator('#admin-field-amount').fill('-');await frame(page);await page.locator('#admin-form').dispatchEvent('submit');
        await page.locator('#admin-field-amount').fill('3.00');await page.locator('#admin-field-amount').press('Enter');await frame(page);
        assert.match(await page.locator('#operation-status').textContent(),/^Write outcome is unknown/);
        assert.equal(await page.getByRole('button',{name:'Save',exact:true}).isDisabled(),true);
      }
      assert.equal(calls.filter(call=>call.pathname.endsWith('/task/create')).length,before+1);
    } finally {held.release();}
  }
  assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
  await page.evaluate(()=>window.adminDispose());assert.equal(await page.locator('#app').textContent(),'');

};
