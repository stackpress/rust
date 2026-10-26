import init, {start} from './preferences.js';
await init();
window.disposePreferences = start(document.getElementById('preferences-seed').textContent);
window.preferencesReady = true;
