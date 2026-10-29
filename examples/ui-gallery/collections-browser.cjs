// Real finite SSR + rebuilt WASM; collection-only fixture never enters owner routes.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
const {chromium} = require(process.env.GALLERY_PLAYWRIGHT);
const m3 = require('./table-rating-proof.cjs');
const output = process.env.GALLERY_OUTPUT_DIR;
const host = process.env.GALLERY_COLLECTION_HOST;
const frame = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const state = async page => JSON.parse(await page.locator('#collection-proof-state').textContent());
const focus = (page,id) => page.waitForFunction(id => document.activeElement?.id===id,id);

/** Compare rendered primitives with the exact vendored masters, including decorative semantics. */
async function iconGeometry(page) {
  const folder=path.resolve(__dirname,'../../crates/leptos/third-party/lucide');
  for(const [id,name] of [['proof-text-1-up','arrow-up'],['proof-text-1-down','arrow-down'],['proof-text-1-remove','trash'],['proof-tags-10-remove','x'],['proof-tags-add','plus']]) {
    const source=fs.readFileSync(path.join(folder,name+'.svg'),'utf8');
    const result=await page.locator('#'+id+' svg').evaluate((svg,source)=>{
      const shapes=root=>[...root.children].map(shape=>({tag:shape.localName,attributes:[...shape.attributes].map(attr=>[attr.name,attr.value]).sort()}));
      const master=new DOMParser().parseFromString(source,'image/svg+xml').documentElement;
      return {same:JSON.stringify(shapes(svg))===JSON.stringify(shapes(master)),hidden:svg.getAttribute('aria-hidden'),focusable:svg.getAttribute('focusable'),size:svg.getAttribute('width'),stroke:svg.getAttribute('stroke')};
    },source);
    assert.deepEqual(result,{same:true,hidden:'true',focusable:'false',size:'16',stroke:'currentColor'});
  }
}

/** Assert layout in the real component tree, including its local overflow boundary. */
async function geometry(page) {
  return page.evaluate(() => {
    const rect = node => node.getBoundingClientRect();
    const rows = [...document.querySelectorAll('.sp-collection-inline li')].map(row => {
      const fields = [...row.querySelectorAll('input')], actions = row.querySelector('.sp-collection-actions');
      return {aligned: fields.every(field => Math.abs((rect(field).top+rect(field).bottom)/2-(rect(actions).top+rect(actions).bottom)/2)<2),
        nowrap:getComputedStyle(actions).flexWrap==='nowrap', local:getComputedStyle(row).overflowX==='auto',
        overflowing:row.scrollWidth>row.clientWidth};
    });
    const add=document.querySelector('.sp-tag-add'), boundary=rect(add);
    return {rows,addInside:[...add.children].every(node=>rect(node).left>=boundary.left-1&&rect(node).right<=boundary.right+1),clipped:document.documentElement.scrollWidth>innerWidth+1,
      coarse:matchMedia('(pointer:coarse)').matches,
      smallest:Math.min(...[...document.querySelectorAll('.sp-icon-action')].map(node=>Math.min(rect(node).width,rect(node).height)))};
  });
}

/** Exercise distinct value, keyboard, composition, disabled and form boundaries. */
async function controls(page) {
  const draft=page.locator('#proof-tags-draft');
  assert.deepEqual((await state(page)).tags,[[10,''],[20,'0'],[30,'雪']]);
  assert.equal((await state(page)).changes,0);
  await page.getByRole('button',{name:'Remove tag (empty)',exact:true}).waitFor();
  assert.equal(await page.locator('#proof-tags [type=text],#proof-tags input:not([type])').count(),1);
  assert.equal(await page.locator('#proof-tags [id$="-up"],#proof-tags [id$="-down"]').count(),0);
  const fields=await page.locator('#collection-proof-form').evaluate(form=>[...new FormData(form)]);
  assert.deepEqual(fields.filter(([name])=>name.startsWith('proof-tags')), [['proof-tags[10]',''],['proof-tags[20]','0'],['proof-tags[30]','雪']]);
  await draft.press('Enter'); await frame(page);
  assert.equal((await state(page)).tags.at(-1)[1],'Early tag </script> 雪');
  assert.equal((await state(page)).changes,1,'a successful append publishes once');
  assert.equal(await page.locator('#proof-tags-add').isDisabled(),true);
  await draft.fill('capacity retained'); await draft.press('Enter');
  assert.equal(await draft.inputValue(),'capacity retained'); assert.equal((await state(page)).tags.length,4);
  await page.locator('#proof-tags-31-remove').click(); await focus(page,'proof-tags-30-remove');
  for(const blank of ['', ' \t ', '\u2003']) {
    const changes=(await state(page)).changes;
    await draft.fill(blank); await draft.press('Enter'); await page.locator('#proof-tags-add').click();
    assert.equal(await draft.inputValue(),blank); assert.equal((await state(page)).tags.length,3);
    assert.equal((await page.locator('#proof-tags-error').textContent()).trim(),'');
    assert.equal((await state(page)).changes,changes,'a blank draft publishes no collection update');
  }
  await draft.fill('0'); await draft.press('Enter');
  assert.deepEqual((await state(page)).tags,[[10,''],[20,'0'],[30,'雪'],[32,'0']]);
  await page.locator('#proof-tags-20-remove').click(); await focus(page,'proof-tags-30-remove');
  await page.locator('#proof-tags-32-remove').click(); await focus(page,'proof-tags-30-remove');
  const oversized='🦀'.repeat(1025);
  const changes=(await state(page)).changes;
  await draft.fill(oversized); await draft.press('Enter');
  assert.equal(await draft.inputValue(),oversized); assert.equal((await state(page)).tags.length,2);
  assert.match(await page.locator('#proof-tags-error').textContent(),/byte budget/);
  assert.equal((await state(page)).changes,changes,'a rejected draft publishes no collection update');
  const exact=' 0,雪 </script><img src=x onerror="window.pwned=1"> ';
  await draft.fill(exact); await page.locator('#proof-tags-add').click(); await focus(page,'proof-tags-draft');
  assert.equal((await state(page)).tags.at(-1)[1],exact); assert.equal((await page.locator('#proof-tags-error').textContent()).trim(),'');
  assert.equal(await page.locator('#proof-tags img, #proof-tags script').count(),0);
  assert.equal(await page.locator('#proof-tags-10-value').getAttribute('name'),'proof-tags[10]');
  await draft.fill('構成');
  await draft.dispatchEvent('compositionstart');
  await draft.dispatchEvent('keydown',{key:'Enter',isComposing:true});
  await draft.dispatchEvent('keydown',{key:'Enter',keyCode:229});
  await draft.dispatchEvent('keydown',{key:'Enter'});
  assert.equal((await state(page)).tags.length,3,'composition confirmation never appends');
  await draft.dispatchEvent('compositionend'); await draft.press('Enter');
  assert.equal((await state(page)).tags.at(-1)[1],'構成');
  await draft.press('Backspace'); await focus(page,'proof-tags-34-remove');
  assert.equal((await state(page)).tags.length,4,'Backspace only focuses the remove action');
  await page.keyboard.press('Space'); await focus(page,'proof-tags-33-remove');
  // Disabled state is controller-owned; even synthetic stale events cannot mutate it.
  await m3.rating(page,'proof-rating',2);
  assert.equal((await state(page)).rating,2);
  await draft.fill('disabled draft'); await page.locator('#proof-disabled').click();
  const before=await state(page);
  for(const id of ['proof-tags-add','proof-tags-33-remove','proof-text-1-down','proof-text-1-remove','proof-metadata-4-remove','proof-rating-5']) {
    await page.locator('#'+id).dispatchEvent('click');
  }
  await page.locator('#proof-rating-5').dispatchEvent('change');
  await draft.dispatchEvent('keydown',{key:'Enter'}); await frame(page);
  assert.deepEqual(await state(page),before);
  assert.equal(await page.locator('#collection-proof-form').evaluate(form=>[...new FormData(form)].length),0,'disabled controls do not submit');
  await page.locator('#proof-disabled').click();
  // Synthetic clicks can toggle a native radio property while its state guard rejects
  // publication. Return through real enabled controls before visual/form proofs.
  await page.locator('.sp-rating-input').getByRole('button',{name:'Clear rating',exact:true}).click();
  await page.locator('label[for=proof-rating-2]').click();
  await page.locator('#proof-error').click();
  assert.equal(await page.locator('#proof-rating-error').textContent(),'Choose a permitted tag');
  assert.equal(await page.locator('#proof-rating-error').locator('..').getAttribute('aria-invalid'),'true');
  assert.equal(await draft.getAttribute('aria-invalid'),'true');
  assert.match(await page.locator('#proof-tags-error').textContent(),/permitted tag/);
  // Preserve existing field nodes, names, empty and zero values while reordering.
  const first=page.locator('#proof-text-1-value'); await first.fill('');
  await first.evaluate(node=>{window.retainedCollection=node;});
  await page.getByRole('button',{name:'Move row 1 down',exact:true}).press('Enter');
  await focus(page,'proof-text-1-remove');
  assert.equal(await first.evaluate(node=>node===window.retainedCollection),true);
  assert.deepEqual((await state(page)).text,[[2,'0'],[1,'']]);
  await page.getByRole('button',{name:'Move row 1 up',exact:true}).press('Space');
  await focus(page,'proof-text-1-remove');
  assert.deepEqual((await state(page)).text,[[1,''],[2,'0']]);
  await page.locator('#proof-text-1-up').dispatchEvent('click');
  assert.deepEqual((await state(page)).text,[[1,''],[2,'0']],'a boundary movement is a no-op');
  await page.getByRole('button',{name:'Move row 1 down',exact:true}).press('Enter');
  await first.fill('🦀'.repeat(1025));
  assert.deepEqual((await state(page)).text,[[2,'0'],[1,'']]);
  assert.match(await page.locator('#proof-text-error').textContent(),/byte budget/);
  await first.fill('accepted'); await first.fill('');
  await page.locator('#proof-metadata-4-key').fill('🦀'.repeat(257));
  assert.deepEqual((await state(page)).metadata,[[4,'count','0']]);
  assert.match(await page.locator('#proof-metadata-error').textContent(),/byte budget/);
  await page.locator('#proof-metadata-4-key').fill('empty'); await page.locator('#proof-metadata-4-value').fill('');
  const submitted=await page.locator('#collection-proof-form').evaluate(form=>Object.fromEntries(new FormData(form)));
  assert.equal(submitted['rating-min'],'2','the visually hidden native radio submits the exact admitted value');
  assert.equal(submitted['proof-text[1]'],''); assert.equal(submitted['proof-text[2]'],'0');
  assert.equal(submitted['proof-metadata[4][value]'],''); assert.equal(submitted['proof-tags[33]'],exact);
  assert.equal((await state(page)).submissions,0,'tag Enter never submits its parent form');
  for(const key of [33,30,10]) {
    await page.locator('#proof-tags-'+key+'-remove').focus(); await page.keyboard.press('Enter');
  }
  await focus(page,'proof-tags-draft'); assert.deepEqual((await state(page)).tags,[]);
  await page.getByRole('button',{name:'Submit proof',exact:true}).click();
  assert.equal((await state(page)).submissions,1);
  // A long unbroken chip must wrap and keep its remove target reachable.
  await draft.fill('長'.repeat(400)); await draft.press('Enter');
  assert.equal((await state(page)).tags[0][0],35,'removed identities are not reused');
  assert.equal(await page.evaluate(()=>window.pwned),undefined);
}

/** Save icon masters at actual sizes/themes and representative active/disabled controls. */
async function contactSheet(browser) {
  const folder=path.resolve(__dirname,'../../crates/leptos/third-party/lucide');
  const names=['arrow-up','arrow-down','arrow-up-down','trash','x','plus','star'];
  const icons=names.map(name=>({name,svg:fs.readFileSync(path.join(folder,name+'.svg'),'utf8')}));
  icons.push({name:'circle anchor',svg:'<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/></svg>'});
  icons.push({name:'square anchor',svg:'<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/></svg>'});
  const page=await browser.newPage({viewport:{width:1280,height:760}});
  try {
    const rows=['#fff','#202124'].map(background=>`<section style="background:${background};color:${background==='#fff'?'#202124':'#fff'}"><h2>${background==='#fff'?'Light':'Dark'} · 16 / 24 px</h2><div class="icons">${icons.map(({name,svg})=>`<article><p>${name}</p>${[16,24].map(size=>svg.replace('width="24"',`width="${size}"`).replace('height="24"',`height="${size}"`)).join(' ')}<p><button aria-label="${name}">${svg.replace('width="24"','width="16"').replace('height="24"','height="16"')}</button><button disabled>${svg.replace('width="24"','width="16"').replace('height="24"','height="16"')}</button></p><p title="Enlarged geometry diagnostic">${svg.replace('width="24"','width="64"').replace('height="24"','height="64"')}</p></article>`).join('')}</div></section>`).join('');
    await page.setContent(`<style>body{font:16px system-ui;margin:0}section{padding:20px}.icons{display:flex;gap:16px}article{width:120px}button{border:1px solid currentColor;background:transparent;color:inherit;width:32px;height:32px;padding:6px}button:disabled{opacity:.4}</style>${rows}`);
    await page.screenshot({path:path.join(output,'screenshots/collection-icons.png'),fullPage:true});
  } finally {await page.close();}
}

/** Each real render mode/mount owns its context; every failure closes the browser. */
/** Preserve the extracted missing/null/false/true, form and reentrant-factory proofs. */
async function nestedPresence(page,mode) {
  const value=page.locator('#nested-8-value');
  assert.equal(await value.inputValue(),mode==='hydrate'?'true':'false');
  if(mode==='hydrate')assert.equal(await value.evaluate(node=>node===window.earlyNested),true);
  for(const token of ['missing','null','false','true']) {
    await value.selectOption(token);
    await page.waitForFunction(token=>JSON.parse(document.getElementById('nested-proof-state').textContent)[0][1]===token,token);
    assert.deepEqual(await page.locator('#nested-proof-form').evaluate(form=>[...new FormData(form)]),[['nested[8]',token]]);
  }
  await page.locator('#nested-rows-add').click();
  assert.equal(await page.locator('#nested-9-value').inputValue(),'missing','reentrant factory runs outside its collection write guard');
  await page.locator('#nested-rows').getByRole('button',{name:'Set null',exact:true}).first().click();
  assert.equal(await value.inputValue(),'null');
  assert.deepEqual(await page.locator('#nested-proof-form').evaluate(form=>[...new FormData(form)]),[['nested[8]','null'],['nested[9]','missing']]);
}

async function main() {
  fs.mkdirSync(path.join(output,'screenshots'),{recursive:true});
  const browser=await chromium.launch({headless:true}); const receipts=[];
  try {
    for(const mode of ['hydrate','csr']) for(const base of ['', '/demo/v1']) {
      const name=`collections-${base?'mounted':'root'}-${mode}`;
      const context=await browser.newContext({viewport:{width:1280,height:900},hasTouch:!!base});
      try {
        const page=await context.newPage(); page.setDefaultTimeout(10000);
        const errors=[],unexpected=[]; let release; const boot=new Promise(resolve=>{release=resolve;});
        page.on('pageerror',error=>errors.push(error.message));
        page.on('console',message=>{if(message.type()==='error')errors.push(message.text());});
        const html=cp.execFileSync(host,mode==='csr'?['--csr']:[],{encoding:'utf8',timeout:10000});
        await page.route('**/*',async route=>{
          const url=new URL(route.request().url());
          if(url.origin!=='https://collections.invalid'){unexpected.push(url.href);return route.abort();}
          if(url.pathname===base+'/proof')return route.fulfill({body:html,contentType:'text/html'});
          if(url.pathname===base+'/assets/collections.js') {
            await boot;
            return route.fulfill({body:"import init,{start_collections} from './gallery.js';await init();window.collectionDispose=start_collections();window.collectionReady=true;",contentType:'text/javascript'});
          }
          const assets=path.join(output,'gallery-assets',mode), file=path.resolve(assets,path.basename(url.pathname));
          return route.fulfill({path:file,contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript'});
        });
        await page.goto('https://collections.invalid'+base+'/proof',{waitUntil:'commit'});
        if(mode==='hydrate') {
          await page.locator('#nested-8-value').selectOption('true');
          await page.locator('#nested-8-value').evaluate(node=>{window.earlyNested=node;});
          await page.locator('#proof-tags-draft').fill('Early tag </script> 雪');
          await page.locator('#proof-tags-draft').evaluate(node=>{window.earlyTagDraft=node;window.beforeCollections=document.getElementById('app').innerHTML;});
        }
        release(); await page.waitForFunction(()=>window.collectionReady===true);
        if(mode==='hydrate') {
          assert.equal(await page.evaluate(()=>beforeCollections===document.getElementById('app').innerHTML),true,'native draft property is adopted without replacing SSR markup');
          assert.equal(await page.locator('#proof-tags-draft').evaluate(node=>node===window.earlyTagDraft),true);
        } else await page.locator('#proof-tags-draft').fill('Early tag </script> 雪');
        await nestedPresence(page,mode);
        await iconGeometry(page);
        await controls(page);
        const layouts=[];
        for(const width of [1280,375]) for(const zoom of [1,2]) {
          await page.setViewportSize({width,height:900}); await page.evaluate(zoom=>{document.documentElement.style.zoom=zoom;},zoom);
          const result=await geometry(page); assert.equal(result.clipped,false,JSON.stringify(result));
          assert.equal(result.addInside,true,JSON.stringify(result));
          assert.equal(result.rows.every(row=>row.aligned&&row.nowrap&&row.local),true,JSON.stringify(result));
          assert.ok(result.smallest >= (result.coarse?44:32)*zoom-1,JSON.stringify(result));
          if(width===375&&zoom===2) {
            assert.equal(result.rows.some(row=>row.overflowing),true,'extreme narrow rows scroll locally');
            await page.locator('#proof-metadata-4-key').focus();
            await page.keyboard.press('Tab'); await page.keyboard.press('Tab');
            await focus(page,'proof-metadata-4-remove');
            assert.equal(await page.locator('#proof-metadata li').evaluate(row=>{
              const outer=row.getBoundingClientRect(),button=row.querySelector('[id$="-remove"]').getBoundingClientRect();
              return row.scrollLeft>0&&button.left>=outer.left-1&&button.right<=outer.right+1;
            }),true,'keyboard focus reveals the offscreen remove action inside its own row');
          }
          if(!(width===375&&zoom===2))await page.locator('#proof-metadata-4-value').scrollIntoViewIfNeeded();
          await page.screenshot({path:path.join(output,'screenshots',`${name}-${width}-zoom${zoom}.png`)});
          await page.locator('.sp-rating-input').screenshot({path:path.join(output,'screenshots',`${name}-rating-min2-${width}-zoom${zoom}.png`)});
          layouts.push({width,zoom,...result});
        }
        await page.locator('.sp-tag-list').evaluate(node=>{node.style.maxWidth='8rem';});
        assert.equal((await geometry(page)).addInside,true,'a nested narrow container keeps both add controls inside its bounds');
        await page.locator('.sp-tag-list').evaluate(node=>{node.style.maxWidth='';});
        assert.deepEqual(errors,[]); assert.deepEqual(unexpected,[]);
        await page.evaluate(()=>window.collectionDispose());
        assert.equal(await page.locator('#app').textContent(),'');
        receipts.push({mode,base,layouts,errors,unexpected});
        fs.writeFileSync(path.join(output,'collections-browser.json'),JSON.stringify(receipts,null,2));
        console.log('PASS',name);
      } finally {await context.close();}
    }
    await contactSheet(browser);
  } finally {await browser.close();}
}
main().catch(error=>{console.error(error);process.exitCode=1;});
