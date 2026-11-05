// The source-family website proof uses cached Chromium and intercepted requests only.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
assert.equal(process.env.AGGREGATE_PLAYWRIGHT, '/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright');
const {chromium} = require(process.env.AGGREGATE_PLAYWRIGHT);
const [pages, assets, mode, output] = process.argv.slice(2);
async function run() {
  const browser = await chromium.launch({headless:true});
  const results = [];
  try {
    for (const width of [375, 1280]) for (const name of ['profile-detail', 'profile-update', 'profile-search']) {
      const context = await browser.newContext({viewport:{width,height:1000},timezoneId:'UTC'});
      const page = await context.newPage();
      page.setDefaultTimeout(10000);
      const errors = [];
      page.on('pageerror', error => errors.push(String(error)));
      page.on('console', message => {if (/hydration.*(?:mismatch|error)/i.test(message.text())) errors.push(message.text());});
      await page.route('**/*', route => {
        const url = new URL(route.request().url());
        if (route.request().method() === 'POST') return route.fulfill({status:204,body:''});
        if (url.pathname.startsWith('/proof/')) return route.fulfill({contentType:'text/html',body:fs.readFileSync(path.join(pages,name+'.html'))});
        if (url.pathname === '/assets/api-checkpoint.js') return route.fulfill({contentType:'text/javascript',body:`import init,{start_source} from '/client/slice11_website.js'; await init('/client/slice11_website_bg.wasm'); ${mode==='csr'?"document.getElementById('app').replaceChildren();":''} window.disposeSource=start_source(); window.sourceReady=true;`});
        if (url.pathname.startsWith('/client/')) return route.fulfill({contentType:url.pathname.endsWith('.wasm')?'application/wasm':'text/javascript',body:fs.readFileSync(path.join(assets,path.basename(url.pathname)))});
        if (url.pathname.includes('initial-theme-')) return route.fulfill({contentType:'text/javascript',body:fs.readFileSync('crates/framework/framework-view/src/initial-theme.js')});
        // Match the actual API checkpoint asset route, including shared collection/table CSS.
        if (url.pathname === '/styles/global.css') return route.fulfill({contentType:'text/css',body:fs.readFileSync('crates/leptos/src/components/styles.css','utf8')+'\n'+fs.readFileSync('crates/framework/framework-api/src/views/styles.css','utf8')});
        // Native SQL/CRUD is proved by the producer. Relation lookup stays a finite public response.
        return route.fulfill({contentType:'application/json',body:'{"results":[]}'});
      });
      await page.goto('https://aggregate.invalid/proof/'+name,{waitUntil:'networkidle'});
      await page.waitForFunction(() => window.sourceReady === true);
      assert.equal(await page.locator('main').count(),1);
      assert.equal(await page.getByRole('navigation',{name:'Models'}).getByRole('link',{name:'Profiles',exact:true}).count(),1);
      assert.ok((await page.locator('main').innerText()).includes('App Source') || name==='profile-update');
      const seed = await page.locator('#api-management-seed').textContent();
      for (const privateValue of ['SOURCE-SEED-PRIVATE','SourceRecordSchema','SourceEnvironment','@encrypted','@hashed']) assert.equal(seed.includes(privateValue),false,privateValue);
      if (name==='profile-update') {
        const nickname=page.getByLabel('Nickname',{exact:true});
        assert.equal(await nickname.inputValue(),'App Source');
        await nickname.fill('Browser extension');
        assert.equal(await nickname.inputValue(),'Browser extension');
        assert.equal(await page.getByRole('button',{name:'Add Role',exact:true}).count(),1);
        assert.equal(await page.getByRole('button',{name:'Add Reference',exact:true}).count(),1);
        const reference=page.locator('#api-form-field-references-1-value');
        assert.equal(await reference.inputValue(),'1');
        await reference.fill('2');
        const submitted=page.waitForRequest(request => request.method()==='POST');
        await page.getByRole('button',{name:'Save',exact:true}).click({noWaitAfter:true});
        const data=new URLSearchParams((await submitted).postData());
        assert.equal(data.get('nickname'),'Browser extension');
        assert.equal(data.get('references[z]'),'2');
      }
      if (name==='profile-search') {
        assert.equal(await page.getByRole('columnheader',{name:'Nickname',exact:true}).count(),1);
        assert.equal(await page.getByRole('link',{name:'View details',exact:true}).count(),1);
      }
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth+1),false);
      assert.deepEqual(errors,[],[mode,name,width].join(' '));
      await page.evaluate(() => window.disposeSource());
      results.push({mode,name,width,errors});
      await context.close();
    }
  } finally {await browser.close();}
  fs.mkdirSync(output,{recursive:true});
  fs.writeFileSync(path.join(output,mode+'-source.json'),JSON.stringify(results,null,2));
  console.log(mode+': '+results.length+' source-family browser cases passed');
}
run().catch(error => {console.error(error);process.exitCode=1;});
