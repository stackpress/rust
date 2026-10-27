// Compatibility entry point; all setup/watchdogs live in the shared harness.
require('./proofs/http_browser.cjs')(require('./proofs/crud.cjs'))
  .catch(error => { console.error(error); process.exitCode = 1; });
