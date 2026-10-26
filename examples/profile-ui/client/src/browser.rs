//! Local transport ownership and strict initial-seed admission.

use crate::seed::ClientSeed;
use leptos::prelude::*;
use stackpress_example_profile_ui_contracts::{
    ReplyDecodeError, SearchFailure, SearchInput, SearchReply, TransportFailure,
};
use stackpress_example_profile_ui_shared::{
    Navigation, ProfilePage, SearchCancellation, SearchCompletion, SearchTransport,
};
use wasm_bindgen::{JsCast, prelude::*};
#[wasm_bindgen(module = "/src/navigation.js")]
extern "C" {
    /// Install DOM listeners; returned disposer removes both listeners.
    #[wasm_bindgen(js_name = installNavigation)]
    fn install_navigation(receive: &js_sys::Function) -> js_sys::Function;
    /// Publish only Rust-checked canonical locations.
    #[wasm_bindgen(js_name = writeNavigation)]
    fn write_navigation(location: &str, push: bool);
}

/// Owned DOM listener lease; root disposal removes listeners before releasing callbacks.
struct NavigationLease {
    stop: js_sys::Function,
    _receive: Closure<dyn FnMut(String, bool)>,
}
impl Drop for NavigationLease {
    fn drop(&mut self) {
        let _ = self.stop.call0(&JsValue::NULL);
    }
}

/// Host acquisition adapter only; parsing and history decisions remain shared Rust.
fn navigation() -> Navigation {
    let request = RwSignal::new(None);
    let receive = Closure::wrap(Box::new(move |location, push| {
        request.try_set(Some((location, push)));
    }) as Box<dyn FnMut(String, bool)>);
    let stop = install_navigation(receive.as_ref().unchecked_ref());
    StoredValue::new_local(NavigationLease {
        stop,
        _receive: receive,
    });
    Navigation {
        request,
        write: UnsyncCallback::new(move |(location, push): (String, bool)| {
            write_navigation(&location, push)
        }),
    }
}
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::JsFuture;

#[cfg(feature = "csr")]
#[wasm_bindgen(module = "/src/ipc.js")]
extern "C" {
    /// Fixed command bridge; global invoke does not confer native permissions.
    #[wasm_bindgen(catch, js_name = profileSearch)]
    async fn profile_search(id: &str, json: &str) -> Result<JsValue, JsValue>;
    /// Request cancellation for this window's active ID.
    #[wasm_bindgen(js_name = cancelProfileSearch)]
    fn cancel_profile_search(id: &str);
}

/// One root-owned cancellation handle; it never holds an Owner or host service.
struct ActiveCall {
    /// Window-scoped public request identifier.
    #[cfg(feature = "csr")]
    id: String,
    /// Abort stops browser acquisition when the root is disposed or superseded.
    abort: web_sys::AbortController,
}

impl Drop for ActiveCall {
    /// Cancellation is best-effort transport disposal, never a rollback promise.
    fn drop(&mut self) {
        self.abort.abort();
        #[cfg(feature = "csr")]
        cancel_profile_search(&self.id);
    }
}

/// Create an Owner-bound local transport, with no Send wrapper around DOM values.
fn transport(endpoint: String) -> (SearchTransport, SearchCancellation) {
    let active = StoredValue::new_local(None::<ActiveCall>);
    let serial = StoredValue::new(0_u64);
    // StoredValue disposal drops the active call; async work cannot retain this root.
    let cancel = UnsyncCallback::new(move |()| {
        active.update_value(|call| *call = None);
    });
    let search = UnsyncCallback::new(move |(input, done): (SearchInput, SearchCompletion)| {
        active.update_value(|call| *call = None);
        serial.update_value(|value| *value = value.saturating_add(1));
        let id = format!("profile-{}", serial.get_value());
        let controller = match web_sys::AbortController::new() {
            Ok(controller) => controller,
            Err(_) => {
                done(Err(SearchFailure::Transport(TransportFailure::Start)));
                return;
            }
        };
        let signal = controller.signal();
        active.set_value(Some(ActiveCall {
            #[cfg(feature = "csr")]
            id: id.clone(),
            abort: controller,
        }));
        let endpoint = endpoint.clone();
        leptos::task::spawn_local(async move {
            // The completion checks disposed/generation state before any signal access.
            done(search(input, &id, signal, &endpoint).await);
        });
    });
    (search, cancel)
}

/// Execute the explicitly compiled transport and validate its bounded public reply.
async fn search(
    input: SearchInput,
    id: &str,
    signal: web_sys::AbortSignal,
    endpoint: &str,
) -> Result<SearchReply, SearchFailure> {
    let json = serde_json::to_string(&input)
        .map_err(|_| SearchFailure::Protocol(ReplyDecodeError::Invalid))?;
    #[cfg(feature = "hydrate")]
    let (text, status) = {
        let _ = id;
        let options = web_sys::RequestInit::new();
        options.set_method("POST");
        options.set_body(&JsValue::from_str(&json));
        options.set_signal(Some(&signal));
        options.set_mode(web_sys::RequestMode::SameOrigin);
        let request = web_sys::Request::new_with_str_and_init(endpoint, &options)
            .map_err(|_| SearchFailure::Transport(TransportFailure::Start))?;
        request
            .headers()
            .set("content-type", "application/json")
            .map_err(|_| SearchFailure::Transport(TransportFailure::Start))?;
        let window =
            web_sys::window().ok_or(SearchFailure::Transport(TransportFailure::Unavailable))?;
        let response: web_sys::Response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_| SearchFailure::Transport(TransportFailure::Unavailable))?
            .dyn_into()
            .map_err(|_| SearchFailure::Protocol(ReplyDecodeError::Invalid))?;
        let status = response.status();
        let text = JsFuture::from(
            response
                .text()
                .map_err(|_| SearchFailure::Protocol(ReplyDecodeError::Invalid))?,
        )
        .await
        .map_err(|_| SearchFailure::Protocol(ReplyDecodeError::Invalid))?
        .as_string()
        .ok_or(SearchFailure::Protocol(ReplyDecodeError::Invalid))?;
        (text, Some(status))
    };
    #[cfg(feature = "csr")]
    let (text, status) = {
        let _ = (signal, endpoint);
        let text = profile_search(id, &json)
            .await
            .map_err(|_| SearchFailure::Transport(TransportFailure::CancelledOrDenied))?
            .as_string()
            .ok_or(SearchFailure::Protocol(ReplyDecodeError::Invalid))?;
        (text, None)
    };
    crate::seed::reply(&text, status)
}

/// Acquire DOM text only; all wire/settings decisions are tested in the seed module.
fn seed(document: &web_sys::Document) -> Result<ClientSeed, String> {
    #[cfg(feature = "hydrate")]
    {
        let text = document
            .get_element_by_id("stackpress-page")
            .and_then(|element| element.text_content())
            .ok_or("Page data missing. Refresh to continue.")?;
        let assets = document
            .get_element_by_id("stackpress-assets")
            .and_then(|element| element.text_content())
            .ok_or("Page assets missing. Refresh to continue.")?;
        crate::seed::decode(&text, &assets)
    }
    #[cfg(feature = "csr")]
    {
        let _ = document;
        crate::seed::packaged()
    }
}

/// Mount/hydrate one root and return a JS disposer that cancels active work.
#[wasm_bindgen]
pub fn start() -> Result<js_sys::Function, JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("Document unavailable"))?;
    let app: web_sys::HtmlElement = document
        .get_element_by_id("app")
        .ok_or_else(|| JsValue::from_str("Page root missing"))?
        .dyn_into()?;
    let initial = match seed(&document) {
        Ok(seed) => seed,
        Err(message) => {
            app.set_text_content(Some(&message));
            return Err(JsValue::from_str(&message));
        }
    };
    let endpoint = stackpress_example_profile_ui_contracts::routes::search_api(&initial.base)
        .map_err(|_| JsValue::from_str("Page location invalid. Refresh to continue."))?
        .as_str()
        .to_owned();
    let root = move || {
        stackpress_framework_ui::leptos_meta::provide_meta_context();
        stackpress_framework_ui::routing::provide_route_context(initial.base.clone());
        let (transport, cancel) = transport(endpoint);
        let navigation = navigation();
        view! { <ProfilePage seed=initial.page page=initial.page_id status=initial.status transport cancel navigation
        initial_language=initial.language initial_theme=initial.theme
        initial_search=cfg!(feature = "csr")/> }
    };
    #[cfg(feature = "hydrate")]
    let handle = leptos::mount::hydrate_from(app, root);
    #[cfg(feature = "csr")]
    let handle = leptos::mount::mount_to(app, root);
    // Explicit ownership permits finite tests and navigation to clean up the root.
    let mut handle = Some(handle);
    let dispose = Closure::wrap(Box::new(move || {
        handle.take();
    }) as Box<dyn FnMut()>);
    Ok(dispose.into_js_value().unchecked_into())
}
