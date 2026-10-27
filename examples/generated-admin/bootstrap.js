import init, { start } from './admin.js';
await init();
let active;
const seed = document.getElementById('admin-seed');
const page = seed ? JSON.parse(seed.textContent) : undefined;
const endpoints = new Set(page ? ['search','get','detail','create','update','remove','restore'].map(action=>`${page.mount}api/${page.spec.module}/${action}`) : []);
const request = async event => {
  if (active) return;
  const command = JSON.parse(event.detail);
  const endpoint = new URL(command.endpoint, location.href);
  if (endpoint.origin !== location.origin || !endpoints.has(endpoint.pathname)) throw Error('Invalid compiled admin endpoint');
  const controller = new AbortController(); active = controller;
  let reply;
  try {
    const response = await fetch(endpoint, { method:'POST', credentials:'same-origin', headers:{'content-type':'application/json','x-admin-csrf':command.csrf}, body:JSON.stringify(command.input), signal:controller.signal });
    reply = { status:response.status, envelope:await response.json() };
  } catch {
    reply = { status:500, envelope:{code:500,error:command.write?'Write acknowledgement unavailable':'Unable to load records',errors:command.write?{outcome:'unknown'}:{}} };
  } finally { if (active === controller) active = undefined; }
  if (!controller.signal.aborted) window.dispatchEvent(new CustomEvent('stackpress-admin-reply',{detail:JSON.stringify(reply)}));
};
window.addEventListener('stackpress-admin-request',request);
const dispose = start();
window.adminDispose = () => { window.removeEventListener('stackpress-admin-request',request); active?.abort(); active=undefined; dispose(); };
window.adminReady = true;
