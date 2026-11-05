// Finite browser proof: every application request goes through a real native HTTP adapter
// over stdio/SQLite. Playwright intercepts bytes; this process never opens a listener.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const assert = require('node:assert/strict');
assert.equal(process.env.API_PLAYWRIGHT,'/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright','use the approved cached Playwright path');
const { chromium } = require(process.env.API_PLAYWRIGHT);
const mode = process.env.API_BROWSER_MODE || 'hydrate';
// Local layout diagnostic only; the final gate never sets API_REVIEW_ONLY.
const reviewOnly = process.env.API_REVIEW_ONLY==='1';
const output = process.env.API_PROOF_OUTPUT;
fs.mkdirSync(output, {recursive:true});
const database = path.join(fs.mkdtempSync(path.join(output, 'fixture-')), 'api.sqlite');
const binary = process.env.API_WEB_BIN;
const calls = [], errors = [], unexpected = [];
function native(input) {
  const response = cp.execFileSync(binary, [], {input:JSON.stringify({database,...input}),
    encoding:'utf8', timeout:15000, maxBuffer:8388608, env:{...process.env,TZ:'UTC'}});
  return JSON.parse(response);
}
const json = response => JSON.parse(response.body);
const url = scope => 'https://api.invalid/auth/oauth?client_id=a&redirect_uri=%2Freturn%3Fx%3D1&state=a%26b&scope='+encodeURIComponent(scope);
let browser;
(async () => {
  native({initialize:true});
  const identity = native({jwt:'p'}).body;
  const editor=native({path:'/admin/application/update/a'});
  const seed=JSON.parse(editor.body.match(/<script id="api-management-seed" type="application\/json">(.*?)<\/script>/s)[1]);
  const longName='A collaborative application requesting permission to synchronize your account preferences across a large collection of connected workspaces';
  native({path:'/admin/application/update/a?json',method:'POST',headers:{cookie:'csrf='+seed.csrf[1],'content-type':'application/json'},body:JSON.stringify({csrf:seed.csrf[1],name:longName})});
  // Pin the exact Moment token output, including the intentional literal i (G43).
  assert.match(native({path:'/admin/application/detail/a'}).body, /0 5, 2100 12:iAM/);
  browser = await chromium.launch({headless:true});
  async function context({width=1280,js=true,theme,os='light'}={}) {
    const context = await browser.newContext({viewport:{width,height:1000},javaScriptEnabled:js,colorScheme:os,timezoneId:'UTC'});
    const cookies=[{name:'identity',value:identity,url:'https://api.invalid'}];
    if(theme)cookies.push({name:'stackpress-theme',value:theme,url:'https://api.invalid'});
    await context.addCookies(cookies);
    const page=await context.newPage();page.setDefaultTimeout(12000);
    page.on('pageerror',error=>errors.push(error.message));
    page.on('console',message=>{if(message.type()==='error' && !/^Failed to load resource/.test(message.text()))errors.push(message.text());});
    // Chromium Fetch pauses every redirect hop; Playwright route() skips redirect targets.
    // Debugging uses Playwright's existing pipe, with no application/debugging TCP listener.
    const protocol=await context.newCDPSession(page);
    await protocol.send('Fetch.enable',{patterns:[{urlPattern:'*',requestStage:'Request'}]});
    protocol.on('Fetch.requestPaused',async paused=>{
      const req=paused.request,target=new URL(req.url);
      async function fulfill(status,headers,body) {
        await protocol.send('Fetch.fulfillRequest',{requestId:paused.requestId,responseCode:status,
          responseHeaders:Object.entries(headers).map(([name,value])=>({name,value})),body:Buffer.from(body).toString('base64')});
      }
      try {
        if(target.origin!=='https://api.invalid'){unexpected.push(target.href);return protocol.send('Fetch.failRequest',{requestId:paused.requestId,errorReason:'BlockedByClient'});}
        if(target.pathname==='/return' || target.pathname==='/auth/signin')return fulfill(200,{'content-type':'text/html'},'<!doctype html><p>Application continuation</p>');
        if(target.pathname==='/fixtures/application-logo.svg')return fulfill(200,{'content-type':'image/svg+xml'},fs.readFileSync('examples/api-checkpoint/proofs/application-logo.svg'));
        if(target.pathname==='/logo.png' || target.pathname==='/favicon.ico')return fulfill(200,{'content-type':'image/svg+xml'},'<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50" viewBox="0 0 50 50"><rect width="50" height="50" rx="8" fill="#64748b"/><text x="10" y="34" font-size="28" fill="white">S</text></svg>');
        if(target.pathname==='/assets/api-checkpoint.js') {
          const original=fs.readFileSync('examples/api-checkpoint/client/api-checkpoint.js','utf8');
          const script=process.env.API_AGGREGATE_ASSET==='1' ? original.replace("import init, { start } from './pkg/api_checkpoint_client.js';", "import init, { start_api as start } from './pkg/slice11_website.js';") : original;
          return fulfill(200,{'content-type':'text/javascript'},script);
        }
        if(target.pathname.startsWith('/assets/pkg/')) {
          const file=path.join(process.env.API_WASM_DIR,mode,'pkg',path.basename(target.pathname));
          return fulfill(200,{'content-type':file.endsWith('.wasm')?'application/wasm':'text/javascript'},fs.readFileSync(file));
        }
        const cookies=await context.cookies(target.href),headers={...req.headers};
        // Browser cookies remain the actual identity/CSRF source, including Set-Cookie replies.
        headers.cookie=cookies.map(cookie=>cookie.name+'='+cookie.value).join('; ');
        const result=native({method:req.method,path:target.pathname+target.search,headers,body:req.postData||''});
        calls.push({method:req.method,path:target.pathname+target.search,status:result.status,body:req.postData||'',location:result.headers.location});
        await fulfill(result.status,result.headers,result.body);
      }catch(error){errors.push(String(error));await protocol.send('Fetch.failRequest',{requestId:paused.requestId,errorReason:'Failed'}).catch(()=>{});}
    });
    return {context,page};
  }
  async function ready(page){await page.waitForFunction(()=>typeof window.disposeApiCheckpoint==='function');}
  async function fit(page,name){assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),true,name+' overflow');await page.screenshot({path:path.join(output,mode+'-'+name+'.png'),fullPage:true});}
  // Consent works with native controls and navigation when scripts are disabled.
  if(!reviewOnly) {
    const {context:c,page}=await context({width:320,js:false,os:'dark'});
    await page.goto(url('read unknown write'));assert.equal(await page.title(),'Grant Access');
    assert.equal(await page.getByRole('heading',{name:'Sign Up'}).count(),1);
    assert.equal(await page.getByRole('switch').count(),2);
    const read=page.getByRole('switch').first();assert.equal(await read.isChecked(),false);
    await read.focus();await page.keyboard.press('Space');assert.equal(await read.isChecked(),true);
    assert.equal(await page.locator('html').getAttribute('data-theme'),'light');
    await page.goto(url('unknown'));assert.ok((await page.locator('.instructions').innerText()).includes(longName));await fit(page,'long-app-320');await page.goto(url('read unknown write'));await read.check();
    await fit(page,'no-js-320');await page.getByRole('button',{name:'Grant',exact:true}).click();await page.waitForURL('**/return?**');
    const target=new URL(page.url());assert.equal(target.searchParams.get('state'),'a&b');const code=target.searchParams.get('code');assert.ok(code);
    const session=json(native({path:`/admin/session/detail/${code}?json`})).results;assert.deepEqual(session.scopes,['read']);assert.equal(session.profileId,'p');
    const token=json(native({path:'/auth/oauth/token?code='+code,headers:{authorization:'Anything a:unused'}}));assert.equal(token.results.access_token,code);assert.ok(token.results.access_secret);assert.equal(token.results.user.id,'p');
    assert.equal(json(native({path:'/auth/oauth/token?code='+code,headers:{authorization:'Anything a:wrong'}})).results.access_token,code);
    await page.goto(url('read'));await page.getByRole('link',{name:'Deny',exact:true}).focus();await page.keyboard.press('Enter');await page.waitForURL('**/return?**');assert.equal(new URL(page.url()).searchParams.get('error'),'denied');
    await c.clearCookies();await page.goto(url('read'));await page.waitForURL('**/auth/signin?**');
    assert.equal(new URL(page.url()).searchParams.get('redirect_uri')?.startsWith('/auth/oauth?'),true);
    await c.close();
  }
  // Responsive, theme, keyboard and hydration/CSR all use actual selected page state.
  if(!reviewOnly) for(const width of [320,768,1280])for(const theme of ['light','dark',undefined]) {
    const {context:c,page}=await context({width,theme,os:'dark'});
    await page.goto(url('read write'));await ready(page);
    assert.equal(await page.getByRole('switch').count(),2);assert.equal(await page.getByRole('switch').first().isChecked(),false);
    assert.equal(await page.locator('html').getAttribute('data-theme'),theme||'dark');
    await page.getByRole('switch').first().focus();await page.keyboard.press('Space');assert.equal(await page.getByRole('switch').first().isChecked(),true);
    assert.ok(await page.getByRole('switch').first().evaluate(node=>getComputedStyle(node).outlineStyle!=='none'));
    await fit(page,'consent-'+width+'-'+(theme||'unset'));
    await page.goto('https://api.invalid/admin/application/detail/a');await ready(page);assert.ok((await page.locator('main').innerText()).includes('app-plaintext'));assert.ok((await page.locator('main').innerText()).includes('Ada Lovelace'));await fit(page,'detail-'+width+'-'+(theme||'unset'));
    await c.close();
  }
  // Requested review captures: check real images, distinct source pill badges, and local
  // table scrolling instead of crushing columns or overflowing the document.
  for(const [width,theme] of [[1280,'light'],[375,'dark']]) {
    const {context:c,page}=await context({width,theme});
    await page.goto('https://api.invalid/admin/application/detail/a');await ready(page);
    await page.locator('img[alt="Image"]').evaluate(image=>image.decode());
    assert.equal(await page.locator('.api-tag').count(),2);
    assert.equal(await page.locator('.api-tag').first().innerText(),'read');
    assert.equal(await page.locator('.api-tag').last().innerText(),'write');
    assert.equal(await page.getByRole('link',{name:'Update',exact:true}).locator('svg[aria-hidden=true]').count(),1);
    await fit(page,'review-detail-'+width+'-'+theme);
    await page.goto('https://api.invalid/admin/application/search');await ready(page);
    assert.equal(await page.getByText('Active action',{exact:true}).count(),0);
    const sizes=await page.locator('.api-search-form .sp-input,.api-search-form .sp-button').evaluateAll(nodes=>nodes.map(node=>node.getBoundingClientRect().height));
    assert.equal(new Set(sizes).size,1);
    const bottoms=await page.locator('.api-search-form .sp-input,.api-search-form .sp-button').evaluateAll(nodes=>nodes.map(node=>Math.round(node.getBoundingClientRect().bottom)));
    assert.equal(new Set(bottoms).size,1,JSON.stringify(await page.locator('.api-search-form').evaluate(node=>({html:node.outerHTML,items:[...node.children].map(child=>({class:child.className,rect:child.getBoundingClientRect().toJSON(),align:getComputedStyle(child).alignSelf,margin:getComputedStyle(child).margin,help:child.querySelector('.sp-help')?.outerHTML}))}))));
    assert.equal(await page.locator('caption').evaluate(node=>node.getBoundingClientRect().height),1);
    const region=page.getByRole('region',{name:'Search results'});await region.focus();
    await page.keyboard.press('ArrowRight');
    assert.equal(await region.evaluate(node=>node.scrollWidth>node.clientWidth),true);
    // Review images begin at column zero after the independent keyboard-scroll assertion.
    await page.locator('.api-table,.api-table .sp-table-scroll').evaluateAll(nodes=>nodes.forEach(node=>node.scrollLeft=0));
    await fit(page,'review-search-'+width+'-'+theme);
    await page.locator('.api-filter-panel>summary').click();
    assert.equal(await page.getByRole('switch',{name:'Active',exact:true}).count(),1);
    assert.equal(await page.locator('.api-filter-panel select').count(),0);
    await fit(page,'review-filters-'+width+'-'+theme);await c.close();
  }
  // Real generated controls create, edit, copy and manage a related session without a key console.
  if(!reviewOnly) {
    const {context:c,page}=await context({width:768});
    await page.goto('https://api.invalid/admin/application/create');await ready(page);
    assert.equal(await page.locator('input[name=website]').getAttribute('type'),'url');assert.equal(await page.locator('input[name=website]').evaluate(node=>node.type),'url');
    const profile=page.getByRole('combobox',{name:'Profile'});await profile.fill('Ada');await page.getByRole('option',{name:'Ada Lovelace'}).click();
    await page.getByLabel('Name',{exact:true}).fill('Browser created application');
    await page.getByLabel('Expires',{exact:true}).fill('2100-01-01T00:00:00Z');await page.getByLabel('Expires',{exact:true}).press('Tab');
    await page.getByRole('button',{name:'Save',exact:true}).click();await page.waitForURL('**/admin/application/detail/*');await ready(page);
    const id=page.url().split('/').pop();assert.ok(id.length===24);const created=json(native({path:'/admin/application/detail/'+id+'?json'})).results;assert.equal(created.profileId,'p');assert.ok(created.secret.length===24);
    await page.getByRole('link',{name:'Update',exact:true}).click();await ready(page);await page.getByLabel('Name',{exact:true}).fill('Updated in browser');await page.getByRole('button',{name:'Save',exact:true}).click();await page.waitForURL('**/detail/'+id);await ready(page);assert.ok((await page.locator('main').innerText()).includes('Updated in browser'));
    await page.getByRole('link',{name:'Copy',exact:true}).click();await ready(page);await page.getByLabel('Name',{exact:true}).fill('Copied in browser');await page.getByRole('button',{name:'Save',exact:true}).click();await page.waitForURL(target=>target.pathname.includes('/detail/')&&!target.pathname.endsWith('/'+id));await ready(page);assert.notEqual(page.url().split('/').pop(),id);
    await page.goto('https://api.invalid/admin/session/create?applicationId='+id);await ready(page);await page.getByRole('combobox',{name:'Profile'}).fill('Grace');await page.getByRole('option',{name:'Grace Hopper'}).click();await page.getByLabel('Expires',{exact:true}).fill('2100-01-01T00:00:00Z');await page.getByLabel('Expires',{exact:true}).press('Tab');await page.getByRole('button',{name:'Save',exact:true}).click();await page.waitForURL('**/admin/session/detail/*');await ready(page);assert.ok((await page.locator('main').innerText()).includes('Grace Hopper'));assert.ok((await page.locator('main').innerText()).includes('Updated in browser'));
    await page.getByRole('link',{name:'Remove',exact:true}).click();await ready(page);await page.getByRole('button',{name:'Confirm',exact:true}).click();await page.waitForURL('**/session/search');await ready(page);
    await fit(page,'management-search');await c.close();
  }
  assert.deepEqual(errors,[]);assert.deepEqual(unexpected,[]);
  fs.writeFileSync(path.join(output,mode+'-browser.json'),JSON.stringify({mode,cases:reviewOnly?2:13,calls,errors,unexpected},null,2));
  console.log(mode+': '+(reviewOnly?2:13)+' finite browser scenarios passed; no listener or outbound app requests');
})().catch(error=>{console.error(error);process.exitCode=1;}).finally(async()=>{fs.writeFileSync(path.join(output,mode+'-debug.json'),JSON.stringify({calls,errors,unexpected},null,2));if(browser)await browser.close();});
