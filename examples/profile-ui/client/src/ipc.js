// Fixed wrappers: native ACL and trusted window identity own admission.
export async function profileSearch(callId, json) {
  return window.__TAURI__.core.invoke('profile_search', { callId, json });
}
export function cancelProfileSearch(callId) {
  void window.__TAURI__.core.invoke('cancel_profile_search', { callId }).catch(() => {});
}
