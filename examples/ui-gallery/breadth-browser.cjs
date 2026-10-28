// Finite M2 browser proof, using the real native host through intercepted .invalid requests.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const { request } = require('./browser.cjs');
const m3 = require('./table-rating-proof.cjs');
const moduleName = process.env.GALLERY_PLAYWRIGHT || 'playwright';
const { chromium } = require(moduleName);
const output = process.env.GALLERY_OUTPUT_DIR || '/private/tmp/s8-m0';
const receipts = [];
const frame = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const opened = (page,id) => page.waitForFunction(id => document.getElementById(id)?.matches(':popover-open'),id);
const closed = (page,id) => page.waitForFunction(id => !document.getElementById(id)?.matches(':popover-open'),id);
const inspect = async page => {
  await page.locator('#breadth-inspect').evaluate(node => node.click()); await frame(page);
  return page.locator('#breadth-state').evaluate(node => JSON.parse(node.textContent));
};

// Track only application window listeners/timers; Playwright's isolated world is separate.
async function monitor(page) {
  await page.addInitScript(() => {
    const listeners = new Map(), timers = new Map(), observers = new Set();
    const OriginalObserver = window.ResizeObserver;
    window.ResizeObserver = class extends OriginalObserver {
      observe(...args) { observers.add(this);return super.observe(...args); }
      disconnect() { observers.delete(this);return super.disconnect(); }
    };
    const add = window.addEventListener, remove = window.removeEventListener;
    window.addEventListener = function(type, callback, options) {
      if (type === 'scroll' || type === 'resize') {
        let registered = listeners.get(callback); if (!registered) listeners.set(callback, registered = new Set());
        registered.add(type);
      }
      return add.call(this,type,callback,options);
    };
    window.removeEventListener = function(type, callback, options) {
      listeners.get(callback)?.delete(type);
      return remove.call(this,type,callback,options);
    };
    const schedule = window.setTimeout, cancel = window.clearTimeout;
    window.setTimeout = function(callback, delay, ...args) {
      let id; id = schedule.call(this, (...values) => { timers.delete(id); callback(...values); }, delay, ...args);
      timers.set(id,delay); return id;
    };
    window.clearTimeout = function(id) { timers.delete(id); return cancel.call(this,id); };
    window.breadthLeases = () => ({ listeners:[...listeners.values()].reduce((sum,entries) => sum+entries.size,0), timers:timers.size, observers:observers.size });
  });
}

// Assert the same gap token on a row and after wrapping into vertical rows.
async function actionGaps(page, label, zoom=1) {
  const group=page.getByRole('group',{name:label,exact:true});
  const result=await group.evaluate((node,zoom) => {
    const style=getComputedStyle(node),buttons=[...node.children].filter(child => child.tagName==='BUTTON');
    const token={x:parseFloat(style.columnGap)*zoom,y:parseFloat(style.rowGap)*zoom};
    const pairs=buttons.slice(1).map((button,index) => {
      const a=buttons[index].getBoundingClientRect(),b=button.getBoundingClientRect();
      const vertical=b.top>=a.bottom-1;
      return {axis:vertical?'y':'x',gap:vertical?b.top-a.bottom:b.left-a.right};
    });
    return {token,pairs};
  },zoom);
  assert.equal(result.token.x>0 && result.token.y>0,true,`explicit two-axis gap ${label}`);
  for(const pair of result.pairs) assert.equal(pair.gap+1>=result.token[pair.axis],true,`${label} ${pair.axis}: ${JSON.stringify(result)}`);
  return result;
}

async function controls(page, mode) {
  const initialUrl = page.url();
  // Native/controlled disclosures retain their descendants while hidden.
  assert.equal(await page.getByRole('button',{name:'Blocked',exact:true}).count() >= 1,true);
  await page.locator('#acc-gamma-button').click(); assert.equal(await page.locator('#acc-gamma-button').getAttribute('aria-expanded'),'true');
  assert.equal(await page.locator('#acc-alpha-button').getAttribute('aria-expanded'),'true');
  await page.locator('#acc-alpha-button').press('Space'); assert.equal(await page.locator('#acc-alpha-panel').isVisible(),false);
  assert.equal(await page.locator('#acc-disabled-button').isDisabled(),true);
  await page.locator('#retained-tab-input').fill('Keep user draft');await page.locator('#retained-tab-input').evaluate(node => {window.tabDraft=node;});
  await page.locator('#manual-tabs-tab-0').focus(); await page.keyboard.press('ArrowRight');
  assert.equal(await page.locator('#manual-tabs-tab-2').evaluate(node => document.activeElement === node),true);
  assert.equal(await page.locator('#manual-tabs-tab-0').getAttribute('aria-selected'),'true');
  await page.keyboard.press('Enter'); assert.equal(await page.locator('#manual-tabs-panel-2').isVisible(),true);
  await page.keyboard.press('End'); await page.keyboard.press('Space'); assert.equal(await page.locator('#manual-tabs-panel-3').isVisible(),true);
  await page.keyboard.press('Home'); await page.keyboard.press('Enter');
  assert.equal(await page.locator('#retained-tab-input').inputValue(),'Keep user draft');
  assert.equal(await page.locator('#retained-tab-input').evaluate(node => node===window.tabDraft),true);
  await page.locator('#auto-tabs-tab-0').focus(); await page.keyboard.press('ArrowDown');
  assert.equal(await page.locator('#auto-tabs-tab-2').getAttribute('aria-selected'),'true');
  for (const selector of ['[role=tab][aria-controls]','[role=tabpanel][aria-labelledby]']) {
    assert.equal(await page.locator(selector).evaluateAll(nodes => nodes.every(node => document.getElementById(node.getAttribute(node.hasAttribute('aria-controls')?'aria-controls':'aria-labelledby')))),true);
  }
  assert.equal(await page.getByRole('navigation',{name:'Breadcrumb',exact:true}).locator('[aria-current=page]').textContent(),'Gallery');
  assert.equal(await page.getByRole('navigation',{name:'Breadcrumb',exact:true}).locator('a').count(),1);

  // Menus have real roving focus, disabled admission, incremental search and menubar transitions.
  await page.locator('#menu-trigger').click(); await opened(page,'command-menu');
  await page.keyboard.press('ArrowDown'); assert.equal(await page.locator('#commands-item-2').evaluate(node => document.activeElement === node),true);
  await page.keyboard.press('g'); await page.keyboard.press('a'); assert.equal(await page.locator('#commands-item-3').evaluate(node => document.activeElement === node),true);
  await page.keyboard.press('Enter'); await closed(page,'command-menu'); assert.equal(await page.locator('#menu-result').textContent(),'gamut');
  assert.equal(await page.locator('#menu-trigger').evaluate(node => document.activeElement === node),true);
  await page.locator('#bar-trigger-0').focus(); await page.keyboard.press('ArrowRight'); assert.equal(await page.locator('#bar-trigger-1').evaluate(node => document.activeElement === node),true);
  await page.keyboard.press('ArrowDown'); await opened(page,'bar-popup-1'); await page.keyboard.press('ArrowLeft'); await opened(page,'bar-popup-0');
  await page.keyboard.press('Escape'); await closed(page,'bar-popup-0');
  await page.locator('#menu-trigger').click(); await opened(page,'command-menu'); await page.keyboard.press('Tab'); await closed(page,'command-menu');
  assert.equal(await page.locator('#menu-trigger').evaluate(node => document.activeElement === node),false,'Tab must not be trapped on the opener');

  // Combobox focus stays on the input; exact empty and zero values remain selectable.
  await page.locator('#combo').fill('Ga'); await page.locator('#combo').press('ArrowDown'); await opened(page,'combo-list');
  const active = await page.locator('#combo').getAttribute('aria-activedescendant'); assert.equal(await page.locator('#'+active).getAttribute('role'),'option');
  await page.locator('#combo').press('Enter'); assert.equal((await inspect(page)).selected,'0');
  await page.locator('#combo').fill('Al'); await page.locator('#combo').press('ArrowDown'); await page.locator('#combo').press('Enter'); assert.equal((await inspect(page)).selected,'');
  assert.equal(await page.locator('#collection-form').evaluate(form => new FormData(form).get('combo-value')),'','selected empty is a successful native field');
  await page.locator('#combo').evaluate(node => {node.value='雪'.repeat(342);node.dispatchEvent(new Event('input',{bubbles:true}));}); await frame(page);
  assert.equal(await page.locator('#collection-form').evaluate(form => new FormData(form).has('combo-value')),false,'an oversized native edit cannot submit a stale selection');
  assert.equal(await page.locator('#combo-error').textContent(),'Query exceeds the field byte budget'); assert.equal(await page.locator('#combo').inputValue(),'Alpha');
  await page.locator('#combo').press('ArrowDown'); await page.locator('#combo').press('Enter'); assert.equal(await page.locator('#combo-error').textContent(),'','valid selection clears the local admission error');
  await page.locator('#combo').fill('No result'); await opened(page,'combo-list'); assert.equal(await page.locator('#combo-list').getByRole('status').textContent(),'No suggestions'); await page.locator('#combo').press('Escape');
  assert.equal(await page.locator('#suggest-calls').textContent(),'0','no initial suggestion acquisition');
  await page.locator('#suggest').fill('slow'); await page.locator('#suggest').fill('Grace');
  await page.locator('#suggest-list').getByRole('option',{name:'Grace Hopper',exact:true}).waitFor();
  await page.locator('#suggest').press('ArrowDown'); await page.locator('#suggest').press('Enter');
  assert.equal(await page.locator('input[name=suggest-value]').inputValue(),'grace');assert.equal(await page.locator('#suggest-calls').textContent(),'2','one acquisition per explicit edit');
  await page.locator('#suggest').fill('bad'); await page.waitForFunction(() => document.querySelector('#suggest-list [role=status]')?.textContent === 'Suggestions unavailable');
  await page.locator('#suggest').press('Escape');

  // Keyed collection nodes/drafts survive edits and reorder, with capacity/identity bounds.
  await page.locator('#text-rows-1-value').fill('Edited row one');
  await page.locator('#text-rows-1-value').evaluate(node => { window.rowOne=node; });
  await page.locator('#text-rows-1-down').click();
  assert.equal(await page.locator('#text-rows-1-value').evaluate(node => window.rowOne === node),true);
  assert.equal(await page.locator('#text-rows-1-value').inputValue(),'Edited row one');
  assert.deepEqual((await inspect(page)).text,['0','Edited row one']);
  await page.locator('#text-rows-add').click(); assert.equal(await page.locator('#text-rows-add').isDisabled(),true); await page.waitForFunction(() => document.activeElement?.id==='text-rows-3-remove');
  await page.locator('#text-rows-3-value').fill('Third'); await page.locator('#text-rows-1-remove').click(); await page.waitForFunction(() => document.activeElement?.id==='text-rows-add');
  await page.locator('#text-rows-add').click(); assert.equal(await page.locator('#text-rows-4-value').count(),1); assert.equal(await page.locator('#text-rows-1-value').count(),0);
  // Tags are plain chips; changing one means remove then append an explicit draft.
  await page.locator('#tag-rows-10-remove').click();
  await page.locator('#tag-rows-draft').fill('0'); await page.locator('#tag-rows-draft').press('Enter');
  assert.deepEqual((await inspect(page)).tags,['雪','0']);
  assert.equal(await page.locator('#tag-rows-21-value').inputValue(),'0');
  assert.equal(await page.locator('#tag-rows [id$="-up"],#tag-rows [id$="-down"]').count(),0);
  await page.locator('#metadata-rows-4-key').fill('empty'); await page.locator('#metadata-rows-4-value').fill('');

  const fields = await page.locator('#collection-form').evaluate(form => Object.fromEntries(new FormData(form)));
  assert.equal(fields['text-rows[2]'],'0'); assert.equal(fields['tag-rows[21]'],'0');
  assert.equal(fields['metadata-rows[4][value]'],'');
  assert.equal(fields['combo-value'],undefined); assert.equal(fields['suggest-value'],undefined,'editing after selection clears stale machine values');

  // Table paging, filter/sort/selection and same-key replacement are actual controlled behavior.
  const table = page.getByRole('table',{name:'Paged people',exact:true}); const pages=page.getByRole('navigation',{name:'Paged people pages',exact:true});
  const region=page.getByRole('region',{name:'Paged people',exact:true});
  await region.evaluate(node => {node.style.height='100px';node.scrollTop=1000;}); await frame(page);
  assert.equal(await region.evaluate(node => {const top=node.getBoundingClientRect().top;return [...node.querySelectorAll('thead th')].every(head => getComputedStyle(head).position==='sticky' && Math.abs(head.getBoundingClientRect().top-top)<=2);}),true,'all table headers stick inside the labelled overflow');
  await region.evaluate(node => {node.style.height='';node.scrollTop=0;});
  assert.equal(await table.locator('tbody tr').count(),12);
  const selectPage=page.getByLabel('Select page',{exact:true});
  await selectPage.check(); assert.equal(await table.locator('tbody input:checked').count(),12);
  await selectPage.uncheck();
  await table.getByRole('checkbox',{name:'Select row r0',exact:true}).check();
  await pages.getByRole('button',{name:'Next',exact:true}).click(); assert.equal(await table.locator('tbody tr').first().getAttribute('data-row-id'),'r12');
  await table.getByRole('checkbox',{name:'Select row r12',exact:true}).check(); await pages.getByRole('button',{name:'Previous',exact:true}).click();
  assert.equal(await table.getByRole('checkbox',{name:'Select row r0',exact:true}).isChecked(),true);
  await table.locator('tbody tr').first().evaluate(node => { window.retainedTableRow=node; });
  await table.getByRole('button',{name:'Name',exact:true}).focus(); await table.getByRole('button',{name:'Name',exact:true}).evaluate(node => {window.retainedTableHeader=node;});
  await page.locator('#update-table-row').evaluate(node => node.click()); await frame(page);
  assert.equal(await table.getByRole('button',{name:'Name',exact:true}).evaluate(node => node===window.retainedTableHeader && document.activeElement===node),true,'same-column replacement retains the focused header');
  assert.equal(await table.locator('tbody tr').first().evaluate(node => window.retainedTableRow === node),true);assert.equal(await table.locator('tbody tr').first().locator('td').nth(2).textContent(),'Updated 00');
  await table.getByRole('button',{name:'Name',exact:true}).click(); assert.equal(await table.locator('th[aria-sort=ascending]').count(),1);
  await table.getByRole('button',{name:'Name',exact:true}).click(); assert.equal(await table.locator('tbody tr').first().getAttribute('data-row-id'),'r47');
  await page.locator('#data-table-filter').fill('Updated 03'); assert.equal(await table.locator('tbody tr').count(),1); assert.equal(await pages.getByRole('button',{name:'Next',exact:true}).isDisabled(),true);
  await page.locator('#data-table-filter').fill('none'); assert.equal(await table.locator('tbody tr').count(),0); assert.equal(await page.getByText('No matching rows',{exact:true}).isVisible(),true);
  await page.locator('#data-table-filter').fill(''); await pages.getByRole('button',{name:'Next',exact:true}).click(); await page.locator('#shrink-table').click();
  assert.equal(await table.locator('tbody tr').count(),1); assert.equal(await pages.getByRole('button',{name:'Previous',exact:true}).isDisabled(),true);
  assert.equal(await page.getByRole('region',{name:'Paged people',exact:true}).getAttribute('tabindex'),'0');
  assert.equal(await page.getByRole('table',{name:'Explicit native table slots',exact:true}).locator('tfoot td[colspan="2"]').textContent(),'Summary');

  const ruler=page.getByRole('table',{name:'Explicit native table slots',exact:true}).locator('tbody td hr');
  assert.equal(await ruler.getAttribute('aria-hidden'),'true');
  assert.equal(await ruler.evaluate(node => getComputedStyle(node).width),'120px');
  assert.equal(page.url(),initialUrl,'local table paging never silently mutates browser history');

  // Range/rating stay native keyboard controls; slug editing preserves draft text until apply.
  await page.locator('#single-range-0').focus(); await page.keyboard.press('Home'); await page.keyboard.press('ArrowRight'); assert.equal(await page.locator('#single-range-0').inputValue(),'1');
  await page.keyboard.press('End'); assert.equal(await page.locator('#single-range-0').inputValue(),'10');
  const cap=await page.locator('#pair-range-1').inputValue();
  await page.locator('#pair-range-0').focus(); await page.keyboard.press('End'); assert.equal(await page.locator('#pair-range-0').inputValue(),cap);
  await page.locator('#pair-range-1').focus(); await page.keyboard.press('Home'); assert.equal(await page.locator('#pair-range-1').inputValue(),cap);
  await m3.rating(page,'rating');
  await page.locator('label[for=rating-0]').click(); assert.equal((await inspect(page)).rating,0); await page.locator('#rating-0').focus(); await page.keyboard.press('ArrowRight'); assert.equal((await inspect(page)).rating,1);
  await page.getByRole('button',{name:'Clear rating',exact:true}).click(); assert.equal((await inspect(page)).rating,null);
  await page.locator('#slug').fill(' Crème 雪 0 '); await page.getByRole('button',{name:'Apply slug',exact:true}).click(); assert.equal(await page.locator('#slug').inputValue(),'crème-雪-0');
  assert.equal(await page.evaluate(() => globalThis.pwned),undefined);
  return {nativeDisclosures:true,rovingMenusTabs:true,controlledCollections:true,pagingSelectionAndReplacement:true,asyncSuggestions:true,rangeRatingSlug:true,mode};
}

async function overlays(page,name) {
  // Nested native popovers close the top surface first; outside clicks retain new focus.
  await page.locator('#popup-trigger').click(); await opened(page,'breadth-popup'); await page.locator('#child-popup-trigger').click(); await opened(page,'child-popup');
  await page.keyboard.press('Escape'); await closed(page,'child-popup'); assert.equal(await page.locator('#breadth-popup').evaluate(node => node.matches(':popover-open')),true);
  await page.keyboard.press('Escape'); await closed(page,'breadth-popup');
  await page.locator('#popup-trigger').click(); await opened(page,'breadth-popup'); await page.locator('#outside-overlay').click(); await closed(page,'breadth-popup'); assert.equal(await page.locator('#outside-overlay').evaluate(node => document.activeElement === node),true);
  await page.locator('#popup-trigger').click(); await opened(page,'breadth-popup'); await page.locator('#popup-unmount').click(); await page.waitForFunction(() => !document.getElementById('breadth-popup'));
  await page.locator('#restore-overlays').click();
  // Native nonmodal explicitly permits background focus; drawers remain modal.
  await page.locator('#nonmodal-trigger').click(); await page.waitForFunction(() => document.getElementById('nonmodal').open); assert.equal(await page.locator('#nonmodal').evaluate(node => node.matches(':modal')),false);
  await page.locator('#nonmodal-child-trigger').click(); await page.waitForFunction(() => document.getElementById('nonmodal-child').open); await page.keyboard.press('Escape');
  await page.waitForFunction(() => !document.getElementById('nonmodal-child').open); assert.equal(await page.locator('#nonmodal').evaluate(node => node.open),true,'nested modeless Escape closes only the child');
  await page.locator('#nested-tip-trigger').focus(); await opened(page,'nested-tip-tip'); await page.keyboard.press('Escape'); await closed(page,'nested-tip-tip');
  assert.equal(await page.locator('#nonmodal').evaluate(node => node.open),true,'tooltip Escape leaves the containing modeless dialog open');
  await page.locator('#outside-overlay').focus(); assert.equal(await page.locator('#outside-overlay').evaluate(node => document.activeElement === node),true); await page.locator('#nonmodal').evaluate(node => node.close()); await page.waitForFunction(() => !document.getElementById('nonmodal').open); await frame(page);
  assert.equal(await page.locator('#outside-overlay').evaluate(node => document.activeElement===node),true,'modeless native close preserves unrelated background focus');
  await page.locator('#nonmodal-trigger').click(); await page.waitForFunction(() => document.getElementById('nonmodal').open); await page.locator('#nonmodal-close').focus(); await page.keyboard.press('Escape');
  await page.waitForFunction(() => !document.getElementById('nonmodal').open); assert.equal(await page.locator('#nonmodal-trigger').evaluate(node => document.activeElement===node),true,'modeless Escape returns its opener');
  await page.locator('#drawer-trigger').click(); await page.waitForFunction(() => document.getElementById('breadth-drawer').matches(':modal'));
  await page.locator('#drawer-child-trigger').click(); await page.waitForFunction(() => document.getElementById('drawer-child').matches(':modal'));
  await page.keyboard.press('Escape'); await page.waitForFunction(() => !document.getElementById('drawer-child').open); assert.equal(await page.locator('#breadth-drawer').evaluate(node => node.open),true);
  await page.locator('#drawer-close').focus(); await page.keyboard.press('Shift+Tab'); assert.equal(await page.locator('#drawer-unmount').evaluate(node => document.activeElement === node),true);
  await page.setViewportSize({width:375,height:812});
  const modal = await page.locator('#breadth-drawer').evaluate(node => {const style=getComputedStyle(node,'::backdrop');return {position:style.position,insets:[style.top,style.right,style.bottom,style.left],corners:[[1,1],[innerWidth-1,1],[1,innerHeight-1],[innerWidth-1,innerHeight-1]].every(([x,y]) => document.elementFromPoint(x,y)===node),scroll:scrollY};});
  assert.equal(modal.position,'fixed'); assert.deepEqual(modal.insets,['0px','0px','0px','0px']); assert.equal(modal.corners,true);
  await page.locator('#outside-overlay').evaluate(node => node.focus());
  assert.equal(await page.locator('#breadth-drawer').evaluate(node => node.contains(document.activeElement)),true,'modal drawer blocks background focus');
  await page.mouse.wheel(0,500); await frame(page); assert.equal(await page.evaluate(() => scrollY),modal.scroll,'drawer locks document scroll');
  const drawerGaps=await actionGaps(page,'Drawer actions'),drawerGeometry=[];
  for(const width of [375,1280]) for(const zoom of [1,2]){
    await page.setViewportSize({width,height:812}); await page.evaluate(zoom => {document.documentElement.style.zoom=zoom;},zoom); await frame(page);
    const bounds=await page.locator('#breadth-drawer').evaluate(node => {const r=node.getBoundingClientRect();return {left:r.left,top:r.top,right:r.right,bottom:r.bottom,width:innerWidth,height:innerHeight};});
    assert.equal(bounds.left>=-1 && bounds.top>=-1 && bounds.right<=bounds.width+1 && bounds.bottom<=bounds.height+1,true,`bounded drawer ${width}/${zoom}: ${JSON.stringify(bounds)}`);
    const gaps=await actionGaps(page,'Drawer actions',zoom);
    for(const id of ['drawer-close','drawer-child-trigger','drawer-unmount']){
      const button=await page.locator('#'+id).boundingBox();assert.equal(button!==null && button.x>=-1 && button.y>=-1 && button.x+button.width<=bounds.width+1 && button.y+button.height<=bounds.height+1,true,`reachable drawer action ${id} ${width}/${zoom}`);
    }
    drawerGeometry.push({width,zoom,bounds,gaps});
    await page.screenshot({path:path.join(output,'screenshots',width===375 && zoom===1?`${name}-m2-drawer-375.png`:`${name}-m2-drawer-${width}-zoom${zoom}.png`)});
  }
  await page.evaluate(() => {document.documentElement.style.zoom=1;}); await page.setViewportSize({width:375,height:812});
  await page.locator('#drawer-unmount').click(); await page.waitForFunction(() => !document.getElementById('breadth-drawer')); await page.locator('#restore-overlays').click();
  await page.locator('#tip-trigger').hover(); await opened(page,'tip-tip'); assert.equal(await page.locator('#tip-tip').getAttribute('role'),'tooltip');
  await page.locator('#tip-tip').hover(); await frame(page); assert.equal(await page.locator('#tip-tip').isVisible(),true,'hoverable tooltip text');
  await page.locator('#tip-trigger').focus(); await page.keyboard.press('Escape'); await closed(page,'tip-tip'); assert.equal(await page.locator('#tip-trigger').evaluate(node => document.activeElement === node),true);
  await page.locator('#outside-overlay').focus(); await page.mouse.move(1,1); await page.locator('#tip-trigger').hover(); await opened(page,'tip-tip'); await page.keyboard.press('Escape'); await closed(page,'tip-tip');
  assert.equal(await page.locator('#outside-overlay').evaluate(node => document.activeElement===node),true,'hover-only Escape preserves background focus');
  await page.locator('#show-toast').click(); assert.equal(await page.locator('#manual-toast').isVisible(),true); await page.locator('#manual-toast').getByRole('button',{name:'Dismiss',exact:true}).click(); assert.equal(await page.locator('#manual-toast').isVisible(),false);
  await page.locator('#show-timer-toast').click(); await page.locator('#timer-toast').hover();
  await page.waitForFunction(() => {const node=document.getElementById('timer-toast');return node.matches(':hover') && breadthLeases().timers===0;});
  // Several rendering turns while hovered leave the notice available; no arbitrary sleep.
  for(let turn=0;turn<4;turn++) await frame(page);
  assert.equal(await page.locator('#timer-toast').isVisible(),true);
  await page.mouse.move(1,1); await page.waitForFunction(() => document.getElementById('timer-toast').hidden);
  await page.locator('#outside-overlay').focus(); await page.mouse.move(1,1); await page.waitForFunction(() => breadthLeases().listeners === 0 && breadthLeases().timers === 0 && breadthLeases().observers === 0);
  return {nestedPopoverDrawer:true,nonmodal:true,tooltip:true,timerAndListenerCleanup:true,drawerGaps,drawerGeometry};
}

(async () => {
  const browser = await chromium.launch({headless:true});fs.mkdirSync(path.join(output,'screenshots'),{recursive:true});
  try {
    for (const mode of ['hydrate','csr']) for (const base of ['', '/demo/v1']) for (const earlySelection of mode==='hydrate'?['partial-page','page-only']:['none']) {
      const name=`${base?'mounted':'root'}-${mode}${earlySelection==='page-only'?'-page-only':''}`,scheme=base?'dark':'light';
      const context=await browser.newContext({viewport:{width:1280,height:900},colorScheme:scheme,hasTouch:!!base});const page=await context.newPage();
      page.setDefaultTimeout(10000);page.setDefaultNavigationTimeout(10000);await monitor(page);
      const errors=[],unexpected=[];let resourceCalls=0;let release;const boot=new Promise(resolve => {release=resolve;});
      page.on('pageerror',error => errors.push(error.message));page.on('console',message => {if(message.type()==='error'||/hydration.*(mismatch|error)/i.test(message.text()))errors.push(message.text());});
      await page.route('**/*',async route => {
        const req=route.request(),url=new URL(req.url());if(url.origin!=='https://gallery.invalid'){unexpected.push(url.href);return route.abort();}
        if(url.pathname.startsWith(`${base}/assets/`)){
          const relative=url.pathname.slice(`${base}/assets/`.length);if(relative==='start.js')await boot;
          const directory=path.join(output,'gallery-assets',mode),file=path.resolve(directory,relative);assert.equal(file.startsWith(directory+path.sep),true);
          return route.fulfill({path:file,contentType:file.endsWith('.wasm')?'application/wasm':file.endsWith('.png')?'image/png':'text/javascript'});
        }
        if(url.pathname===`${base}/resource`)resourceCalls++;
        const headers=await req.allHeaders();const response=request(base,{method:req.method(),path:url.pathname+url.search,cookie:headers.cookie||'',origin:headers.origin||null,body:req.postData()||'',csr:mode==='csr'});
        return route.fulfill({status:response.status,headers:response.headers,body:response.body});
      });
      try {
        await page.goto(`https://gallery.invalid${base}/en/gallery?theme=system`,{waitUntil:'commit'});
        let early=false;
        if(mode==='hydrate'){
          await page.locator('#breadth-core').waitFor();
          const color=await page.locator('.sp-root').evaluate(node => getComputedStyle(node).backgroundColor);assert.equal(color,scheme==='dark'?'rgb(32, 33, 36)':'rgb(255, 255, 255)','CSS system resolves before bootstrap');
          await page.locator('#native-details summary').click();await page.locator('#text-rows-1-value').fill('Before JavaScript 雪');await page.locator('#slug').fill('Before JavaScript');await page.locator('#combo').fill('Ga');
          await page.locator('#tag-rows-draft').fill('Early tag 雪'); await page.locator('#metadata-rows-4-key').fill('early'); await page.locator('#metadata-rows-4-value').fill('');
          await page.locator('label[for=rating-2]').click();
          await page.locator('#rating-2').focus(); await page.keyboard.press('ArrowRight'); await page.keyboard.press('ArrowLeft');
          assert.equal(await page.locator('.sp-rating-input .sp-icon').evaluateAll(nodes=>nodes.filter(node=>getComputedStyle(node).fill!=='none').length),2,'native checked state fills stars before JavaScript');
          await page.locator('#single-range-0').fill('4'); await page.locator('#pair-range-0').fill('3'); await page.locator('#pair-range-1').fill('7');
          await page.locator('#suggest').fill('Early query'); await page.locator('#data-table-filter').fill('Person 02'); if(earlySelection==='partial-page') {
            await page.locator('#data-table-row-r0').check();
            assert.equal(await page.locator('#data-table-row-r0').evaluate(node => getComputedStyle(node.closest('tr')).boxShadow!=='none'),true,'selection is visible from native checked state before JavaScript');
          }
          assert.equal(await page.locator('.sp-data-table tbody input:checked').count(),earlySelection==='partial-page'?1:0,'keep distinct partial and empty native row-selection scenarios');
          await page.locator('#data-table-select-page').check();
          await page.evaluate(() => {window.beforeBreadth=document.getElementById('app').innerHTML;window.earlyRow=document.getElementById('text-rows-1-value');window.earlyBreadthNodes=[...document.querySelectorAll('#breadth-core input,#breadth-core select,#breadth-core details,#breadth-core [data-row-id]')].map(node => [node.id || node.getAttribute('data-row-id'),node]);window.earlyRow.focus();});early=true;
        }
        assert.equal(await page.locator('#nested-rows').count(),0,'raw nullable-boolean fixture is absent from the owner gallery');
        release();await page.waitForFunction(() => window.galleryReady === true);
        if(early){
          const initialDom=await page.evaluate(() => ({before:beforeBreadth,after:document.getElementById('app').innerHTML}));
          if(initialDom.before!==initialDom.after){
            fs.writeFileSync(path.join(output,`${name}-before-hydrate.html`),initialDom.before); fs.writeFileSync(path.join(output,`${name}-after-hydrate.html`),initialDom.after);
            let first=0;while(first<Math.min(initialDom.before.length,initialDom.after.length) && initialDom.before[first]===initialDom.after[first])first++;
            console.error('initial DOM difference',JSON.stringify({offset:first,before:initialDom.before.slice(first-100,first+300),after:initialDom.after.slice(first-100,first+300)}));
          }
          assert.equal(initialDom.before,initialDom.after,'identical adopted initial DOM with no exception');
          assert.equal(await page.evaluate(() => earlyBreadthNodes.every(([id,node]) => node.isConnected && document.getElementById('breadth-core').contains(node) && (!node.id || document.getElementById(id)===node))),true,'all initial native controls and rows retain identity');assert.equal(await page.locator('#text-rows-1-value').evaluate(node => earlyRow===node && document.activeElement===node),true);const state=await inspect(page);assert.equal(state.text[0],'Before JavaScript 雪');assert.equal(state.combo,'Ga');assert.equal(await page.locator('#native-details').evaluate(node => node.open),true);
          assert.deepEqual(state.tags,['rust','雪']); assert.equal(await page.locator('#tag-rows-draft').inputValue(),'Early tag 雪','the unsubmitted draft survives hydration separately'); assert.deepEqual(state.metadata,[['early','']]);
          assert.deepEqual(state.ranges,[4,3,7]); assert.equal(state.rating,2); assert.equal(state.suggestQuery,'Early query'); assert.equal(state.filter,'Person 02'); assert.deepEqual(new Set(state.selectedRows),new Set(Array.from({length:12},(_,i)=>'r'+i)));}
        assert.equal(resourceCalls,mode==='hydrate'?0:1,'initial resource acquisition count');assert.equal(await page.locator('.sp-root').getAttribute('data-theme'),'system');
        await page.emulateMedia({colorScheme:scheme==='dark'?'light':'dark'});assert.equal(await page.locator('.sp-root').evaluate(node => getComputedStyle(node).backgroundColor),scheme==='dark'?'rgb(255, 255, 255)':'rgb(32, 33, 36)','media-only changes do not mutate the seeded token');
        const widgets=await controls(page,mode),layers=await overlays(page,name),geometries=[];
        await page.locator('#update-table-row').click();
        await page.getByRole('table',{name:'Paged people',exact:true}).getByRole('button',{name:'Name',exact:true}).click();
        for(const width of [375,1280]) for(const zoom of [1,2]){
          await page.setViewportSize({width,height:900});await page.evaluate(zoom => {document.documentElement.style.zoom=zoom;},zoom);
          const geometry = await page.evaluate(() => ({width:innerWidth,scroll:document.documentElement.scrollWidth,nodes:[...document.querySelectorAll('body *')].map(node => {const r=node.getBoundingClientRect();return {tag:node.tagName,id:node.id,cls:node.className,left:r.left,right:r.right,width:r.width};}).filter(node => node.right>innerWidth+1 || node.left< -1)}));
          assert.equal(geometry.scroll<=geometry.width+1,true,`no page overflow ${width}/${zoom}: ${JSON.stringify(geometry)}`);
          if(width===375){
            const overflow=page.getByRole('region',{name:'Explicit native table slots',exact:true});
            assert.equal(await overflow.evaluate(node => node.scrollWidth>node.clientWidth),true,'wide table stays inside its labelled scroll area');
            await overflow.focus(); await page.keyboard.press('ArrowRight'); await page.waitForFunction(() => [...document.querySelectorAll('[role=region]')].some(node => node.getAttribute('aria-label')==='Explicit native table slots' && node.scrollLeft>0));
            await overflow.evaluate(node => {node.scrollLeft=0;});
          }
          const tableGeometry=await m3.tableLayout(page,output,name,width,zoom);
          await page.locator('#rating-3').focus(); await page.keyboard.press('Space'); await page.keyboard.press('ArrowRight');
          await page.locator('.sp-rating-input').screenshot({path:path.join(output,'screenshots',`${name}-rating-${width}-zoom${zoom}.png`)});
          geometries.push({table:tableGeometry});
          await page.screenshot({path:path.join(output,'screenshots',`${name}-m3-${width}-zoom${zoom}.png`),fullPage:true});
          if(zoom===1) for(const id of ['text-rows','metadata-rows','tag-rows']) {
            await page.locator('#'+id).screenshot({path:path.join(output,'screenshots',`${name}-${id}-${width}.png`)});
          }
          await page.locator('#popup-trigger').click();await opened(page,'breadth-popup');
          const popup = await page.locator('#breadth-popup').evaluate(node => {const r=node.getBoundingClientRect();return {left:r.left,top:r.top,right:r.right,bottom:r.bottom,width:innerWidth,height:innerHeight};});
          assert.equal(popup.left>=-1 && popup.top>=-1 && popup.right<=popup.width+1 && popup.bottom<=popup.height+1,true,`bounded popup ${width}/${zoom}: ${JSON.stringify(popup)}`);
          const gaps=await actionGaps(page,'Popover actions',zoom);
          geometries.push({width,zoom,popup,gaps});
          if(width===375 && zoom===2) assert.equal(gaps.pairs.every(pair => pair.axis==='y'),true,'stacked popup actions are measured');
          const close=await page.locator('#popup-close').boundingBox(); assert.equal(close!==null && close.y>=0 && close.y+close.height<=popup.height+1,true,'close remains fully reachable at narrow zoom');
          await page.screenshot({path:path.join(output,'screenshots',`${name}-m2-popup-${width}-zoom${zoom}.png`)});
          await page.locator('#popup-close').click();await closed(page,'breadth-popup');
        }
        await page.evaluate(() => {document.documentElement.style.zoom=1;});
        await page.locator('#suggest').fill('slow');await page.locator('#show-timer-toast').click();await page.evaluate(() => window.disposeGallery());
        await page.waitForFunction(() => document.getElementById('app').innerHTML==='' && breadthLeases().listeners===0 && breadthLeases().timers===0 && breadthLeases().observers===0);
        assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);receipts.push({name,scheme,earlySelection,systemBeforeJs:early,resourceCalls,earlyDraftAndFocus:early,widgets,layers,geometries,leasesReleased:true,errors,unexpected});console.log('PASS breadth '+name);
      } finally {release();await context.close();}
    }
    fs.writeFileSync(path.join(output,'breadth-browser.json'),JSON.stringify({listenerStarted:false,chromium:browser.version(),receipts},null,2)+'\n');
  } finally {await browser.close();}
})().catch(error => {console.error(error);process.exitCode=1;});
