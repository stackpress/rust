/* Actual Chromium text/clipboard editing; composition dispatch is explicitly synthetic. */
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const {chromium}=require(process.env.UI_FORMAT_PLAYWRIGHT || process.env.GALLERY_PLAYWRIGHT);
const out=process.env.UI_FORMAT_OUTPUT_DIR || '/private/tmp/s8-format';
const native=JSON.parse(fs.readFileSync(path.join(out,'native.json'),'utf8'));
const modifier=process.platform==='darwin'?'Meta':'Control';
const css='*{box-sizing:border-box}body{margin:0;font:16px/1.5 system-ui;background:#fff;color:#222}main{max-width:60rem;margin:auto;padding:1rem;overflow-wrap:anywhere}h1{font-size:1.5rem}input,select,button{font:inherit;max-width:100%}input:not([type=hidden]),select{display:block;width:100%;min-width:0;padding:.5rem;background:#fff;color:#222;border:1px solid #666}input:focus-visible,button:focus-visible{outline:3px solid #555;outline-offset:2px}.sp-edit{margin-block:1rem}.sp-edit p{font-size:.85rem;margin:.4rem 0}.sp-edit [role=alert]{font-weight:700}.edit-actions{display:flex;flex-wrap:wrap;gap:.75rem}html:has(dialog:modal),body:has(dialog:modal){overflow:hidden}dialog{max-width:calc(100% - 2rem)}button{padding:.5rem;border:1px solid #666;border-radius:.2rem;background:#eee;color:#222}';
/** Bound evaluations too: Playwright's action timeout does not bound page.evaluate. */
function deadline(promise){let timer;return Promise.race([promise,new Promise((_,reject)=>{timer=setTimeout(()=>reject(new Error('10-second evaluation deadline')),10000);})]).finally(()=>clearTimeout(timer));}
/** Observe real form storage rather than infer a value from display text. */
async function values(page){return page.$eval('#editing-form',form=>Object.fromEntries(new FormData(form)));}
/** Read native UTF-16 selection, including its backward anchor/focus direction. */
async function selected(input){return input.evaluate(node=>[node.selectionStart,node.selectionEnd,node.selectionDirection]);}
/** Set the OS Chromium clipboard at this fixture origin, then issue a trusted paste key. */
async function paste(page,context,input,text){
  await deadline(context.grantPermissions(['clipboard-read','clipboard-write'],{origin:'https://format.invalid'}));
  await page.evaluate(text=>navigator.clipboard.writeText(text),text);
  await input.press(modifier+'+V');
}
/** Deliberately synthetic IME ordering; these events do not claim a real OS IME. */
async function composition(page,final,blurFirst){
  await page.evaluate(({final,blurFirst})=>{
    const input=document.getElementById('edit-number');input.focus();
    input.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true,data:''}));
    input.value='一';input.setSelectionRange(1,1);
    input.dispatchEvent(new CompositionEvent('compositionupdate',{bubbles:true,data:'一'}));
    input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertCompositionText',data:'一',isComposing:true}));
    if(input.value!=='一')throw new Error('composition text rewritten');
    if(blurFirst)document.getElementById('edit-outside').focus();
    input.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:final}));
    input.value=final;input.setSelectionRange(final.length,final.length);
    input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:final}));
    input.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertText',data:final}));
  },{final,blurFirst});
}
(async()=>{
 let browser;const results=[];fs.mkdirSync(path.join(out,'screenshots'),{recursive:true});
 try{
  browser=await deadline(chromium.launch({headless:true,timeout:10000}));
  for(const locale of ['en','fr'])for(const mode of ['hydrate','csr'])for(const mount of ['','/demo/v1']){
   const context=await deadline(browser.newContext({viewport:{width:375,height:900},locale:locale==='fr'?'en-US':'fr-FR',serviceWorkers:'block'}));
   try{
    const page=await context.newPage(),errors=[],unexpected=[];page.setDefaultTimeout(10000);page.setDefaultNavigationTimeout(10000);
    const evaluate=page.evaluate.bind(page);page.evaluate=(...args)=>deadline(evaluate(...args));
    page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(['error','warning'].includes(message.type()))errors.push(message.text());});
    await page.addInitScript(()=>{
      const set=window.setTimeout.bind(window),clear=window.clearTimeout.bind(window),active=new Set();
      window.setTimeout=(callback,delay,...args)=>{const id=set(()=>{active.delete(id);callback(...args);},delay);active.add(id);return id;};
      window.clearTimeout=id=>{active.delete(id);return clear(id);};window.editTimers=()=>active.size;
      window.trustedPastes=[];document.addEventListener('paste',event=>trustedPastes.push(event.isTrusted));
    });
    await context.route('**/*',route=>{
      const url=new URL(route.request().url()),prefix=mount+'/';
      if(url.origin!=='https://format.invalid'||!url.pathname.startsWith(prefix)){unexpected.push(url.href);return route.abort();}
      const file=url.pathname.slice(prefix.length);
      if(file==='')return route.fulfill({contentType:'text/html',body:`<!doctype html><html lang="${locale}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Editing proof</title><link rel="stylesheet" href="${prefix}edit.css"></head><body style="zoom:2"><div id="app">${mode==='hydrate'?native['edit_'+locale]:''}</div></body></html>`});
      if(file==='edit.css')return route.fulfill({contentType:'text/css',body:css});
      if(['proof.js','proof_bg.wasm'].includes(file))return route.fulfill({contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript',path:path.join(out,'assets',mode+'-both',file)});
      unexpected.push(url.href);return route.abort();
    });
    await page.goto('https://format.invalid'+mount+'/');const number=page.locator('#edit-number'),mark=locale==='fr'?',':'.';let before;
    if(mode==='hydrate'){
      await number.fill('12'+mark);await number.evaluate(node=>{node.focus();node.setSelectionRange(1,2,'backward');window.originalNumber=node;});
      before=await page.$eval('#app',node=>node.innerHTML);
    }
    await page.evaluate(async({url,locale})=>{const module=await import(url);await module.default();window.disposeEditor=module.start_editor(locale);},{url:'https://format.invalid'+mount+'/proof.js',locale});
    await number.waitFor();
    if(mode==='hydrate'){
      assert.equal(await page.$eval('#app',node=>node.innerHTML),before,'exact SSR/adopted hydrate DOM');assert.equal(await number.evaluate(node=>node===originalNumber&&document.activeElement===node),true);assert.deepEqual(await selected(number),[1,2,'backward']);assert.equal(await number.inputValue(),'12'+mark);
    }
    assert.equal(await page.locator('#edit-commits').textContent(),'0');assert.equal((await values(page)).decimal,'1234.50');
    // Partial failure preserves the prior successful-field value and the actual draft.
    await number.fill('-');await number.press('Enter');assert.equal(await number.inputValue(),'-');assert.equal((await values(page)).decimal,'1234.50');assert.equal(await page.locator('#edit-number-error').textContent(),'Complete the field before committing');
    await number.fill('1234'+mark+'125');await number.evaluate(node=>node.setSelectionRange(1,4,'backward'));await number.press('Enter');
    assert.equal((await values(page)).decimal,'1234.12');assert.deepEqual(await selected(number),[2,5,'backward']);
    // Normalization does not erase native user-edit undo/redo transactions.
    await number.press(modifier+'+Z');assert.equal(await number.inputValue(),'1234'+mark+'125','undo normalization restores the native draft');
    await number.press(modifier+'+Z');assert.equal(await number.inputValue(),'-','prior user edit remains undoable');
    await number.press(modifier+'+Shift+Z');assert.equal(await number.inputValue(),'1234'+mark+'125');
    await number.press(modifier+'+Shift+Z');assert.equal(await number.inputValue(),locale==='fr'?'1\u202f234,12':'1,234.12','redo restores native normalization');
    // Native middle insertion/deletion regroups at commit and preserves logical digits.
    await number.evaluate(node=>node.setSelectionRange(3,3));await number.press('9');await number.press('Enter');assert.equal((await values(page)).decimal,'12934.12');
    await number.press('Backspace');await number.press('Enter');assert.equal((await values(page)).decimal,'1234.12');
    // Clipboard permission is explicitly scoped to this .invalid fixture origin.
    await number.press(modifier+'+A');await paste(page,context,number,locale==='fr'?'1\u00a0234,50':'1,234.50');await number.press('Enter');assert.equal((await values(page)).decimal,'1234.50');
    await number.press(modifier+'+A');await paste(page,context,number,'9'+mark+'25');assert.equal(await number.inputValue(),'9'+mark+'25');
    await number.press(modifier+'+Z');assert.equal(await number.inputValue(),locale==='fr'?'1\u202f234,50':'1,234.50','native undo restores the previous edit');
    await number.press(modifier+'+Shift+Z');assert.equal(await number.inputValue(),'9'+mark+'25','native redo restores paste');
    const raw=await number.inputValue();await number.press(modifier+'+A');const range=await selected(number);
    for(const bad of ['1.2,3','9\n2','9'.repeat(25),'雪'.repeat(130),'9'.repeat(400)]){await paste(page,context,number,bad);assert.equal(await number.inputValue(),raw);assert.deepEqual(await selected(number),range);}
    await number.press('Enter');assert.equal((await values(page)).decimal,'9.25');
    // Locale switching while dirty never reinterprets the outstanding decimal mark.
    await number.fill('7'+mark+'5');await page.locator(locale==='en'?'#edit-fr':'#edit-en').click();assert.equal(await number.inputValue(),'7'+mark+'5');await number.press('Enter');assert.equal((await values(page)).decimal,'7.50');assert.equal(await number.inputValue(),locale==='en'?'7,50':'7.50');
    await page.locator(locale==='en'?'#edit-en':'#edit-fr').click();
    await number.fill('1000000');await number.press('Enter');assert.equal((await values(page)).decimal,'7.50');assert.equal(await page.locator('#edit-number-error').textContent(),'Value is outside the field bounds');await number.press('Escape');
    await number.fill('');await number.press('Enter');assert.equal((await values(page)).decimal,undefined,'explicit nullable clear');await number.fill('-0'+mark+'004');await number.press('Enter');assert.equal((await values(page)).decimal,'-0.00');
    // Calendar, precision and explicit offset failures retain text plus prior typed values.
    const date=page.locator('#edit-date');await date.fill('2023-02-29');await date.press('Enter');assert.equal((await values(page)).date,'2024-02-29');assert.equal(await date.inputValue(),'2023-02-29');await date.fill('2024-02-');await date.press('Enter');assert.equal(await date.inputValue(),'2024-02-');await date.fill('2024-03-01');await date.press('Enter');assert.equal((await values(page)).date,'2024-03-01');
    const time=page.locator('#edit-time');await time.fill('23:59:59.1234');await time.press('Enter');assert.equal(await page.locator('#edit-time-error').textContent(),'Too many fractional digits');await time.fill('23:59:59.12');await time.press('Enter');assert.equal((await values(page)).time,'23:59:59.120');
    const instant=page.locator('#edit-instant');await instant.fill('2026-11-01T01:30');await instant.press('Enter');assert.equal((await values(page)).instant,'2026-11-01T01:30:00-04:00');await instant.fill('2026-11-01T01:30-05:00');await instant.press('Enter');assert.equal((await values(page)).instant,'2026-11-01T01:30:00-05:00');
    // Explicit-region national numbers and pasted international prefixes commit only E.164.
    const phone=page.locator('#edit-phone');await phone.fill('06 12 34 56 78');await phone.press('Enter');assert.equal((await values(page)).phone,'+33612345678');await phone.fill('+33');await phone.press('Enter');assert.equal((await values(page)).phone,'+33612345678');await page.locator('#edit-region').selectOption('US');assert.equal(await page.locator('#edit-region').inputValue(),'FR','dirty region change is rejected visibly');await phone.focus();await phone.press('Escape');await page.locator('#edit-region').selectOption('US');assert.equal(await page.locator('#edit-region').inputValue(),'US');
    await phone.fill('4155552671');await phone.press('Enter');assert.equal((await values(page)).phone,'+14155552671');await phone.press(modifier+'+A');await paste(page,context,phone,'0044 20 7946 0018');await phone.press('Enter');assert.equal((await values(page)).phone,'+442079460018');await phone.press(modifier+'+A');const previous=await phone.inputValue();await paste(page,context,phone,'+442079460018x2');assert.equal(await phone.inputValue(),previous);await phone.press('Escape');
    // A clean composition cancellation must not fabricate a dirty region context.
    await phone.evaluate(node=>{node.focus();node.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));node.value='雪';node.setSelectionRange(1,1);node.dispatchEvent(new InputEvent('input',{bubbles:true,isComposing:true,data:'雪'}));});await phone.press('Escape');await page.locator('#edit-region').selectOption('GB');assert.equal(await page.locator('#edit-region').inputValue(),'GB','clean cancellation permits explicit region switch');
    // Blur commits rounding without resetting native text or undo history.
    await number.fill('5'+mark+'555');await page.locator('#edit-outside').focus();assert.equal((await values(page)).decimal,'5.56');assert.equal(await number.inputValue(),'5'+mark+'555');
    await number.focus();await number.press(modifier+'+Z');assert.equal(await number.inputValue(),'-0'+mark+'00');await number.press(modifier+'+Shift+Z');assert.equal(await number.inputValue(),'5'+mark+'555');await number.press('Enter');
    // Actual keyboard range selection and replacement, independent of scripted caret seeds.
    await number.press(process.platform==='darwin'?'Meta+ArrowLeft':'Home');await number.press('ArrowRight');await number.press('Shift+ArrowRight');await number.press('Shift+ArrowRight');assert.deepEqual(await selected(number),[1,3,'forward']);await number.press('2');await number.press('Enter');assert.equal((await values(page)).decimal,'526.00');
    // Synthetic composition: untouched intermediate text, blur-before-final and duplicate input.
    const count=Number(await page.locator('#edit-commits').textContent());await composition(page,'6'+mark+'25',true);await page.waitForFunction(()=>document.querySelector('input[name=decimal]').value==='6.25');assert.equal(Number(await page.locator('#edit-commits').textContent()),count+1);await page.locator('#edit-outside').focus();assert.equal(Number(await page.locator('#edit-commits').textContent()),count+1);
    await composition(page,'8'+mark+'75',false);await page.waitForFunction(()=>editTimers()===0);assert.equal((await values(page)).decimal,'6.25','composition end alone does not commit');await number.press('Enter');assert.equal((await values(page)).decimal,'8.75');
    // Synthetic composition cancellation restores its starting draft and exact selection.
    await number.fill('9'+mark+'5');await number.evaluate(node=>{node.setSelectionRange(1,2,'backward');node.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));node.value='雪';node.setSelectionRange(1,1);node.dispatchEvent(new InputEvent('input',{bubbles:true,isComposing:true,data:'雪'}));});
    await number.press('Escape');assert.equal(await number.inputValue(),'9'+mark+'5');assert.deepEqual(await selected(number),[1,2,'backward']);assert.equal((await values(page)).decimal,'8.75');await number.press('Escape');
    // Dirty Escape cancels one edit; clean Escape remains available to the native dialog.
    await page.locator('#edit-modal-open').click();await page.waitForFunction(()=>document.getElementById('edit-modal').open);await page.locator('#edit-modal-number').press('Escape');await page.waitForFunction(()=>!document.getElementById('edit-modal').open);
    await page.locator('#edit-modal-open').click();await page.waitForFunction(()=>document.getElementById('edit-modal').open);await page.locator('#edit-modal-number').fill('2'+mark);await page.locator('#edit-modal-number').press('Escape');assert.equal(await page.locator('#edit-modal').evaluate(node=>node.open),true);assert.equal(await page.locator('#edit-modal-number').inputValue(),locale==='fr'?'1,00':'1.00');await page.locator('#edit-modal-number').press('Escape');await page.waitForFunction(()=>!document.getElementById('edit-modal').open);assert.equal(await page.locator('#edit-modal-open').evaluate(node=>document.activeElement===node),true);
    const slug=`edit-${locale}-${mode}-${mount?'mounted':'root'}`;
    for(const [width,zoom] of [[375,2],[1280,1]]){
      await page.setViewportSize({width,height:900});await page.evaluate(zoom=>document.body.style.zoom=String(zoom),zoom);await number.scrollIntoViewIfNeeded();assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),true);
      // Wrapped and same-row actions both retain the declared gap after CSS zoom.
      assert.equal(await page.evaluate(()=>[...document.querySelectorAll('.edit-actions')].every(group=>{
        const gap=parseFloat(getComputedStyle(group).gap)*Number(getComputedStyle(document.body).zoom||1);
        const buttons=[...group.querySelectorAll('button')].map(node=>node.getBoundingClientRect());
        return buttons.slice(1).every((button,index)=>{
          const previous=buttons[index];
          return (Math.abs(button.top-previous.top)<1?button.left-previous.right:button.top-previous.bottom)>=gap-1;
        });
      })),true,'horizontal and wrapped action gaps');
      await page.screenshot({path:path.join(out,'screenshots',`${slug}-${width}-zoom${zoom}.png`)});
    }
    assert.equal(await page.evaluate(()=>trustedPastes.length>=7&&trustedPastes.every(Boolean)),true,'paste events came from actual clipboard keyboard commands');
    await page.evaluate(()=>{
      const input=document.getElementById('edit-number');window.detachedEditor=input;
      input.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));input.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:''}));window.disposeEditor();
      input.dispatchEvent(new InputEvent('input',{bubbles:true,data:'late'}));
    });
    await page.waitForFunction(()=>document.getElementById('app').innerHTML===''&&editTimers()===0);assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
    results.push({locale,mode,mount,viewportInteractions:'375px/200%',trustedClipboard:true,syntheticComposition:true,realOsIme:false,initialDomRetained:mode==='hydrate',timersReleased:true,errors,unexpected});console.log('PASS '+slug);
   }finally{await deadline(context.close());}
  }
  fs.writeFileSync(path.join(out,'edit-browser.json'),JSON.stringify({listenerStarted:false,chromium:browser.version(),results},null,2)+'\n');
 }finally{if(browser)await deadline(browser.close());}
})().catch(error=>{console.error(error);process.exitCode=1;});
