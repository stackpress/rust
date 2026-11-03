// Shared native-form assertions for the ordinary and formatted admin scenarios.
const assert = require('node:assert/strict');

module.exports = async function booleanFilter(page, id, name) {
  assert.equal(await page.locator(`#${id}-presence`).count(),0);
  assert.deepEqual(await page.locator(`input[name="${name}"]`).evaluateAll(nodes=>nodes.map(node=>[node.type,node.value])),[['hidden','0'],['checkbox','1']]);
  const control=page.locator('#'+id);
  const values=()=>control.evaluate(node=>new FormData(node.closest('form')).getAll(node.name));
  // Restore unchecked state after proving native duplicate-name submission ordering.
  assert.deepEqual(await values(),['0']);
  await control.check(); assert.deepEqual(await values(),['0','1']);
  await control.uncheck(); assert.deepEqual(await values(),['0']);
};
