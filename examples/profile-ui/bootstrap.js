// External entry owns startup recovery; the shared Rust client remains unchanged.
try {
  // Dynamic import also makes a missing JS chunk a recoverable startup failure.
  const { default: init, start } = await import('./profile_ui.js');
  await init();
  window.disposeProfileUi = start();
  window.profileUiReady = true;
} catch {
  // Retain useful server HTML and require an explicit action, with no reload loop.
  window.profileUiReady = false;
  if (!document.getElementById('stackpress-recovery')) {
    const notice = document.createElement('div');
    notice.id = 'stackpress-recovery';
    notice.setAttribute('role', 'alert');
    notice.style.cssText = 'font:1rem/1.5 system-ui,sans-serif;color:#1a1a1a;background:#f5f5f5;padding:1rem 2rem;border-bottom:1px solid #767676';
    const message = document.createElement('p');
    message.textContent = 'This page could not start. Reload to continue.';
    const reload = document.createElement('button');
    reload.type = 'button';
    reload.textContent = 'Reload page';
    reload.style.cssText = 'font:inherit;padding:.5rem 1rem;border:1px solid currentColor;border-radius:.25rem;background:white;color:inherit';
    reload.addEventListener('click', () => window.location.reload());
    notice.append(message, reload);
    // Put recovery before the full-height page and give keyboard users the next action.
    document.body.prepend(notice);
    reload.focus();
  }
}
