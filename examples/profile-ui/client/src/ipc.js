// Fixed wrappers: native ACL and trusted window identity own admission.
/** Forward one fixed public DTO; native code checks identity and admission. */
export async function profileSearch(callId, json) {
  return window.__TAURI__.core.invoke('profile_search', { callId, json });
}
/** Best-effort revocation for this window's ID; late completion guards remain in Rust. */
export function cancelProfileSearch(callId) {
  void window.__TAURI__.core.invoke('cancel_profile_search', { callId }).catch(() => {});
}
