// M3 browser assertions shared by the gallery and proof-only fixture.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const folder = path.resolve(__dirname,'../../crates/framework/framework-ui/third-party/lucide');

/** Compare real inline primitives with a pinned master, without normalizing geometry away. */
async function icon(svg, name, size) {
  const source=fs.readFileSync(path.join(folder,name+'.svg'),'utf8');
  const result=await svg.evaluate((node,source)=>{
    const shapes=root=>[...root.children].map(shape=>[shape.localName,[...shape.attributes].map(attr=>[attr.name,attr.value]).sort()]);
    const master=new DOMParser().parseFromString(source,'image/svg+xml').documentElement;
    return {same:JSON.stringify(shapes(node))===JSON.stringify(shapes(master)),size:node.getAttribute('width'),height:node.getAttribute('height'),hidden:node.getAttribute('aria-hidden'),focusable:node.getAttribute('focusable'),stroke:node.getAttribute('stroke')};
  },source);
  assert.deepEqual(result,{same:true,size:String(size),height:String(size),hidden:'true',focusable:'false',stroke:'currentColor'});
}

/** Prove bounded local scroll, keyboard reach, sticky heads and inline sort controls at each layout. */
async function tableLayout(page, output, name, width, zoom) {
  const region=page.getByRole('region',{name:'Paged people',exact:true});
  const table=page.getByRole('table',{name:'Paged people',exact:true});
  assert.equal(await table.locator('thead th').count(),13);
  assert.equal(await table.locator('thead').evaluate((node,zoom)=>node.getBoundingClientRect().height<=(matchMedia('(pointer:coarse)').matches?72:64)*zoom,zoom),true,'headers stay compact without breaking words');
  assert.equal(await table.locator('tbody tr').count(),12);
  const scroll=await region.evaluate(node=>({horizontal:node.scrollWidth>node.clientWidth,vertical:node.scrollHeight>node.clientHeight,tab:node.tabIndex}));
  assert.deepEqual(scroll,{horizontal:true,vertical:true,tab:0});
  await region.evaluate(node=>{node.scrollLeft=0;node.scrollTop=0;});
  await region.focus(); await page.keyboard.press('ArrowRight'); await page.keyboard.press('ArrowDown');
  await page.waitForFunction(()=>{const n=document.querySelector('.sp-data-table .sp-table-scroll');return n.scrollTop>0 && n.scrollLeft>0;});
  await region.evaluate(node=>{node.scrollTop=200;});
  assert.equal(await region.evaluate(node=>[...node.querySelectorAll('thead th')].every(head=>getComputedStyle(head).position==='sticky' && Math.abs(head.getBoundingClientRect().top-node.getBoundingClientRect().top)<=2*Number(document.documentElement.style.zoom||1))),true);
  const reference=table.getByRole('button',{name:'Reference',exact:true});
  await reference.focus();
  const visible=await reference.evaluate(node=>{
    const r=node.getBoundingClientRect(),p=node.closest('[role=region]').getBoundingClientRect(),style=getComputedStyle(node);
    return {inside:r.left>=p.left && r.right<=p.right, border:style.borderTopWidth,background:style.backgroundColor,padding:style.paddingTop,outline:style.outlineStyle};
  });
  assert.deepEqual(visible,{inside:true,border:'0px',background:'rgba(0, 0, 0, 0)',padding:'0px',outline:'solid'});
  assert.equal(await reference.evaluate((node,zoom)=>node.getBoundingClientRect().height+1>=(matchMedia('(pointer:coarse)').matches?44:32)*zoom,zoom),true,'sort controls keep the approved invisible hit area');
  await icon(reference.locator('svg'),'arrow-up-down',16);
  await reference.press('Enter'); assert.equal(await reference.locator('..').getAttribute('aria-sort'),'ascending');
  await icon(reference.locator('svg'),'arrow-up',16);
  await reference.press('Space'); assert.equal(await reference.locator('..').getAttribute('aria-sort'),'descending');
  await icon(reference.locator('svg'),'arrow-down',16);
  await reference.press('Enter'); assert.equal(await reference.locator('..').getAttribute('aria-sort'),'none');
  await reference.evaluate(node=>node.scrollIntoView({block:'nearest',inline:'nearest'}));
  await page.locator('.sp-data-table').screenshot({path:path.join(output,'screenshots',`${name}-table-right-${width}-zoom${zoom}.png`)});
  await region.evaluate(node=>{node.scrollLeft=0;node.scrollTop=0;});
  assert.equal(await page.locator('#data-table-select-page').getAttribute('type'),'checkbox');
  assert.equal(await page.getByLabel('Select page',{exact:true}).count(),1);
  const notes=table.locator('tbody tr').first().locator('td').nth(11);
  assert.equal(await notes.evaluate(node=>getComputedStyle(node).whiteSpace),'pre-line');
  assert.equal(await table.locator('tbody tr').first().locator('td').nth(3).evaluate(node=>getComputedStyle(node).whiteSpace),'nowrap');
  await page.locator('.sp-data-table').screenshot({path:path.join(output,'screenshots',`${name}-table-left-${width}-zoom${zoom}.png`)});
  return {width,zoom,...scroll,keyboardReach:true,sticky:true,sortGeometry:true};
}

/** Native radios retain full names and cumulative fill, including exact zero and clearing. */
async function rating(page, prefix, minimum=0) {
  const field=page.locator('fieldset').filter({has:page.locator('#'+prefix+'-'+minimum)});
  assert.equal(await field.getByRole('radio').count(),6-minimum);
  for(const value of [minimum,5]) assert.equal(await field.getByRole('radio',{name:`${value} of 5`,exact:true}).count(),1);
  assert.equal(await field.locator('input[value="0"]').count(),minimum===0?1:0);
  const option=value=>page.locator(`label[for="${prefix}-${value}"]`);
  await option(5).click();
  const fills=()=>field.locator('.sp-icon').evaluateAll(nodes=>nodes.map(node=>getComputedStyle(node).fill));
  assert.equal((await fills()).every(fill=>fill!=='none'),true);
  await icon(option(5).locator('svg'),'star',24);
  await page.locator('#'+prefix+'-5').focus(); await page.keyboard.press('ArrowLeft');
  assert.equal(await page.locator('#'+prefix+'-4').isChecked(),true);
  assert.equal(await option(4).evaluate(node=>getComputedStyle(node).outlineStyle),'solid');
  assert.equal((await fills()).at(-1),'none');
  await field.getByRole('button',{name:'Clear rating',exact:true}).click();
  assert.equal(await field.getByRole('radio',{checked:true}).count(),0);
  assert.equal(await field.locator('output').textContent(),'Unrated');
  await page.keyboard.press('Shift+Tab');
  assert.equal(await field.evaluate(node=>node.contains(document.activeElement)&&document.activeElement.type==='radio'),true,'Tab reaches the native radio group');
  await page.keyboard.press('Tab');
  assert.equal(await field.getByRole('button',{name:'Clear rating',exact:true}).evaluate(node=>node===document.activeElement),true,'Tab leaves the group for Clear rating');
  await page.locator('#'+prefix+'-'+minimum).focus(); await page.keyboard.press('Space');
  assert.equal(await page.locator('#'+prefix+'-'+minimum).isChecked(),true);
  if(minimum===0) {assert.equal(await option(0).locator('svg').count(),0);assert.equal((await fills()).every(fill=>fill==='none'),true);}
}
module.exports={icon,tableLayout,rating};
