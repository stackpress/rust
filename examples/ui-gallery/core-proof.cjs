// Actual native-control, semantic presentation and viewport-modal behavior.
const assert = require('node:assert/strict');
const path = require('node:path');
const textareaPayload = '\n</textarea><script>globalThis.pwned=1</script>&"\'\u2028';
const ids = ['core-text','core-notes','core-password','core-check','core-switch','core-radio-zero','core-radio-fr','core-select','core-multiple'];
const state = async page => {
  await page.locator('#core-inspect').evaluate(node => node.click());
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(resolve)));
  return page.locator('#core-state').evaluate(node => JSON.parse(node.textContent));
};

exports.early = async page => {
  assert.equal(await page.locator('#core-notes').inputValue(),textareaPayload,'hostile initial SSR textarea parses exactly before JavaScript');
  assert.equal(await page.evaluate(() => globalThis.pwned),undefined);
  await page.locator('#core-text').fill('Before JS 雪😀');
  await page.locator('#core-notes').fill('Before JS\n日本');
  await page.locator('#core-password').fill('Transient before JS');
  await page.locator('#core-check').check();
  await page.locator('#core-switch').uncheck();
  await page.locator('#core-radio-fr').check();
  await page.locator('#core-select').selectOption('0');
  await page.locator('#core-multiple').selectOption(['','fr']);
  await page.evaluate(ids => { window.earlyCore = ids.map(id => document.getElementById(id)); }, ids);
};

exports.roundtrip = async page => {
  assert.equal(await page.locator('#core-notes').inputValue(),textareaPayload,'hostile initial textarea round-trips after hydrate/CSR');
  assert.equal(await page.evaluate(() => globalThis.pwned),undefined);
  assert.equal(await page.locator('#ordinary-core script').count(),0);
};

exports.controls = async (page, mode) => {
  const textareaParsing = await page.evaluate(() => {
    const fragment = document.createElement('textarea'); fragment.innerHTML='\nOne';
    const parsed = new DOMParser().parseFromString('<textarea>\nOne</textarea>','text/html').querySelector('textarea');
    return {fragment:fragment.value,document:parsed.value};
  });
  if (mode === 'hydrate') {
    assert.equal(await page.evaluate(ids => ids.every((id,index) => document.getElementById(id) === window.earlyCore[index]),ids), true, 'all ordinary controls retain their actual nodes');
    const initial = await state(page);
    assert.equal(initial.text,'Before JS 雪😀'); assert.equal(initial.notes,'Before JS\n日本');
    assert.equal(initial.passwordLength,19); assert.equal(initial.checked,true); assert.equal(initial.enabled,false);
    assert.equal(initial.radio,'fr'); assert.equal(initial.single,'0'); assert.deepEqual(initial.multiple,['','fr']);
  } else {
    const initial = await state(page);
    await exports.roundtrip(page);
    assert.equal(initial.notes,textareaPayload);
    assert.equal(initial.text,'Seed 雪'); assert.equal(initial.single,''); assert.deepEqual(initial.multiple,['0','fr']);
    assert.equal(initial.checked,false); assert.equal(initial.enabled,true); assert.equal(initial.passwordLength,0);
  }
  assert.equal(await page.getByRole('switch',{name:'Enable reminders'}).count(),1);
  assert.equal(await page.getByRole('group',{name:'Delivery mode'}).count(),1);
  assert.equal(await page.getByRole('progressbar',{name:'Progress',exact:true}).count(),1);
  assert.equal(await page.getByRole('table',{name:'Read-only values',exact:true}).count(),1);
  assert.equal(await page.getByRole('region',{name:'Presentation example',exact:true}).count(),1);
  for (const id of ids) {
    assert.equal(await page.locator(`label[for="${id}"]`).count(),1, `${id} native label`);
    const described = await page.locator('#'+id).getAttribute('aria-describedby');
    if (described) for (const target of described.split(' ')) assert.equal(await page.locator('#'+target).count(),1, 'described-by resolves');
  }
  await page.locator('#core-text').fill(''); await page.locator('#core-validate').click();
  await page.waitForFunction(() => document.getElementById('core-text').getAttribute('aria-invalid') === 'true');
  assert.equal(await page.locator('#core-text-error').textContent(),'Enter text');
  // Composition is unfiltered: partial text and the final Unicode edit both publish verbatim.
  await page.locator('#core-text').evaluate(node => {
    node.focus(); node.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));
    node.value='に'; node.setSelectionRange(1,1); node.dispatchEvent(new InputEvent('input',{bubbles:true,data:'に',inputType:'insertCompositionText',isComposing:true}));
  });
  assert.equal((await state(page)).text,'に');
  await page.locator('#core-text').evaluate(node => {
    node.value='日本😀'; node.setSelectionRange(4,4); node.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'日本😀'}));
    node.dispatchEvent(new InputEvent('input',{bubbles:true,data:'日本😀',inputType:'insertText'}));
  });
  assert.equal((await state(page)).text,'日本😀');
  assert.equal(await page.locator('#core-text').evaluate(node => node.selectionStart),4,'composition caret retained');
  await page.locator('#core-notes').fill('Line'); await page.locator('#core-notes').press('Enter'); await page.locator('#core-notes').pressSequentially('two');
  assert.equal((await state(page)).notes,'Line\ntwo');
  await page.locator('#core-password').fill('Transient 雪');
  await page.locator('#core-password').evaluate(node => { window.passwordNode=node; });
  const reveal = page.locator('.sp-password button'); await reveal.click();
  assert.equal(await reveal.getAttribute('aria-pressed'),'true'); assert.equal(await reveal.textContent(),'Hide password');
  assert.equal(await page.locator('#core-password').getAttribute('type'),'text');
  assert.equal(await page.evaluate(() => window.passwordNode === document.getElementById('core-password') && document.activeElement === window.passwordNode),true);
  assert.equal(await page.locator('#core-password').inputValue(),'Transient 雪'); await reveal.click();
  await page.locator('#core-check').uncheck(); await page.locator('#core-check').focus(); await page.keyboard.press('Space');
  assert.equal((await state(page)).checked,true);
  await page.locator('#core-switch').uncheck(); await page.locator('#core-switch').focus(); await page.keyboard.press('Space');
  assert.equal((await state(page)).enabled,true); assert.equal(await page.locator('#core-switch').getAttribute('aria-checked'),'true');
  await page.locator('#core-radio-zero').check(); await page.locator('#core-radio-zero').focus(); await page.keyboard.press('ArrowDown');
  assert.equal((await state(page)).radio,'fr');
  await page.locator('#core-select').selectOption(''); await page.locator('#core-multiple').selectOption(['','0']);
  assert.equal((await state(page)).single,''); assert.deepEqual((await state(page)).multiple,['','0']);
  assert.equal(await page.locator('#core-select option[value=blocked]').isDisabled(),true);
  let data = await page.locator('#core-form').evaluate(node => [...new FormData(node).entries()]);
  assert.deepEqual(data.filter(([key]) => key === 'choices').map(([,value]) => value),['','0']);
  assert.equal(data.some(([key,value]) => key === 'choice' && value === ''),true,'selected empty value is successful');
  assert.equal(data.some(([key,value]) => key === 'readonly' && value === 'Kept 0'),true);
  assert.equal(await page.locator('#core-readonly').evaluate(node => node.readOnly),true);
  assert.equal(await page.locator('#core-check').evaluate(node => node.required),true);
  await page.locator('#core-disabled').click();
  for (const id of ids) assert.equal(await page.locator('#'+id).isDisabled(),true);
  const before = await state(page); await page.locator('#core-check').evaluate(node => node.click());
  assert.deepEqual(await state(page),before,'native disabled click has no event/state mutation');
  data = await page.locator('#core-form').evaluate(node => [...new FormData(node).entries()]);
  assert.deepEqual(data,[['readonly','Kept 0']],'disabled fields are omitted');
  await page.locator('#core-disabled').click(); await page.locator('#core-reset').click();
  await page.waitForFunction(() => document.getElementById('core-text').value === 'Programmatic 雪');
  assert.equal(await page.locator('#core-notes').inputValue(),'Updated\n雪');
  assert.equal(await page.locator('#core-check').isChecked(),false); assert.equal(await page.locator('#core-switch').isChecked(),false);
  assert.equal(await page.locator('#core-radio-fr').isChecked(),false);
  data = await page.locator('#core-form').evaluate(node => [...new FormData(node).entries()]);
  assert.equal(data.some(([key]) => ['updates','enabled','mode','choice','choices'].includes(key)),false,'None/empty multiple are unselected');
  assert.equal(await page.locator('#core-text').getAttribute('aria-invalid'),'false');
  assert.equal(await page.locator('#core-email a').getAttribute('href'),'mailto:demo%2Btag@example.invalid');
  assert.equal(await page.locator('#core-phone a').getAttribute('href'),'tel:+639171234567');
  assert.equal(await page.locator('#core-overflow [aria-hidden=true]').textContent(),'雪😀…');
  assert.equal(await page.locator('#core-overflow .sp-sr-only').textContent(),'雪😀abc');
  assert.equal(await page.locator('#core-rating').textContent(),'★★★☆☆2.5 / 5');
  assert.equal(await page.locator('#core-color').textContent(),'#777');
  assert.equal(await page.locator('.sp-image').getAttribute('alt'),'One pixel demonstration asset');
  await page.locator('.sp-image').scrollIntoViewIfNeeded();
  await page.waitForFunction(() => document.querySelector('.sp-image').complete && document.querySelector('.sp-image').naturalWidth === 1);
  await page.locator('#core-progress-unknown').click();
  await page.waitForFunction(() => !document.getElementById('core-progress').hasAttribute('value'));
  await page.locator('#core-progress-known').click();
  assert.equal(await page.locator('#core-progress').getAttribute('value'),'10');
  assert.equal(await page.locator('#ordinary-core script,#ordinary-core iframe').count(),0);
  await page.locator('#draft').scrollIntoViewIfNeeded(); await page.locator('#draft').focus();
  return {textareaParsing,earlyStateAdopted:mode === 'hydrate',nativeFormData:true,compositionAndCaret:true,controlledUpdates:true};
};

exports.spacing = async page => {
  for (const selector of ['nav.sp-actions','[aria-label="Gallery actions"]']) {
    const boxes = await page.locator(selector+' button').evaluateAll(nodes => nodes.map(node => { const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom}; }));
    for (let index=1; index<boxes.length; index++) {
      const a=boxes[index-1],b=boxes[index];
      assert.equal(Math.abs(a.top-b.top)<1 ? b.left-a.right >= 8 : b.top-a.bottom >= 8,true,'action gap >=8px');
    }
  }
};

exports.modal = async (page, output, name) => {
  await page.setViewportSize({width:375,height:812});
  const native = await page.locator('#editor').evaluate(dialog => {
    const style=getComputedStyle(dialog,'::backdrop');
    return {position:style.position,top:style.top,right:style.right,bottom:style.bottom,left:style.left,viewport:innerHeight,document:document.documentElement.scrollHeight,overflow:getComputedStyle(document.documentElement).overflow,
      corners:[[1,1],[innerWidth-1,1],[1,innerHeight-1],[innerWidth-1,innerHeight-1]].map(([x,y]) => document.elementFromPoint(x,y)?.id)};
  });
  assert.equal(native.position,'fixed'); for (const side of ['top','right','bottom','left']) assert.equal(native[side],'0px');
  assert.deepEqual(native.corners,['editor','editor','editor','editor'],'modal top layer covers every viewport corner');
  assert.equal(native.overflow,'hidden');
  const before = await page.evaluate(() => scrollY);
  await page.mouse.move(1,1); await page.mouse.wheel(0,800);
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  assert.equal(await page.evaluate(() => scrollY),before,'background wheel cannot scroll');
  await page.locator('#english').evaluate(node => node.focus());
  assert.equal(await page.evaluate(() => document.getElementById('editor').contains(document.activeElement)),true,'background native focus blocked');
  await page.keyboard.press('PageDown');
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  assert.equal(await page.evaluate(() => scrollY),before,'background keyboard cannot scroll');
  await page.evaluate(() => { window.backgroundClicks=0; document.addEventListener('click',event => { if (!event.target.closest('dialog')) window.backgroundClicks++; }); });
  await page.mouse.click(1,1); assert.equal(await page.evaluate(() => window.backgroundClicks),0);
  const viewport = await page.screenshot({path:path.join(output,'screenshots',`${name}-dialog-viewport-375.png`),fullPage:false});
  assert.equal(viewport.readUInt32BE(20),812,'capture is exactly the viewport height');
  assert.equal(native.document > native.viewport,true,'fullPage extends beyond the viewport-fixed backdrop');
  await page.setViewportSize({width:1280,height:900});
  await page.locator('#profile-name').focus();
  return {...native,backgroundScrollBlocked:true,backgroundFocusBlocked:true,viewportScreenshotHeight:812};
};
