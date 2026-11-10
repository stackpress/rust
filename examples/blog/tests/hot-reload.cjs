// Finite client protocol/fallback tests: no browser, network, listener or shared environment.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const cp = require('node:child_process');
const source = fs.readFileSync(path.join(__dirname, '../web/src/hot-reload.js'), 'utf8');
// Retain the existing gate's syntax check alongside this guard's executable tests.
cp.execFileSync(process.execPath, ['--check', path.join(__dirname, 'browser.cjs')]);
cp.execFileSync(process.execPath, ['--check', path.join(__dirname, '../web/src/hot-reload.js')]);
/** Execute the shipped guard against owned DOM/socket/patcher state. */
function run({ markers = ['hot-reload|view-1|open', 'hot-reload|view-1|close'], patch, ready = true, failed = false, invalid = false } = {}) {
  const sent = [], scheduled = [], state = { calls: 0, now: 0, ready };
  const context = {
    msg: { view: invalid ? 'invalid' : JSON.stringify([['view-1', []]]), revision: 7 },
    ws: { blogPatchFailed: failed, send: value => sent.push(JSON.parse(value)) },
    document: {
      get readyState() { return state.ready ? 'complete' : 'loading'; },
      documentElement: { get dataset() { return state.ready ? { blogReady: 'hydrate' } : {}; } },
      body: {}, createTreeWalker() {
        let index = -1;
        return { nextNode: () => ++index < markers.length, get currentNode() { return { textContent: markers[index] }; } };
      }
    },
    NodeFilter: { SHOW_COMMENT: 128 }, Set, JSON,
    Date: { now: () => state.now },
    setTimeout: callback => scheduled.push(callback),
    console: { warn() {} },
    patch: data => { state.calls++; if (patch) patch(context, data); }
  };
  const originalWarn = context.console.warn;
  vm.runInNewContext(source, context);
  assert.equal(context.console.warn, originalWarn);
  return { sent, scheduled, state, context };
}
assert.equal(run().state.calls, 1);
assert.equal(run().sent.length, 0);
for (const scenario of [ { markers: [] }, { failed: true }, { invalid: true },
  { patch: () => { throw Error('patch failed'); } },
  { patch: context => context.console.warn('official patcher caught a failure') } ]) {
  const result = run(scenario);
  assert.equal(result.sent.length, 1);
  assert.equal(result.sent[0].reload, true);
  assert.equal(result.sent[0].revision, 7);
}
const loading = run({ ready: false });
assert.equal(loading.state.calls, 0);
loading.state.ready = true;
loading.scheduled.shift()();
assert.equal(loading.state.calls, 1);
const timeout = run({ ready: false });
timeout.state.now = 30000;
timeout.scheduled.shift()();
assert.equal(timeout.sent[0].reload, true);
console.log('Hot-reload guard: ready, pending, timeout, CSR/missing markers, thrown/caught patch failures passed');

// Execute the shipped renderer selection before hydration as well as on active pages.
const reloadMode = fs.readFileSync(path.join(__dirname, '../web/src/reload-mode.js'), 'utf8');
for (const renderer of ['hydrate', 'csr']) {
  for (const notification of ['all', 'hydrate', 'csr']) {
    let reloads = 0;
    const msg = notification === 'all' ? { all: true } : { blog_reload: notification };
    vm.runInNewContext(reloadMode, { msg, URL, window: { location: {
      href: `http://localhost:3000/?renderer=${renderer}`, reload() { reloads++; }
    } } });
    assert.equal(reloads, Number(notification === 'all' || notification === renderer));
  }
}
console.log('Deferred renderer reload: hydration and CSR notifications select only their own tabs');
