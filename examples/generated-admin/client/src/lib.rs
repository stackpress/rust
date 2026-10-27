//! Browser-only mount of exactly the generated SSR component source; no host services.

#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use stackpress_framework_admin::pages::{Command, Reply, Snapshot};
    use wasm_bindgen::{JsCast, prelude::*};

    /// The exact event registration is removed before its callback is freed on root disposal.
    struct Receiver(Closure<dyn FnMut(web_sys::Event)>);
    impl Drop for Receiver {
        fn drop(&mut self) {
            if let Some(window) = web_sys::window() {
                let _ = window.remove_event_listener_with_callback(
                    "stackpress-admin-reply",
                    self.0.as_ref().unchecked_ref(),
                );
            }
        }
    }

    /// Root-owned platform transport seam; JavaScript owns fetch cancellation, not domain state.
    fn transport() -> Callback<Command> {
        let pending = StoredValue::new(None::<Callback<Reply>>);
        let receive = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let Some(raw) = event
                .dyn_ref::<web_sys::CustomEvent>()
                .and_then(|event| event.detail().as_string())
            else {
                return;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
                return;
            };
            let mut callback = None;
            pending.update_value(|pending| callback = pending.take());
            if let Some(callback) = callback {
                callback.run(Reply {
                    status: value
                        .get("status")
                        .and_then(|value| value.as_u64())
                        .and_then(|value| u16::try_from(value).ok())
                        .unwrap_or(500),
                    envelope: value.get("envelope").cloned().unwrap_or_default(),
                });
            }
        }) as Box<dyn FnMut(web_sys::Event)>);
        let window = web_sys::window().expect("browser window");
        window
            .add_event_listener_with_callback(
                "stackpress-admin-reply",
                receive.as_ref().unchecked_ref(),
            )
            .expect("fixed browser event listener");
        let _receiver = StoredValue::new_local(Receiver(receive));
        Callback::new(move |command: Command| {
            // Exactly one UI command may be in flight in this root; no mutation replay.
            if pending.with_value(|pending| pending.is_some()) {
                return;
            }
            pending.set_value(Some(command.complete));
            let raw=serde_json::json!({"endpoint":command.endpoint,"input":command.input,"csrf":command.csrf,"write":command.write}).to_string();
            let options = web_sys::CustomEventInit::new();
            options.set_detail(&JsValue::from_str(&raw));
            if let Ok(event) =
                web_sys::CustomEvent::new_with_event_init_dict("stackpress-admin-request", &options)
                && let Some(window) = web_sys::window()
            {
                let _ = window.dispatch_event(&event);
            }
        })
    }

    /// Hydrate or mount one caller-owned root, returning its explicit disposer.
    /// # Errors
    /// Reports a missing DOM root or platform API failure to the finite proof harness.
    #[wasm_bindgen]
    pub fn start() -> Result<js_sys::Function, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("missing document"))?;
        let app: web_sys::HtmlElement = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("missing root"))?
            .dyn_into()?;
        let seed = document
            .get_element_by_id("admin-seed")
            .map(|element| element.text_content().unwrap_or_default())
            .map(|json| {
                serde_json::from_str::<Snapshot>(&json)
                    .map_err(|_| JsValue::from_str("invalid public admin seed"))
            })
            .transpose()?;
        let root = move || match seed {
            Some(snapshot) => {
                let transport = transport();
                view! {<generated_admin_shared::crud::Admin snapshot transport/>}.into_any()
            }
            None => view! { <generated_admin_shared::Admin/> }.into_any(),
        };
        #[cfg(feature = "hydrate")]
        let handle = leptos::mount::hydrate_from(app, root);
        #[cfg(feature = "csr")]
        let handle = leptos::mount::mount_to(app, root);
        let mut handle = Some(handle);
        let dispose = Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>);
        Ok(dispose.into_js_value().unchecked_into())
    }
}
