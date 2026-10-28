// STACKPRESS_INSTALLED_REVIEW: packaged only by the explicit debug review build.
// These fixed checks reuse existing Profile commands; they confer no permission.
const panel = document.createElement('section');
panel.setAttribute('aria-label', 'Installed review checks');
panel.style.cssText = 'padding:1rem;font:16px system-ui';
const heading = document.createElement('h2');
heading.textContent = 'Installed review checks — 011 / 013';
const catalog = document.createElement('a');
catalog.href = '/admin-review.html';
catalog.textContent = 'Generated admin review (local role catalog only)';
const status = document.createElement('p');
status.id = 'installed-review-status'; status.setAttribute('role', 'status');
status.textContent = 'Ready. Review fixtures are not authentication.';
const deny = document.createElement('button'); deny.textContent = 'Check fixed denial';
const start = document.createElement('button'); start.textContent = 'Start cancellation check';
const cancel = document.createElement('button'); cancel.textContent = 'Cancel review read'; cancel.disabled = true;
let active, serial = 0;
const invoke = (command, args) => window.__TAURI__.core.invoke(command, args);
deny.addEventListener('click', async () => {
  deny.disabled = true;
  try {
    await invoke('profile_search', { callId: 'review-denial', json: '{"revision":1,"id":"abc","event":"shell"}' });
    status.textContent = 'Unexpected admission. Record a finding.';
  } catch (error) {
    status.textContent = error === 'denied'
      ? 'Denied: the fixed extra-field search request was refused.'
      : 'Denial acknowledgement unavailable; outcome uncertain.';
  } finally { deny.disabled = false; }
});
start.addEventListener('click', async () => {
  const id = `review-cancel-${++serial}`;
  active = id; start.disabled = true; cancel.disabled = false;
  status.textContent = 'Pending read (four-second review delay). Cancel now.';
  try {
    await invoke('profile_search', { callId: id, json: '{"revision":1,"id":"review-wait"}' });
    status.textContent = 'Read completed before cancellation. Start the check again.';
  } catch (error) {
    status.textContent = error === 'cancelled'
      ? 'Cancelled: native read returned cancelled. No result was published.'
      : 'Read was not confirmed as cancelled. Start the check again.';
  } finally { active = undefined; start.disabled = false; cancel.disabled = true; }
});
cancel.addEventListener('click', async () => {
  cancel.disabled = true;
  try { await invoke('cancel_profile_search', { callId: active }); }
  catch { status.textContent = 'Cancellation acknowledgement unavailable; outcome uncertain.'; }
});
panel.append(heading, catalog, document.createElement('br'), deny, start, cancel, status);
document.body.prepend(panel);
// Navigation disposes this document and the ordinary Profile root together.
