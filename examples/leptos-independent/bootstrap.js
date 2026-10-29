// A consumer-owned bootstrap: explicit choices, inert generic replay, native draft adoption.
import init, { start } from './independent.js';
const config = JSON.parse(document.getElementById('independent-config').textContent);
const replay = document.getElementById('independent-replay').textContent;
await init();
window.independentMount = (theme = config.theme, language = config.language, values = replay) => {
  const draft = document.getElementById('independent-draft')?.value ?? 'Seeded draft';
  return start(theme, language, draft, values);
};
window.independentDispose = window.independentMount();
window.independentReady = true;
