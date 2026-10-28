// Actual Profile SSR + hydration, including the pending LayoutProvider handoff.
const fs=require('node:fs'),path=require('node:path'),cp=require('node:child_process'),assert=require('node:assert/strict');
const {chromium}=require(process.env.PROFILE_UI_PLAYWRIGHT);
const root=path.resolve(__dirname,'../../..'),output=process.env.PROFILE_UI_THEME_LOGS||process.env.PROFILE_UI_IOS_LOGS;
(async()=>{
  const browser=await chromium.launch({headless:true});
  try{
    for(const base of ['','/demo/v1']) for(const display of ['fr-unset','fr-dark']){
      const html=cp.execFileSync(process.env.PROFILE_UI_HOST||path.join(process.env.CARGO_TARGET_DIR,'debug/stackpress-example-profile-ui-web'),{env:{...process.env,PROFILE_UI_BASE:base,PROFILE_UI_DISPLAY:display},encoding:'utf8',timeout:10000});
      fs.writeFileSync(path.join(output,'profile-source-'+(base?'mounted':'root')+'-'+display+'.html'),html);
      const page=await browser.newPage({colorScheme:'dark'});page.setDefaultTimeout(10000);
      await page.addInitScript(()=>{
        window.rootColors=[];window.themeQueries=0;
        const query=window.matchMedia.bind(window);
        window.matchMedia=value=>{if(value==='(prefers-color-scheme: dark)')window.themeQueries++;return query(value);};
        const sample=()=>{const node=document.querySelector('.sp-root');if(node)rootColors.push(getComputedStyle(node).backgroundColor);requestAnimationFrame(sample);};requestAnimationFrame(sample);
      });
      const errors=[];let release;const gate=new Promise(resolve=>release=resolve);
      page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(/hydration.*(mismatch|error)/i.test(m.text()))errors.push(m.text());});
      await page.route('**/*',async route=>{
        const url=new URL(route.request().url());assert.equal(url.origin,'https://profile-theme.invalid');
        if(url.pathname===base+'/ui/profiles')return route.fulfill({body:html,contentType:'text/html'});
        const relative=url.pathname.slice((base+'/assets/').length),folder=path.join(root,'examples/profile-ui/dist/hydrate'),file=path.resolve(folder,relative);
        assert(file.startsWith(folder+path.sep));if(relative==='start.js')await gate;
        return route.fulfill({path:file,contentType:file.endsWith('.wasm')?'application/wasm':'text/javascript'});
      });
      try{
        await page.goto('https://profile-theme.invalid'+base+'/ui/profiles?id=abc',{waitUntil:'commit'});
        await page.locator('.sp-root').waitFor();
        assert.equal(await page.locator('.sp-root').evaluate(node=>getComputedStyle(node).backgroundColor),'rgb(32, 33, 36)','pre-WASM root must already be dark');
        await page.locator('input').fill('EARLY-profile-draft');await page.evaluate(()=>{window.inputBefore=document.querySelector('input');window.inputBefore.focus();});
        release();await page.waitForFunction(()=>window.profileUiReady);
        assert.equal(await page.locator('.sp-root').getAttribute('data-theme'),'dark');
        assert.equal(await page.locator('[data-initial-theme-pending]').count(),0);
        assert.equal(await page.evaluate(()=>inputBefore===document.querySelector('input')&&document.activeElement===inputBefore),true);
        assert.equal(await page.locator('input').inputValue(),'EARLY-profile-draft');
        await page.emulateMedia({colorScheme:'light'});
        assert.equal(await page.locator('.sp-root').evaluate(node=>getComputedStyle(node).backgroundColor),'rgb(32, 33, 36)');
        await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
        assert.equal(await page.evaluate(()=>window.themeQueries),display==='fr-unset'?1:0);
        assert.equal(await page.evaluate(()=>rootColors.length>0&&rootColors.every(color=>color==='rgb(32, 33, 36)')),true,'no wrong-color content frame across hydration');
        assert.deepEqual(errors,[]);
        await page.screenshot({path:path.join(output,'profile-first-'+(base?'mounted':'root')+'-'+display+'.png'),fullPage:true});
      }finally{release();await page.close();}
    }
    console.log('PASS Profile SSR/hydration first-theme handoff: 4 cases, early draft and node identity');
  }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
