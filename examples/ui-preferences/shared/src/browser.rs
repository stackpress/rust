//! Thin fixed-endpoint browser transport; pure native-tested state owns publication.
use super::*;
use stackpress_framework_view::BasePath;
use stackpress_framework_view::preferences::SettingsUpdate;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Abort actual networking when the root or acquisition future is dropped.
struct AbortOnDrop(web_sys::AbortController);
impl Drop for AbortOnDrop {
    /// Browser fetch outlives a dropped JsFuture unless its controller is aborted.
    fn drop(&mut self) {
        self.0.abort();
    }
}
/// Acquire one finite acknowledgement from a fixed endpoint; no write retries.
async fn post(path: &str, body: &str) -> Result<String, ()> {
    // Keep the real fetch abortable through root disposal, including while JsFuture is pending.
    let guard = AbortOnDrop(web_sys::AbortController::new().map_err(|_| ())?);
    let signals = js_sys::Array::new();
    signals.push(&guard.0.signal());
    signals.push(&web_sys::AbortSignal::timeout_with_u32(10_000));
    // The adapter owns method/body/header shape; user preferences cannot choose a URL or verb.
    let options = web_sys::RequestInit::new();
    options.set_method("POST");
    // Root disposal and the action deadline both abort real networking.
    options.set_signal(Some(&web_sys::AbortSignal::any(&signals)));
    options.set_body(&JsValue::from_str(body));
    let headers = web_sys::Headers::new().map_err(|_| ())?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|_| ())?;
    options.set_headers(&headers);
    let window = web_sys::window().ok_or(())?;
    // Await only acquisition here; pure state admission still checks the complete reply later.
    let response = JsFuture::from(window.fetch_with_str_and_init(path, &options))
        .await
        .map_err(|_| ())?;
    let response: web_sys::Response = response.dyn_into().map_err(|_| ())?;
    // Failed HTTP actions do not publish settings or trigger an automatic retry.
    if !response.ok() {
        return Err(());
    }
    // Acknowledgements remain bounded public text, independent of inaccessible HttpOnly cookies.
    let text = JsFuture::from(response.text().map_err(|_| ())?)
        .await
        .map_err(|_| ())?
        .as_string()
        .ok_or(())?;
    if text.len() > stackpress_framework_view::preferences::MAX_PREFERENCE_BYTES {
        return Err(());
    }
    Ok(text)
}
/// Persist once and confirm cookies; preserve draft/content on any unconfirmed outcome.
pub(super) fn save(
    base: BasePath,
    update: SettingsUpdate,
    state: RwSignal<PreferenceState>,
    phrases: LanguageContext,
) {
    // Disposal drops acquisition and its abort lease; late work cannot retain this root.
    leptos::task::spawn_local_scoped_with_cancellation(async move {
        let result = async {
            // Freeze the admitted intent once and use the same body for write and readback.
            let body = serde_json::to_string(&update).map_err(|_| ())?;
            let policy = action(base.clone());
            let written = post(&format!("{}/ui-preferences", base.as_str()), &body).await?;
            state
                .get_untracked()
                .validate(&policy, &written)
                .map_err(|_| ())?;
            // Read back both HttpOnly cookies before publishing. A blocked cookie
            // or lost confirmation is failure; refresh may still reveal a saved write.
            let confirmed =
                post(&format!("{}/ui-preferences/confirm", base.as_str()), &body).await?;
            let reply = state
                .get_untracked()
                .validate(&policy, &confirmed)
                .map_err(|_| ())?;
            // Stage history before publishing reactive settings; failure leaves drafts/content intact.
            web_sys::window()
                .ok_or(())?
                .history()
                .map_err(|_| ())?
                .replace_state_with_url(&JsValue::NULL, "", Some(&reply.location))
                .map_err(|_| ())?;
            Ok((reply, policy, confirmed))
        }
        .await;
        // Publish dictionary/settings together only after both acknowledgements and history succeed.
        batch(move || match result {
            Ok((reply, policy, confirmed)) => {
                let mut published = false;
                state.update(|state| published = state.complete(&policy, &confirmed).is_ok());
                // A stale/disposed completion cannot change the root dictionary.
                if published {
                    phrases.change(language(&reply.preferences.locale));
                }
            }
            Err(()) => state.update(PreferenceState::fail),
        });
    });
}
