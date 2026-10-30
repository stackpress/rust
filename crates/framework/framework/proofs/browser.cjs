// Finite facade proof: all application/module/WASM requests are fulfilled from memory.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
assert.equal(process.env.AGGREGATE_PLAYWRIGHT,'/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright','use the approved cached Playwright path');
const {chromium}=require(process.env.AGGREGATE_PLAYWRIGHT);
const [pages,assets,mode,out]=process.argv.slice(2);
async function main(){
 const browser=await chromium.launch({headless:true});const results=[];
 try{
  for(const width of [1280,375])for(const name of ['signin','email','account']){
   const context=await browser.newContext({viewport:{width,height:900}});
   const page=await context.newPage();const errors=[];
   page.on('pageerror',error=>errors.push(String(error)));
   page.on('console',message=>{if(/hydration.*mismatch|hydration.*error/i.test(message.text()))errors.push(message.text());});
   await page.route('**/*',async route=>{
    const pathname=new URL(route.request().url()).pathname;
    if(pathname.startsWith('/proof/'))return route.fulfill({contentType:'text/html',body:fs.readFileSync(path.join(pages,name+'.html'))});
    if(pathname==='/assets/session.js')return route.fulfill({contentType:'text/javascript',body:`import init,{start} from '/client/slice11_website.js'; await init('/client/slice11_website_bg.wasm'); const seed=document.getElementById('stackpress-page').textContent; ${mode==='csr'?"document.getElementById('app').replaceChildren();":''} window.aggregateDispose=start(seed); window.aggregateReady=true;`});
    if(pathname.startsWith('/client/'))return route.fulfill({contentType:pathname.endsWith('.wasm')?'application/wasm':'text/javascript',body:fs.readFileSync(path.join(assets,path.basename(pathname)))});
    return route.fulfill({status:204,body:''});
   });
   await page.goto('https://aggregate.invalid/proof/'+name,{waitUntil:'networkidle'});
   await page.waitForFunction(()=>window.aggregateReady===true);
   assert.equal(await page.locator('main').count(),1);
   assert.equal(await page.locator('h1').count(),1);
   assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>document.documentElement.clientWidth+1),false);
   if(name==='email'){
    await page.getByRole('button',{name:'Send one-time pin code',exact:true}).click();
    assert.equal(await page.locator('input[name=auth]').inputValue(),'otp');
    await page.getByRole('button',{name:'Sign in with password',exact:true}).click();
    await page.getByRole('button',{name:'Show password',exact:true}).focus();await page.keyboard.press('Enter');
    assert.equal(await page.locator('input[name=secret]').getAttribute('type'),'text');
   }
   assert.deepEqual(errors,[],[mode,name,width].join(' '));
   await page.evaluate(()=>window.aggregateDispose());
   results.push({mode,name,width,errors});await context.close();
  }
 }finally{await browser.close();}
 fs.mkdirSync(out,{recursive:true});fs.writeFileSync(path.join(out,mode+'.json'),JSON.stringify(results,null,2));
 console.log(mode+': '+results.length+' finite facade browser cases passed');
}
main().catch(error=>{console.error(error);process.exitCode=1});
