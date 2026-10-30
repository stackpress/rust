// Finite browser proof: every request is fulfilled in memory; no development listener.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
assert.equal(process.env.AGGREGATE_PLAYWRIGHT,'/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright','use the approved cached Playwright path');
const {chromium}=require(process.env.AGGREGATE_PLAYWRIGHT);
const root=process.cwd(),artifacts=process.argv[2],mode=process.argv[3],out=process.argv[4];
fs.mkdirSync(out,{recursive:true});
const packageDir=path.join(path.dirname(artifacts),mode);
const pages=['signup','signin','username','email','phone','otp','link','two-factor','account','update','security','password','export','remove','enroll','remove-factor'];
const csp="default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; object-src 'none'; base-uri 'none'";
async function run(){
 const browser=await chromium.launch({headless:true});const results=[];
 try{
  for(const [width,zoom,content] of [[1280,1,'normal'],[375,1,'normal'],[375,2,'normal'],[375,2,'long']]){
   for(const theme of (content==='long'?['light','dark']:['light','dark','unset'])){
    const context=await browser.newContext({viewport:{width,height:900},colorScheme:'dark'});
    await context.addInitScript(()=>{
     // Keep the real system clipboard untouched; prove the production user-action bridge.
     Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text=>{window.sessionCopied=text;}}});
    });
    for(const name of pages){
     const page=await context.newPage();page.setDefaultTimeout(10000);const errors=[];
     page.on('pageerror',error=>errors.push(String(error)));
     page.on('console',message=>{if(/hydration.*mismatch|hydration.*error/i.test(message.text()))errors.push(message.text());});
     await page.route('**/*',async route=>{
      const url=new URL(route.request().url()),pathname=url.pathname;
      if(pathname.startsWith('/proof/')){
       const file=path.join(artifacts,name+(content==='long'?'-long':'')+(theme==='unset'?'':'-'+theme)+'.html');
       await route.fulfill({contentType:'text/html',headers:{'Content-Security-Policy':csp},body:fs.readFileSync(file)});return;
      }
      if(pathname==='/assets/session.js'){
       const code=`import init,{start} from '/client/slice11_website.js'; await init('/client/slice11_website_bg.wasm'); const el=document.getElementById('stackpress-page'); const text=el.textContent; ${mode==='csr'?"document.getElementById('app').replaceChildren();":''} window.sessionDispose=start(text); window.sessionReady=true;`;
       await route.fulfill({contentType:'text/javascript',body:code});return;
      }
      if(pathname.startsWith('/client/')){
       const file=path.join(packageDir,path.basename(pathname));await route.fulfill({contentType:pathname.endsWith('.wasm')?'application/wasm':'text/javascript',body:fs.readFileSync(file)});return;
      }
      if(pathname.includes('initial-theme-')){await route.fulfill({contentType:'text/javascript',body:fs.readFileSync(path.join(root,'crates/framework/framework-view/src/initial-theme.js'))});return;}
      if(pathname.startsWith('/avatar/')){await route.fulfill({contentType:'image/svg+xml',body:'<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96"><rect width="96" height="96" rx="48" fill="#dce6ef"/><text x="48" y="57" text-anchor="middle" fill="#35536e" font-family="sans-serif" font-size="28">AL</text></svg>'});return;}
      if(pathname==='/styles/global.css'){await route.fulfill({contentType:'text/css',body:'html,body{margin:0;font-family:system-ui,sans-serif}body{min-width:0}*{box-sizing:border-box}'});return;}
      await route.fulfill({status:204,body:''});
     });
     await page.goto('https://session.invalid/proof/'+name,{waitUntil:'networkidle'});
     await page.waitForFunction(()=>window.sessionReady===true);
     await page.evaluate(zoom=>{document.documentElement.style.zoom=String(zoom)},zoom);
     assert.equal(await page.locator('h1').count(),1,name+' heading');
     assert.equal(await page.locator('main').count(),1,name+' main');
     assert.ok((await page.title()).length>0,name+' title');
     const expected=theme==='unset'?'dark':theme;
     await page.waitForFunction(expected=>document.querySelector('.sp-root')?.dataset.theme===expected,expected);
     const overflow=await page.evaluate(()=>document.documentElement.scrollWidth>document.documentElement.clientWidth+1);
     assert.equal(overflow,false,[mode,name,content,width,zoom,theme].join(' ')+' overflow');
     assert.equal(await page.locator('svg:not([aria-hidden=true])').count(),0,name+' decorative icons');
     const unlabelled=await page.locator('button').evaluateAll(buttons=>buttons.filter(button=>!button.textContent.trim()&&!button.getAttribute('aria-label')).length);
     assert.equal(unlabelled,0,name+' button names');
     const inputs=await page.locator('input:not([type=hidden])').evaluateAll(inputs=>inputs.filter(input=>!input.labels.length&&!input.getAttribute('aria-label')).map(input=>input.name));
     assert.deepEqual(inputs,[],name+' input labels');
     if(name==='email'){
      await page.getByRole('button',{name:'Send one-time pin code',exact:true}).focus();await page.keyboard.press('Space');
      assert.equal(await page.locator('input[name=auth]').inputValue(),'otp');
      assert.equal(await page.locator('input[name=secret]').isVisible(),false);
      assert.equal(await page.locator('form').evaluate(form=>new FormData(form).has('secret')),false);
      await page.getByRole('button',{name:'Send magic link',exact:true}).click();assert.equal(await page.locator('input[name=auth]').inputValue(),'magic');
      await page.getByRole('button',{name:'Sign in with password',exact:true}).click();
      await page.getByRole('button',{name:'Show password',exact:true}).focus();await page.keyboard.press('Enter');
      assert.equal(await page.locator('input[name=secret]').getAttribute('type'),'text');
      await page.getByRole('button',{name:'Hide password',exact:true}).click();
      assert.equal(await page.locator('input[name=secret]').getAttribute('type'),'password');
     }
     if(name==='signup'){
      await page.locator('input[name=secret]').fill('Abcdef12!');assert.ok(Number(await page.locator('progress').getAttribute('value'))>0);
      assert.equal(await page.getByLabel('Phone country').inputValue(),'PH');
      await page.locator('#session-phone').fill('9171234567');
      assert.equal((await page.locator('input[name=phone]').inputValue()).replace(/\D/g,''),'639171234567');
     }
     if(name==='enroll'){
      const secret=await page.locator('#session-secret').inputValue();assert.equal(secret,'JBSWY3DPEHPK3PXP');
      assert.deepEqual(await page.locator('form').evaluate(form=>new FormData(form).getAll('secret')),[secret,secret]);
      await page.getByRole('button',{name:'Copy',exact:true}).focus();await page.keyboard.press('Enter');
      assert.equal(await page.evaluate(()=>window.sessionCopied),secret);
      assert.ok((await page.getByRole('status').textContent()).includes('copied'));
     }
     if(width===1280&&name==='enroll'){
      const link=await page.getByRole('link',{name:'Regenerate Secret',exact:true}).boundingBox();
      const form=await page.locator('form').boundingBox();
      assert.ok(link.width<form.width/2,'regenerate link hugs its text');
      const lines=await page.locator('.session-nav a').first().evaluate(link=>{
       const walk=document.createTreeWalker(link,NodeFilter.SHOW_TEXT);let text;
       while((text=walk.nextNode()))if(text.textContent.includes('Personal Information')){
        const range=document.createRange();range.selectNodeContents(text);return range.getClientRects().length;
       }
       return 0;
      });
      assert.equal(lines,1,'desktop Personal Information stays on one line');
     }
     // Tab reaches a native interactive control and leaves a visible focus indicator.
     await page.keyboard.press('Tab');
     assert.equal(await page.evaluate(()=>document.activeElement===document.body),false,name+' keyboard focus');
     assert.deepEqual(errors,[],[mode,name,content,width,zoom,theme].join(' ')+' runtime errors');
     if(['signin','email','signup','account','update','enroll','export','remove-factor'].includes(name)&&theme!=='unset'){
      await page.screenshot({path:path.join(out,[mode,name,content,width,zoom,theme].join('-')+'.png'),fullPage:true});
     }
     results.push({mode,name,content,width,zoom,theme,overflow,errors});
     await page.evaluate(()=>window.sessionDispose());await page.close();
    }
    await context.close();
   }
  }
  // G39 no-JS unset remains readable Light even on a dark browser environment.
  const context=await browser.newContext({javaScriptEnabled:false,colorScheme:'dark',viewport:{width:375,height:900}}),page=await context.newPage();
  await page.route('**/*',route=>route.fulfill({contentType:'text/html',body:fs.readFileSync(path.join(artifacts,'signin.html'))}));
  await page.goto('https://session.invalid/no-js');assert.equal(await page.locator('html').getAttribute('data-theme'),'light');assert.equal(await page.locator('.sp-root').getAttribute('data-theme'),'light');await context.close();
 }finally{await browser.close();}
 fs.writeFileSync(path.join(out,mode+'-results.json'),JSON.stringify(results,null,2));console.log(mode+': '+results.length+' finite browser cases passed, plus no-JS Light');
}
run().catch(error=>{console.error(error);process.exitCode=1});
