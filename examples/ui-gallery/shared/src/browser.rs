//! Thin browser fetch/history adapter; all locale authority remains host-side.
use futures::channel::oneshot;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Own the actual HTTP acquisition until its scoped future completes or is cancelled.
struct AbortOnDrop(web_sys::AbortController);
impl Drop for AbortOnDrop {
    /// Dropping a JsFuture alone does not abort browser networking.
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Acquire bounded text from one fixed same-origin endpoint, with scoped cancellation.
async fn fetch(path: String, body: Option<String>) -> Result<String, ()> {
    let controller = web_sys::AbortController::new().map_err(|_| ())?;
    let guard = AbortOnDrop(controller);
    let options = web_sys::RequestInit::new();
    options.set_signal(Some(&guard.0.signal()));
    if let Some(body) = body {
        options.set_method("POST");
        options.set_body(&JsValue::from_str(&body));
        let headers = web_sys::Headers::new().map_err(|_| ())?;
        headers
            .set("Content-Type", "application/json")
            .map_err(|_| ())?;
        options.set_headers(&headers);
    }
    let window = web_sys::window().ok_or(())?;
    let response = JsFuture::from(window.fetch_with_str_and_init(&path, &options))
        .await
        .map_err(|_| ())?;
    let response: web_sys::Response = response.dyn_into().map_err(|_| ())?;
    if !response.ok() {
        return Err(());
    }
    let text = JsFuture::from(response.text().map_err(|_| ())?)
        .await
        .map_err(|_| ())?
        .as_string()
        .ok_or(())?;
    if text.len() > 16 * 1024 {
        return Err(());
    }
    Ok(text)
}

/// Return a Send receiver to Resource; DOM values stay in a local scoped future.
pub(super) fn resource(base: String) -> oneshot::Receiver<String> {
    let (send, receive) = oneshot::channel();
    leptos::task::spawn_local_scoped_with_cancellation(async move {
        let result = fetch(format!("{base}/resource"), None)
            .await
            .ok()
            .and_then(|text| serde_json::from_str::<String>(&text).ok())
            .unwrap_or_else(|| "Resource unavailable".to_owned());
        let _ = send.send(result);
    });
    receive
}

/// Persist through the fixed action; publication occurs only after acknowledgement.
pub(super) fn preference(
    base: String,
    locale: String,
    location: String,
    complete: impl FnOnce(Result<(String, String), ()>) + 'static,
) {
    leptos::task::spawn_local_scoped_with_cancellation(async move {
        let result = async {
            let update =
                stackpress_framework_view::preferences::PreferenceUpdate { locale, location };
            let body = serde_json::to_string(&update).map_err(|_| ())?;
            let text = fetch(format!("{base}/ui-preferences"), Some(body)).await?;
            let reply = super::decode_preference(&text, &base).map_err(|_| ())?;
            // Acknowledged path is same-app and checked before history publication.
            web_sys::window()
                .ok_or(())?
                .history()
                .map_err(|_| ())?
                .replace_state_with_url(&JsValue::NULL, "", Some(&reply.location))
                .map_err(|_| ())?;
            Ok((reply.locale, reply.location))
        }
        .await;
        complete(result);
    });
}
