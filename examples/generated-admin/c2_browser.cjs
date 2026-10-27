// Compatibility entry point; all setup/watchdogs live in the shared harness.
require('./proofs/http_browser.cjs')(require('./proofs/format.cjs'))
  .catch(error => { console.error(error); process.exitCode = 1; });
