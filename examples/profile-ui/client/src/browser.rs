//! Local transport ownership and strict initial-seed admission.

use leptos::prelude::*;
use stackpress_example_profile_ui_contracts::{REVISION, SearchInput, SearchPage, SearchReply};
use stackpress_example_profile_ui_shared::{ProfileSearch, SearchCompletion, SearchTransport};
use stackpress_framework_view::MAX_SNAPSHOT_BYTES;
use wasm_bindgen::{JsCast, prelude::*};
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
fn transport(base: String) -> SearchTransport {
    let active = StoredValue::new_local(None::<ActiveCall>);
    let serial = StoredValue::new(0_u64);
    // StoredValue disposal drops the active call; async work cannot retain this root.
    UnsyncCallback::new(move |(input, done): (SearchInput, SearchCompletion)| {
        active.update_value(|call| *call = None);
        serial.update_value(|value| *value = value.saturating_add(1));
        let id = format!("profile-{}", serial.get_value());
        let controller = match web_sys::AbortController::new() {
            Ok(controller) => controller,
            Err(_) => {
                done(Err("Search could not start".into()));
                return;
            }
        };
        let signal = controller.signal();
        active.set_value(Some(ActiveCall {
            #[cfg(feature = "csr")]
            id: id.clone(),
            abort: controller,
        }));
        let base = base.clone();
        leptos::task::spawn_local(async move {
            // The completion checks disposed/generation state before any signal access.
            done(search(input, &id, signal, &base).await);
        });
    })
}

/// Execute the explicitly compiled transport and validate its bounded public reply.
async fn search(
    input: SearchInput,
    id: &str,
    signal: web_sys::AbortSignal,
    base: &str,
) -> Result<SearchReply, String> {
    let json = serde_json::to_string(&input).map_err(|_| "Invalid search input")?;
    #[cfg(feature = "hydrate")]
    let text = {
        let _ = id;
        let options = web_sys::RequestInit::new();
        options.set_method("POST");
        options.set_body(&JsValue::from_str(&json));
        options.set_signal(Some(&signal));
        options.set_mode(web_sys::RequestMode::SameOrigin);
        let request = web_sys::Request::new_with_str_and_init(
            &format!("{base}/ui-api/v1/profile-search"),
            &options,
        )
        .map_err(|_| "Search could not start")?;
        request
            .headers()
            .set("content-type", "application/json")
            .map_err(|_| "Search could not start")?;
        let window = web_sys::window().ok_or("Window unavailable")?;
        let response: web_sys::Response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_| "Search cancelled or unavailable")?
            .dyn_into()
            .map_err(|_| "Invalid search response")?;
        if !response.ok() {
            return Err("Search unavailable".into());
        }
        JsFuture::from(response.text().map_err(|_| "Invalid search response")?)
            .await
            .map_err(|_| "Invalid search response")?
            .as_string()
            .ok_or("Invalid search response")?
    };
    #[cfg(feature = "csr")]
    let text = {
        let _ = (signal, base);
        profile_search(id, &json)
            .await
            .map_err(|_| "Search cancelled or denied")?
            .as_string()
            .ok_or("Invalid search response")?
    };
    if text.len() > MAX_SNAPSHOT_BYTES {
        return Err("Search reply is too large".into());
    }
    let reply: SearchReply = serde_json::from_str(&text).map_err(|_| "Invalid search response")?;
    reply.validate().map_err(str::to_owned)?;
    Ok(reply)
}

/// Decode textContent before creating reactive state or trusting rendered markup.
fn seed(document: &web_sys::Document) -> Result<SearchPage, String> {
    #[cfg(feature = "hydrate")]
    {
        use stackpress_example_profile_ui_contracts::BUILD;
        use stackpress_framework_view::PageSnapshot;
        let text = document
            .get_element_by_id("stackpress-page")
            .and_then(|element| element.text_content())
            .ok_or("Page data missing. Refresh to continue.")?;
        let snapshot = PageSnapshot::<SearchPage>::decode(&text, REVISION, BUILD)
            .map_err(|error| error.to_string())?;
        snapshot.data.input.validate().map_err(str::to_owned)?;
        snapshot.data.reply.validate().map_err(str::to_owned)?;
        Ok(snapshot.data)
    }
    #[cfg(feature = "csr")]
    {
        let _ = document;
        // No duplicated domain rows: packaged CSR obtains them through permitted IPC.
        Ok(SearchPage {
            input: SearchInput::new("abc".into()).map_err(str::to_owned)?,
            reply: SearchReply {
                revision: REVISION,
                rows: Vec::new(),
                total: 0,
            },
        })
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
    #[cfg(feature = "hydrate")]
    let base = document
        .get_element_by_id("stackpress-assets")
        .and_then(|element| element.text_content())
        .ok_or_else(|| "Page assets missing. Refresh to continue.".to_owned())
        .and_then(|text| {
            stackpress_framework_view::PageAssets::decode(
                &text,
                stackpress_example_profile_ui_contracts::BUILD,
            )
            .map_err(|error| error.to_string())
        })
        .map(|assets| assets.base);
    #[cfg(feature = "csr")]
    let base: Result<stackpress_framework_view::BasePath, String> = Ok(Default::default());
    let (initial, base) = match seed(&document).and_then(|seed| base.map(|base| (seed, base))) {
        Ok(seed) => seed,
        Err(message) => {
            app.set_text_content(Some(&message));
            return Err(JsValue::from_str(&message));
        }
    };
    let root = move || {
        stackpress_framework_ui::leptos_meta::provide_meta_context();
        stackpress_framework_ui::routing::provide_route_context(base.clone());
        let transport = transport(base.as_str().to_owned());
        view! { <ProfileSearch seed=initial transport/> }
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
