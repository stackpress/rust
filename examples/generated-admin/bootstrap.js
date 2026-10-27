import init, { start } from './admin.js';
await init();
window.adminDispose = start();
window.adminReady = true;
