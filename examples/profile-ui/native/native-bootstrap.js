// Mobile-only bundled bootstrap: acquire host settings before creating any CSR root.
try {
  const { default: init, start_native } = await import('./profile_ui.js');
  await init();
  window.disposeProfileUi = await start_native();
  window.profileUiReady = true;
} catch {
  window.profileUiReady = false;
  const notice = document.createElement('p');
  notice.setAttribute('role', 'alert');
  notice.textContent = 'Saved preferences could not be loaded. Reopen the app to continue.';
  document.getElementById('app').replaceChildren(notice);
}
