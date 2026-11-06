// Actual generated WASM roles; every request is answered in-process, without a listener.
const fs=require('node:fs');const path=require('node:path');const assert=require('node:assert/strict');
const cached='/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright';
assert.equal(process.env.SOURCE_ROLES_PLAYWRIGHT,cached);const {chromium}=require(cached);const out=process.argv[2];

async function prove(page,mode,width,theme){
  const errors=[];const requests=[];
  page.on('pageerror',error=>{errors.push(String(error));console.error(String(error));});
  page.on('console',message=>{if(message.type()==='error'||/hydration.*(?:mismatch|error)/i.test(message.text()))errors.push(message.text());});
  await page.route('**/*',route=>{
    const url=new URL(route.request().url());requests.push({method:route.request().method(),path:url.pathname});
    if(url.pathname==='/')return route.fulfill({contentType:'text/html',body:fs.readFileSync(path.join(out,'page.html'),'utf8').replace('data-theme="light"',`data-theme="${theme}"`)});
    if(url.pathname==='/start.js')return route.fulfill({contentType:'text/javascript',body:`import init,{start} from '/source_roles.js';await init('/source_roles_bg.wasm');${mode==='csr'?"document.getElementById('app').replaceChildren();":''}window.disposeRoles=start();window.rolesReady=true;`});
    if(['/source_roles.js','/source_roles_bg.wasm'].includes(url.pathname))return route.fulfill({contentType:url.pathname.endsWith('.wasm')?'application/wasm':'text/javascript',body:fs.readFileSync(path.join(out,mode,path.basename(url.pathname)))});
    if(url.pathname==='/picture.svg')return route.fulfill({contentType:'image/svg+xml',body:'<svg xmlns="http://www.w3.org/2000/svg" width="160" height="100"><rect width="160" height="100" fill="#64748b"/></svg>'});
    return route.fulfill({status:200,body:''});
  });
  await page.goto('https://roles.invalid/',{waitUntil:'networkidle'});await page.waitForFunction(()=>window.rolesReady);
  const form=page.locator('#generated-field');const control=name=>page.locator(`#generated-field-${name}`);
  const snapshot=async()=>JSON.parse(await page.locator('#snapshot').textContent());
  const data=async()=>page.evaluate(()=>Object.fromEntries(new FormData(document.querySelector('#generated-field'))));
  assert.equal(await page.locator('#formulas').textContent(),'14|20|-1|inf|-inf|NaN|inf|0');

  // Code and JSON remain native editable text, including malformed JSON, with real tab selection.
  const code=control('f_code');await code.fill('abc');await code.evaluate(node=>{node.focus();node.setSelectionRange(1,2);});await page.keyboard.press('Tab');assert.equal(await code.inputValue(),'a\tc');assert.equal((await snapshot()).f_code,'a\tc');
  const json=control('f_json');await json.fill('{ invalid');assert.equal((await snapshot()).f_json,'{ invalid');
  const markdown=control('f_markdown');await markdown.fill('# Updated\n\n**preview**');
  const markdownRoot=markdown.locator('xpath=../..');await markdownRoot.getByRole('button',{name:'Preview Markdown',exact:true}).focus();await page.keyboard.press('Enter');
  assert.match(await markdownRoot.locator('iframe').getAttribute('srcdoc'),/<h1>Updated<\/h1>/);assert.equal(await markdown.isVisible(),false);
  await markdownRoot.getByRole('button',{name:'Edit Markdown',exact:true}).click();assert.equal(await markdown.isVisible(),true);

  // Color picker and free text synchronize while preserving the authored form spelling.
  const color=control('f_color');await color.fill('red');assert.equal((await data()).f_color,'red');assert.equal(await color.locator('xpath=..').locator('input[type=color]').inputValue(),'#ff0000');
  await color.locator('xpath=..').locator('input[type=color]').evaluate(node=>{node.value='#00ff00';node.dispatchEvent(new Event('input',{bubbles:true}));});assert.equal(await color.inputValue(),'#00ff00');

  // Shared source country/currency selectors use original data/search and actual native keys.
  for(const [name,query,label,codeValue] of [['f_country','United States','United States of America','US'],['f_currency','Philippine','Philippine Peso','PHP']]){
    const trigger=control(name);await trigger.click();const popup=page.locator(`#generated-field-${name}-options`);const search=popup.getByRole('textbox',{name:'Search choices'});await search.fill(query);
    const choices=popup.getByRole('option');assert.ok(await choices.count()>0,label);await search.press('ArrowDown');await search.press('Enter');
    assert.equal((await data())[name],codeValue);assert.equal((await snapshot())[name][name==='f_country'?'iso2':'code'],codeValue);
    await trigger.locator('xpath=..').getByRole('button',{name:'Clear selection',exact:true}).click();assert.equal(Object.hasOwn(await data(),name),false);
  }

  // Source suggestions keep all authored options; remote is a native attribute, never a fetch.
  const suggest=control('f_suggest');await suggest.fill('arbitrary');const suggestions=page.locator('#generated-field-f_suggest-suggestions');assert.equal(await suggestions.getByRole('option').count(),2);await suggest.press('ArrowDown');await suggest.press('Enter');assert.equal(await suggest.inputValue(),'Alpha');assert.equal((await snapshot()).f_suggest,'Alpha');
  const mask=control('f_mask');await mask.fill('1234');assert.equal(await mask.inputValue(),'12-34');assert.equal((await data()).f_mask,'12-34');await mask.press('End');await mask.press('Backspace');assert.equal(await mask.inputValue(),'12-3_');await mask.fill('1234');
  const small=control('f_small');await small.fill('99');assert.equal(await small.inputValue(),'9');await small.press('ArrowDown');assert.equal(await small.inputValue(),'8');assert.equal((await snapshot()).f_small,8);
  const radio=control('f_radio');await radio.focus();await page.keyboard.press('Space');assert.equal(await radio.isChecked(),true);
  await control('f_email').fill('person@example.test');assert.equal((await data()).f_email,'person@example.test');

  // Upload hooks receive native File objects; the component itself has no upload transport.
  for(const [fileIndex,name] of ['f_file','f_image','f_filelist','f_imagelist'].entries()){
    const picker=control(name);const image=name.includes('image');await picker.setInputFiles({name:image?'test.png':'test.txt',mimeType:image?'image/png':'text/plain',buffer:Buffer.from('local')});
    const parent=form.locator('.sp-source-files').nth(fileIndex);await parent.getByRole('button',{name:name.endsWith('list')?'Remove file':'Reset file',exact:true}).click();
  }
  assert.equal(JSON.parse(await page.locator('#uploads').textContent()).length,4);

  // Date/time/number lists intentionally stay text controls, retaining tombstoned row identities.
  for(const name of ['f_datelist','f_datetimelist','f_numberlist','f_timelist']){
    const list=form.locator(`input[name="${name}[]"]`).first().locator('xpath=../..');const rows=list.locator('.sp-source-text-row');const first=rows.first().locator('input[type=text]');assert.equal(await first.getAttribute('type'),'text');await first.fill('changed');await rows.first().getByRole('button',{name:'Remove row 1'}).click();await list.getByRole('button',{name:'Add',exact:true}).click();
    assert.equal(await rows.count(),2);assert.equal(await rows.last().locator('input[type=text]').inputValue(),'');
  }

  // Generated nested fieldsets use actual child functions and nested native form names.
  const nested=form.locator('input[name="f_fieldset[text]"]');await nested.fill('nested changed');assert.equal((await snapshot()).f_fieldset.text,'nested changed');
  const collection=form.locator('input[name="f_fieldsets[0][text]"]').locator('xpath=../../../..');await form.locator('input[name="f_fieldsets[0][text]"]').fill('row changed');assert.equal((await snapshot()).f_fieldsets[0].text,'row changed');
  assert.ok(await form.locator('input[name="f_fieldsets[1][text]"]').count());
  const nestedGroup=form.getByRole('group',{name:'f_fieldsets',exact:true});await nestedGroup.getByRole('button',{name:'Add Detail',exact:true}).click();assert.ok(await form.locator('input[name="f_fieldsets[2][text]"]').count());await nestedGroup.getByRole('button',{name:'Remove row 1',exact:true}).click();assert.equal((await snapshot()).f_fieldsets.length,2);

  // Every filter/span selection is present in the generated consumer, with source form names.
  assert.ok(await page.locator('#generated-filter input[name="eq[f_email]"]').count());assert.ok(await page.locator('#generated-span input[name="f_email"]').count());
  assert.ok((await page.locator('#generated-view').textContent()).includes('5.5'));
  // Source carousel has no imposed arrow children: its native viewport owns keyboard scrolling.
  const carousel=page.locator('#generated-view .sp-source-carousel-viewport').first();
  await carousel.scrollIntoViewIfNeeded();
  await page.waitForFunction(()=>{const el=document.querySelector('#generated-view .sp-source-carousel-viewport');return el.scrollLeft>=el.scrollWidth-el.clientWidth-1;});
  assert.equal(await carousel.evaluate(el=>getComputedStyle(el).overflow),'auto');
  const lastFrame=await carousel.evaluate(el=>el.scrollLeft);assert.ok(lastFrame>0);
  await carousel.focus();await carousel.press('ArrowLeft');
  await page.waitForFunction(start=>document.querySelector('#generated-view .sp-source-carousel-viewport').scrollLeft<start,lastFrame);
  assert.equal(await page.locator('#generated-view .sp-source-film img').count(),2);
  assert.equal(requests.some(request=>request.method!=='GET'||request.path==='/never-fetch'),false);
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1),false,'horizontal overflow');
  await page.screenshot({path:path.join(out,`${mode}-${width}-${theme}.png`),fullPage:true});
  await control('f_code').locator('xpath=..').screenshot({path:path.join(out,`${mode}-${width}-${theme}-code.png`)});
  await page.locator('#radio-profile').screenshot({path:path.join(out,`${mode}-${width}-${theme}-radio.png`)});
  await form.locator('.sp-source-fieldset').first().screenshot({path:path.join(out,`${mode}-${width}-${theme}-collection.png`)});
  assert.equal(await page.locator('#profile-check').isChecked(),true);
  assert.equal(await page.locator('#radio-profile svg').getAttribute('width'),'16');
  await control('f_file').setInputFiles({name:'late.txt',mimeType:'text/plain',buffer:Buffer.from('pending')});
  await page.evaluate(()=>{window.disposeRoles();window.finishLateUpload();window.disposeRoles();window.lateFinished=true;});
  await page.waitForFunction(()=>window.lateFinished);
  assert.deepEqual(errors,[],`${mode} ${width} ${theme}`);
  return {mode,width,theme,errors,requests};
}

(async()=>{const browser=await chromium.launch({headless:true});const results=[];try{
  for(const mode of ['hydrate','csr'])for(const width of [375,1280])for(const theme of ['light','dark']){
    const context=await browser.newContext({viewport:{width,height:1000}});const page=await context.newPage();page.setDefaultTimeout(10000);results.push(await prove(page,mode,width,theme));await context.close();
  }
}finally{await browser.close();}fs.writeFileSync(path.join(out,'browser.json'),JSON.stringify(results,null,2));console.log(`PASS ${results.length} actual generated source-role hydrate/CSR theme/viewport cases`);
})().catch(error=>{console.error(error);process.exitCode=1;});
