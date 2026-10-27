// Fixed wrappers: native ACL and trusted window identity own admission.
/** Forward one fixed public DTO; native code checks identity and admission. */
export async function profileSearch(callId, json) {
  return window.__TAURI__.core.invoke('profile_search', { callId, json });
}
/** Best-effort revocation for this window's ID; late completion guards remain in Rust. */
export function cancelProfileSearch(callId) {
  void window.__TAURI__.core.invoke('cancel_profile_search', { callId }).catch(() => {});
}
/** A failed/late native write is unconfirmed, never automatically retried. */
function preferencesDeadline(promise) {
  let timer;
  return Promise.race([promise, new Promise((_, reject) => {
    timer = setTimeout(() => reject(Error('Preferences unavailable')), 10000);
  })]).finally(() => clearTimeout(timer));
}
/** Acquire before the first native CSR mount. */
export function loadPreferences() {
  return preferencesDeadline(window.__TAURI__.core.invoke('load_preferences'));
}
/** The only native write is this fixed versioned settings command. */
export function updatePreferences(json) {
  return preferencesDeadline(window.__TAURI__.core.invoke('update_preferences', { json }));
}
/** System colors are resolved before mounting, never inferred for legacy startup. */
export function systemDark() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}
