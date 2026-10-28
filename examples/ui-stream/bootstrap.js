// This external module runs after parsing. Incomplete pages keep their safe
// recovery notice and never hydrate partial markup or retry an operation.
import init, { start } from './stream.js';
const recovery = document.getElementById('stackpress-stream-recovery');
try {
  if (!document.getElementById('stackpress-stream-complete') || document.getElementById('stackpress-stream-failed')) {
    recovery.setAttribute('role', 'alert');
    // Parsing ended without an admitted completion marker; keep the same
    // definitive wording as the independent early status asset.
    recovery.firstChild.textContent = "This page didn't finish loading. ";
  } else {
    const templates = [...document.querySelectorAll('template[data-stackpress-replay]')];
    // WASM validates the aggregate count, identities, duplicates and byte budget.
    const replay = templates.flatMap(template => JSON.parse(template.content.textContent));
    const seed = document.getElementById('stackpress-page').textContent;
    await init();
    templates.forEach(template => template.remove());
    window.streamBeforeHydrate = document.getElementById('app').innerHTML;
    window.streamInitialButton = document.getElementById('counter');
    window.streamDispose = start(seed, JSON.stringify(replay));
    recovery.hidden = true;
    document.documentElement.dataset.hydrated = 'true';
  }
} catch {
  recovery.setAttribute('role', 'alert');
  recovery.firstChild.textContent = 'The page could not start. You can ';
}
