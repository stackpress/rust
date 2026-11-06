// Real WASM/Chromium editing; all page requests are fulfilled in-process.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const cached = '/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright';
assert.equal(process.env.EDITOR_PLAYWRIGHT, cached);
const {chromium} = require(cached);
const out = process.argv[2];

async function prove(page, mode, width, theme) {
  const errors = [];
  const network = [];
  page.on('pageerror', error => errors.push(String(error)));
  page.on('console', message => {
    if (message.type() === 'error' || /hydration.*(?:mismatch|error)/i.test(message.text())) errors.push(message.text());
  });
  await page.route('**/*', route => {
    const url = new URL(route.request().url());
    network.push({method:route.request().method(), path:url.pathname});
    if (url.pathname === '/') return route.fulfill({contentType:'text/html',body:fs.readFileSync(path.join(out,'page.html'),'utf8').replace('data-theme="light"',`data-theme="${theme}"`)});
    if (url.pathname === '/start.js') return route.fulfill({contentType:'text/javascript',body:`import init,{start} from '/editor.js'; await init('/editor_bg.wasm'); ${mode==='csr'?"document.getElementById('app').replaceChildren();":''} window.disposeEditor=start(); window.editorReady=true;`});
    if (['/editor.js','/editor_bg.wasm'].includes(url.pathname)) return route.fulfill({contentType:url.pathname.endsWith('.wasm')?'application/wasm':'text/javascript',body:fs.readFileSync(path.join(out,mode,path.basename(url.pathname)))});
    // No network image upload or remote media fetch is needed by this proof.
    return route.fulfill({status:200,body:''});
  });
  await page.goto('https://editor.invalid/',{waitUntil:'networkidle'});
  await page.waitForFunction(() => window.editorReady);
  const form = page.locator('#editor-form');
  const editable = form.getByRole('textbox',{name:'Rich Text Editor',exact:true});
  const source = form.getByRole('textbox',{name:'HTML Source',exact:true});
  const button = name => form.getByRole('button',{name,exact:true});
  const html = () => editable.innerHTML();
  const hidden = () => page.locator('#hidden').inputValue();
  const updates = async () => JSON.parse(await page.locator('#updates').textContent());
  const changes = async () => JSON.parse(await page.locator('#changes').textContent());
  const selectAll = async () => editable.evaluate(node => {
    node.focus(); const range=document.createRange(); range.selectNodeContents(node);
    const selection=getSelection(); selection.removeAllRanges(); selection.addRange(range);
  });
  const reset = async value => {
    await button('Switch to Code View').click();
    await source.fill(value);
    await button('Switch to TextEditor View').click();
  };
  assert.equal(await html(),'<p>Seed <strong>HTML</strong></p>');
  assert.equal(await hidden(),await html());
  assert.equal(await page.locator('#generated-display mark').getAttribute('data-authored'),'yes');
  assert.equal(await page.locator('#generated').getByRole('button',{name:'Bold',exact:true}).count(),1);
  assert.equal(await page.locator('#generated input[name=contents]').inputValue(),'<p>Generated <em>HTML</em></p>');
  const generatedEditable=page.locator('#generated').getByRole('textbox',{name:'Rich Text Editor',exact:true});
  assert.equal(await generatedEditable.getAttribute('dir'),null);
  await page.locator('#generated').getByRole('button',{name:'Toggle Text Direction',exact:true}).click();
  assert.equal(await generatedEditable.getAttribute('dir'),'rtl');
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth+1),false);

  // Keyboard activation, exact raw HTML editing, form submission and source-mode disablement.
  await button('Switch to Code View').focus();
  await page.keyboard.press('Enter');
  assert.equal(await button('Bold').isDisabled(),true);
  const raw = '<p data-authored="yes"><em>Round &amp; trip 雪</em></p>';
  await source.fill(raw);
  assert.equal(await hidden(),raw);
  assert.equal(JSON.parse(await page.locator('#submitted-snapshots').textContent()).at(-1),raw,
    'onChange observes the already-updated hidden form value');
  await button('Switch to TextEditor View').focus();
  await page.keyboard.press('Space');
  assert.equal(await html(),raw);
  assert.equal(await page.evaluate(() => new FormData(document.getElementById('editor-form')).get('contents')),raw);
  await editable.click();
  await page.keyboard.press('End');
  await page.keyboard.type(' keyboard');
  assert.match(await hidden(),/keyboard/);
  assert.equal(await hidden(),await html());

  if (width === 1280 && theme === 'light') {
    // Exiting source mode publishes the browser-canonical HTML, while Frui's
    // synchronous hidden write still contains the typed source during onChange.
    await button('Switch to Code View').click();
    const authored="<P data-case='source'>Mixed<BR/></P>";
    await source.fill(authored);
    await button('Switch to TextEditor View').click();
    const canonical='<p data-case="source">Mixed<br></p>';
    assert.equal(await html(),canonical);
    assert.equal((await changes()).at(-1),canonical);
    assert.equal(JSON.parse(await page.locator('#submitted-snapshots').textContent()).at(-1),authored);
    assert.equal(await hidden(),canonical);
    // Each browser command reaches the DOM; its callback order is input then command.
    for (const [label,command] of [
      ['Bold','bold'],['Italic','italic'],['Underline','underline'],['Strikethrough','strikeThrough'],
      ['Subscript','subscript'],['Superscript','superscript'],['Paragraph','formatBlock'],
      ['Blockquote','formatBlock'],['Indent','indent'],['Outdent','outdent'],
      ['Align Left','justifyLeft'],['Align Center','justifyCenter'],['Align Right','justifyRight'],
      ['Ordered List','insertOrderedList'],['Unordered List','insertUnorderedList'],['Horizontal Rule','insertHorizontalRule']]) {
      await reset('Selected text'); await selectAll();
      const before=(await changes()).length;
      const beforeUpdates=(await updates()).length;
      await button(label).click();
      assert.deepEqual((await updates()).slice(-2),['input',command],label);
      // execCommand may additionally fire the browser's native input event. Frui
      // forwards it as well as its explicit input call; do not deduplicate it here.
      const inputCount=(await updates()).slice(beforeUpdates).filter(action=>action==='input').length;
      assert.equal((await changes()).length,before+inputCount,label);
      assert.equal(await hidden(),await html(),label);
      if (label === 'Bold') assert.match(await html(),/<b>Selected text<\/b>/);
      if (label === 'Ordered List') assert.match(await html(),/<ol>/);
    }
    for (const [label,value,command,resetValue] of [
      ['Font','Georgia','fontName',true],['Font Size','18px','fontSize',true],
      ['Block Format','h2','formatBlock',false],['Insert Template','<p><strong>Header</strong><br>Content</p>','insertHTML',true]]) {
      await reset('Selected text'); await selectAll();
      await form.getByLabel(label,{exact:true}).selectOption(value);
      assert.equal((await updates()).at(-1),command);
      assert.equal(await form.getByLabel(label,{exact:true}).inputValue(),resetValue?'':value);
    }
    for (const [label,color,command] of [['Text Color','#123456','foreColor'],['Highlight','#aabbcc','hiliteColor']]) {
      await reset('Selected text'); await selectAll();
      await form.getByLabel(label,{exact:true}).fill(color);
      await form.getByLabel(label,{exact:true}).dispatchEvent('change');
      assert.equal((await updates()).at(-1),command);
      assert.equal(await hidden(),await html());
    }
    await reset('<span class="__frui-form-text-editor-t-code">styled</span>'); await selectAll();
    await form.getByLabel('Text Style',{exact:true}).selectOption('shadow');
    assert.match(await html(),/__frui-form-text-editor-t-shadow/);
    assert.equal((await html()).includes('__frui-form-text-editor-t-code'),false);
    await selectAll(); await button('Remove Format').click();
    assert.equal((await updates()).at(-1),'removeFormat');
    assert.equal((await html()).includes('__frui-form-text-editor-t-'),false);
    await form.getByLabel('Line Height',{exact:true}).selectOption('1.5');
    assert.equal(await editable.evaluate(node => node.style.lineHeight),'1.5');
    await button('Right to Left').click(); assert.equal(await editable.getAttribute('dir'),'rtl');
    await button('Toggle Text Direction').click(); assert.equal(await editable.getAttribute('dir'),'ltr');
    await editable.evaluate(node=>node.setAttribute('dir',''));
    await button('Toggle Text Direction').click(); assert.equal(await editable.getAttribute('dir'),'rtl');
    await button('Left to Right').click(); assert.equal(await editable.getAttribute('dir'),'ltr');
    await button('Toggle Block Visibility').click(); assert.match(await editable.getAttribute('class'),/frui-form-text-editor-show-block/);

    // Prompt adapters use actual DOM insertion. Cancellation and rejected link schemes are inert.
    await page.evaluate(() => {window.promptReplies=[]; window.prompt=() => window.promptReplies.shift() ?? null;});
    const prompt = async (label,replies) => {
      await reset('Selected text'); await selectAll();
      await page.evaluate(replies => {window.promptReplies=replies;},replies);
      await button(label).click();
    };
    await prompt('Insert Table',['2','3']); assert.equal(await editable.locator('td').count(),6);
    await prompt('Insert Table',['0x2','3']); assert.equal(await editable.locator('td').count(),6);
    await prompt('Insert Table',['2.8','1']); assert.equal(await editable.locator('td').count(),2);
    await prompt('Insert Table',[null,'3']); assert.equal(await editable.locator('table').count(),0);
    await prompt('Insert Table',['invalid','3']); assert.equal(await editable.locator('table').count(),0);
    await prompt('Insert Link',['https://example.invalid/page']); assert.equal(await editable.locator('a').getAttribute('href'),'https://example.invalid/page');
    await prompt('Insert Link',['javascript:app()']); assert.equal(await editable.locator('a').count(),0);
    await prompt('Insert Video',['https://example.invalid/video']); assert.equal(await editable.locator('iframe').getAttribute('src'),'https://example.invalid/video');
    await prompt('Insert Audio',['https://example.invalid/audio']); assert.equal(await editable.locator('audio').getAttribute('src'),'https://example.invalid/audio');
    await prompt('Insert Math',['x^2']); assert.match(await html(),/\[Math: x\^2\]/);

    // FileReader is real. Wrong MIME is ignored; gallery reads each image, with no HTTP upload.
    const png=Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScLbtAAAAABJRU5ErkJggg==','base64');
    await reset('Image '); await selectAll();
    await form.locator('input[type=file]').nth(0).setInputFiles({name:'no.txt',mimeType:'text/plain',buffer:Buffer.from('no')});
    assert.equal(await editable.locator('img').count(),0);
    await form.locator('input[type=file]').nth(0).setInputFiles({name:'one.png',mimeType:'image/png',buffer:png});
    await page.waitForFunction(() => document.querySelector('#editor-form [contenteditable] img'));
    assert.match(await editable.locator('img').getAttribute('src'),/^data:image\/png;base64,/);
    await form.locator('input[type=file]').nth(1).setInputFiles([{name:'two.png',mimeType:'image/png',buffer:png},{name:'three.png',mimeType:'image/png',buffer:png}]);
    await page.waitForFunction(() => document.querySelectorAll('#editor-form [contenteditable] img').length === 3);
    await reset('History '); await editable.click(); await page.keyboard.type('typed');
    await button('Undo').click(); assert.equal((await updates()).at(-1),'undo');
    await button('Redo').click(); assert.equal((await updates()).at(-1),'redo');

    // Native save/fullscreen execute locally; preview/print host requests are observed in-process.
    await page.evaluate(() => {window.printCalls=0;window.print=()=>window.printCalls++;window.openCalls=[];window.open=(...args)=>{window.openCalls.push(args);return {document:{write:s=>{window.previewHtml=s;},close:()=>{}}};};});
    await button('Preview Content').click(); assert.match(await page.evaluate(() => window.previewHtml),/<html><body>/);
    await button('Print').click(); assert.equal(await page.evaluate(() => window.printCalls),1);
    const download = page.waitForEvent('download'); await button('Save as HTML').click();
    assert.equal((await download).suggestedFilename(),'editor-content.html');
    await button('Toggle Fullscreen').click();
    await page.waitForFunction(() => document.fullscreenElement !== null);
    await page.evaluate(() => document.exitFullscreen());

    // Upstream external value effects change the hidden/inactive surfaces, preserving active HTML.
    const before = await html(); await page.locator('#external').click();
    assert.equal(await hidden(),'<p>External</p>'); assert.equal(await html(),before);
    await button('Switch to Code View').click(); assert.equal(await source.inputValue(),before);
  }
  await page.screenshot({path:path.join(out,`${mode}-${width}-${theme}.png`),fullPage:true});
  assert.equal(network.some(request=>request.method!=='GET'),false,'editor made a network upload');
  await page.evaluate(() => {
    // A real FileReader completion queued before disposal must not access dead root state.
    const toggle=[...document.querySelectorAll('#editor-form button')].find(node=>node.getAttribute('aria-label')==='Switch to TextEditor View');
    toggle?.click();
    const transfer=new DataTransfer();
    transfer.items.add(new File(['pending'],'late.png',{type:'image/png'}));
    const picker=document.querySelector('#editor-form input[type=file]');
    picker.files=transfer.files;
    picker.dispatchEvent(new Event('change',{bubbles:true}));
    window.disposeEditor();window.disposeEditor();
  });
  await page.waitForTimeout(50);
  assert.deepEqual(errors,[],[mode,width,theme].join(' '));
  return {mode,width,theme,requests:network.length,errors};
}

(async () => {
  const browser=await chromium.launch({headless:true});
  const results=[];
  try {
    for (const mode of ['hydrate','csr']) for (const width of [375,1280]) for (const theme of ['light','dark']) {
      const context=await browser.newContext({viewport:{width,height:1000},acceptDownloads:true});
      const page=await context.newPage(); page.setDefaultTimeout(10000);
      results.push(await prove(page,mode,width,theme)); await context.close();
    }
  } finally {await browser.close();}
  fs.writeFileSync(path.join(out,'browser.json'),JSON.stringify(results,null,2));
  console.log(`PASS ${results.length} editor SSR/hydrate/CSR browser/theme/viewport cases`);
})().catch(error=>{console.error(error);process.exitCode=1;});
