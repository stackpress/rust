// Permanent delayed-acknowledgement regression; actual compiled HTTP and generated SQLite ops.
const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');
if (!process.env.ADMIN_PLAYWRIGHT) throw Error('ADMIN_PLAYWRIGHT is required');
const { chromium } = require(process.env.ADMIN_PLAYWRIGHT);
const artifacts = process.env.ADMIN_REVIEW_ARTIFACTS;
const output = process.env.ADMIN_BROWSER_OUTPUT || path.join(process.env.ADMIN_PROOF_ROOT || '/private/tmp/s8-admin', 'races');
assert.ok(artifacts, 'ADMIN_REVIEW_ARTIFACTS must name the matching finite build');
fs.mkdirSync(output, { recursive: true });
const sha = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const frame = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
const records = [];
let browser, timer;
async function one(mode, mount, action, outcome) {
  const directory = fs.mkdtempSync(path.join(output, 'race-fixture-'));
  const database = path.join(directory, 'admin.sqlite');
  const tag = `${mode}-${mount === '/' ? 'root' : 'admin'}-${action}-${outcome}`;
  const errors = [], unexpected = [];
  let release, committed;
  const holding = new Promise(resolve => { release = resolve; });
  const persisted = new Promise(resolve => { committed = resolve; });
  let input, writes = 0;
  const native = (pathname, method = 'GET', body, extra = {}) => JSON.parse(cp.execFileSync(path.join(artifacts, 'admin-web'), ['--request'], {
    input: JSON.stringify({ database, mount, actor: 'editor', path: pathname, method, body, csr: mode === 'csr', headers: { origin: 'https://admin.invalid', 'x-admin-csrf': 'finite-session-csrf' }, ...extra }),
    encoding: 'utf8', timeout: 10000, killSignal: 'SIGKILL', maxBuffer: 2097152
  }));
  let context;
  try {
    assert.equal(native('/', 'GET', undefined, { initialize: true }).status, 200);
    context = await browser.newContext({ viewport: { width: 1280, height: 1000 } });
    const page = await context.newPage();
    page.setDefaultTimeout(10000);
    page.on('pageerror', error => errors.push(error.message));
    await page.route('**/*', async route => {
      const request = route.request(), target = new URL(request.url());
      if (target.origin !== 'https://admin.invalid') { unexpected.push(target.href); return route.abort(); }
      const prefix = mount + 'assets/';
      if (target.pathname.startsWith(prefix)) {
        const root = path.join(artifacts, 'assets', mode);
        const file = path.resolve(root, target.pathname.slice(prefix.length));
        assert.ok(file.startsWith(root + path.sep));
        return route.fulfill({ path: file, contentType: file.endsWith('.wasm') ? 'application/wasm' : 'text/javascript' });
      }
      const body = request.postData() ? JSON.parse(request.postData()) : undefined;
      const writing = request.method() === 'POST' && target.pathname.endsWith(`/profile/${action}`);
      const reply = native(target.pathname + target.search, request.method(), body, writing ? {actor:outcome==='denied'?'reader':'editor',fault:outcome==='unknown'} : {});
      if (request.method() === 'POST' && target.pathname.endsWith(`/profile/${action}`)) {
        input = body; writes++;
        assert.equal(reply.status, {saved:200,validation:400,denied:403,unknown:500}[outcome]);
        // The native process has closed its pool after the real commit; only delivery is held.
        committed(reply);
        await holding;
      }
      return route.fulfill({ status: reply.status, headers: reply.headers, body: reply.body });
    });
    if(action==='update')assert.equal(native(`${mount}api/profile/create`,'POST',{input:{id:'race',name:'Initial value'}}).status,200);
    await page.goto(`https://admin.invalid${mount}profile/${action}${action==='update'?'?key='+encodeURIComponent(JSON.stringify({id:'race'})):''}`);
    await page.waitForFunction(() => window.adminReady === true);
    if(action==='create')await page.locator('#admin-field-id').fill('race');
    const sentName=outcome==='validation'?'':'Submitted value';
    await page.locator('#admin-field-name').fill('temporary');
    await page.locator('#admin-field-name').fill(sentName);
    await page.getByRole('button', { name: 'Save', exact: true }).click();
    await persisted;
    assert.equal((await page.locator('#operation-status').textContent()).trim(), 'Submitting…');
    assert.equal(await page.locator('#admin-field-name').isEnabled(), true);
    assert.equal(await page.getByRole('button',{name:'Save',exact:true}).isDisabled(),true);
    await page.locator('#admin-form').dispatchEvent('submit');
    await page.locator('#admin-field-name').fill('Later unsaved value');
    await frame(page);
    const during = JSON.parse(await page.locator('#admin-draft').textContent());
    assert.equal(during.name, 'Later unsaved value');
    release();
    await page.waitForFunction(() => document.querySelector('#operation-status').textContent.trim() !== 'Submitting…');
    await frame(page);
    const ui = await page.locator('#admin-field-name').inputValue();
    const status = (await page.locator('#operation-status').textContent()).trim();
    const result = native(`${mount}api/profile/get`, 'POST', { key: { id: 'race' } });
    assert.equal(result.status, action==='update'||['saved','unknown'].includes(outcome)?200:404);
    const stored = JSON.parse(result.body).results;
    const expectedStored=['saved','unknown'].includes(outcome)?'Submitted value':action==='update'?'Initial value':undefined;
    assert.equal((input.input||input.patch).name, sentName);
    assert.equal(stored?.name, expectedStored);
    assert.equal(ui, 'Later unsaved value');
    if(outcome==='saved')assert.equal(status, 'Saved the submitted version. You have newer unsaved changes.');
    else if(outcome==='unknown'){assert.match(status,/unknown for the submitted version/);assert.match(status,/newer unsaved changes/);assert.doesNotMatch(status,/Saved/);}
    else assert.match(status,/Submitted version:.*newer unsaved changes/);
    if(outcome==='validation')assert.match(await page.locator('#admin-field-name-error').textContent(),/Submitted version: Name <\/script> 🦀 is required/);
    assert.equal(await page.getByRole('button', { name: 'Save', exact: true }).isEnabled(), outcome!=='unknown');
    assert.equal(writes,1,'duplicate submit and automatic replay are blocked');
    assert.deepEqual(errors, []);
    assert.deepEqual(unexpected, []);
    await page.screenshot({ path: path.join(output, `write-races-${tag}.png`), fullPage: true });
    records.push({ mode, mount, action, outcome, submitted: (input.input||input.patch).name, database: stored?.name, visibleDraft: ui, status, regressionPassed: true, errors, unexpected });
    console.log(JSON.stringify(records.at(-1)));
    await page.evaluate(() => window.adminDispose());
  } finally {
    release();
    if (context) await context.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
(async () => {
  try {
    browser = await chromium.launch({ headless: true });
    await Promise.race([
      (async () => { for (const mode of ['hydrate', 'csr']) for (const mount of ['/', '/admin/']) for(const action of ['create','update']) for(const outcome of ['saved','validation','denied','unknown']) await one(mode, mount, action, outcome); })(),
      new Promise((_, reject) => { timer = setTimeout(() => reject(Error('inner counterexample deadline')), 105000); })
    ]);
    fs.writeFileSync(path.join(output, 'write-races.json'), JSON.stringify({
      sourceHead: cp.execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
      source: 'examples/generated-admin/shared/src/crud.rs',
      sourceSha256: sha('examples/generated-admin/shared/src/crud.rs'),
      artifacts, nativeSha256: sha(path.join(artifacts, 'admin-web')), records
    }, null, 2));
  } finally { clearTimeout(timer); if (browser) await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
