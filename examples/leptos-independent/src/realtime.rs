//! Explicit helper consumer used by separate SSR, hydrate and CSR browser proofs.
use leptos::prelude::*;
use stackpress_leptos::realtime::*;

/// One component owns both connections; identical initial state makes hydration deterministic.
#[component]
pub fn RealtimeRoot() -> impl IntoView {
    let sse = use_event_source(
        "/events".into(),
        EventSourceOptions::default()
            .with_events(&["ready"])
            .unwrap(),
    )
    .unwrap();
    let ws = use_websocket(
        "/ws".into(),
        BrowserWebSocketOptions::default()
            .with_limits(65536, 262144)
            .unwrap()
            .with_reconnect(2)
            .unwrap(),
    )
    .unwrap();
    let result = RwSignal::new(String::new());
    view! {
        <section id="realtime-ui">
            <output id="sse-state">{move || format!("{:?}",sse.state().get())}</output>
            <output id="sse-message">{move || sse.latest().get().map(|m|format!("{}|{}|{}",m.event,m.last_event_id,m.data)).unwrap_or_default()}</output>
            <output id="ws-state">{move || format!("{:?}",ws.state().get())}</output>
            <output id="ws-message">{move || format!("{:?}",ws.latest().get())}</output>
            <output id="result">{move || result.get()}</output>
            <button id="send-text" on:click=move |_| {result.set(format!("{:?}",ws.send(BrowserWsMessage::Text("hello".into()))));}>"Text"</button>
            <button id="send-binary" on:click=move |_| {result.set(format!("{:?}",ws.send(BrowserWsMessage::Binary(vec![0,255]))));}>"Binary"</button>
            <button id="close-ws" on:click=move |_| {result.set(format!("{:?}",ws.close(1000,"done")));}>"Close"</button>
            <button id="reconnect" on:click=move |_| {result.set(format!("{:?}",ws.reconnect()));}>"Reconnect"</button>
            <button id="close-sse" on:click=move |_| {result.set(format!("{:?}",sse.close()));}>"Stop events"</button>
        </section>
    }
}

/// Mount/hydrate one real root and return its explicit disposer to the JS proof.
#[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn realtime_start() -> Result<js_sys::Function, wasm_bindgen::JsValue> {
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    let element = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("realtime-root"))
        .ok_or_else(|| JsValue::from_str("missing realtime root"))?
        .dyn_into::<web_sys::HtmlElement>()?;
    #[cfg(feature = "hydrate")]
    let handle = leptos::mount::hydrate_from(element, || view! {<RealtimeRoot/>});
    #[cfg(feature = "csr")]
    let handle = leptos::mount::mount_to(element, || view! {<RealtimeRoot/>});
    let mut handle = Some(handle);
    let dispose = Closure::wrap(Box::new(move || {
        handle.take();
    }) as Box<dyn FnMut()>);
    Ok(dispose.into_js_value().unchecked_into())
}

/// Fresh owner/URL pair for browser lifecycle proofs after the rendered root is disposed.
#[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn realtime_clients(suffix: String) -> Result<js_sys::Function, wasm_bindgen::JsValue> {
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    let owner = Owner::new();
    owner.with(|| {
        use_event_source(format!("/events?{suffix}"), Default::default())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        use_websocket(format!("/ws?{suffix}"), Default::default())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>(())
    })?;
    let dispose = Closure::wrap(Box::new(move || owner.cleanup()) as Box<dyn FnMut()>);
    Ok(dispose.into_js_value().unchecked_into())
}
