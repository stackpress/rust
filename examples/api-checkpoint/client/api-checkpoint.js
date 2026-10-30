// The app owns asset location and optional WASM startup. Consent forms work with no script.
import init, { start } from './pkg/api_checkpoint_client.js';
await init();
window.disposeApiCheckpoint = start();
// File transport only supplies the same CSV textarea/draft; the selected model owns batch writes.
document.addEventListener('change', async event => {
  const file = event.target.matches('[data-api-csv]') && event.target.files?.[0];
  if (!file) return;
  const text = await file.text();
  const textarea = document.getElementById('api-csv-text');
  if (textarea) { textarea.value = text; textarea.dispatchEvent(new Event('input', { bubbles: true })); }
});
