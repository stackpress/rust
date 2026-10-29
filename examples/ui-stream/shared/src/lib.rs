//! One real component source for native in-order SSR and browser hydration.
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use stackpress_leptos::resources::cancel_on_cleanup;

/// Only deliberately public rendering choices enter the seed.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemoData {
    /// Delayed section's bounded demo latency.
    pub delay_ms: u64,
    /// Trigger a trusted render failure after the Resource settles.
    pub fail: bool,
    /// Keep the demo Resource pending until its body is cancelled.
    pub pending: bool,
}
/// Exact hostile text must survive both HTML escaping and Resource replay.
pub const HOSTILE: &str =
    "Resolved: </script><script>window.hostile=true</script>&\u{2028}\u{2029}";

/// Useful static content precedes the delayed Resource/Suspense section.
#[component]
pub fn StreamDemo(
    /// Checked public seed for this page.
    data: DemoData,
) -> impl IntoView {
    let fail = data.fail;
    let retained = use_context::<std::sync::Arc<()>>();
    let delayed = Resource::new(
        || (),
        move |()| {
            let retained = retained.clone();
            let data = data.clone();
            cancel_on_cleanup(async move {
                let _retained = retained;
                load(data).await
            })
        },
    );
    let (count, set_count) = signal(0);
    view! {
        <h1>"Streamed page"</h1>
        <p id="shell">"You can read this while the details load."</p>
        <button id="counter" on:click=move |_| set_count.update(|value| *value += 1)>"Clicks: "{count}</button>
        <Suspense fallback=||view! { <p>"Waiting for details…"</p> }>
            {move || delayed.get().map(|value| {
                if fail { panic!("private demo late-render sentinel"); }
                view! { <section aria-label="Details"><h2>"Details ready"</h2><p id="details">"The project summary is ready. Review the latest activity with your team."</p>
                    <section aria-labelledby="stream-safety-title"><h2 id="stream-safety-title">"Safety samples"</h2><p>"Deliberately hostile text is displayed as data to test escaping and replay."</p><p id="stream-safety-value">{value.unwrap_or_else(|_| "Safety sample unavailable".into())}</p></section></section> }
            })}
        </Suspense>
    }
}

/// Server acquisition runs once and remains owned by the render runtime.
#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
async fn load(data: DemoData) -> String {
    if data.pending {
        std::future::pending::<()>().await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(data.delay_ms.min(5000))).await;
    HOSTILE.to_owned()
}
/// Hydration should never invoke this endpoint; the browser proof counts it.
#[cfg(target_arch = "wasm32")]
async fn load(_: DemoData) -> String {
    // Keep DOM futures local; Resource itself awaits a Send oneshot receiver.
    // A replay miss is intentionally conspicuous in the browser network proof.
    let (send, receive) = futures::channel::oneshot::channel();
    leptos::task::spawn_local_scoped_with_cancellation(async move {
        use wasm_bindgen::JsCast;
        let text = async {
            let response = wasm_bindgen_futures::JsFuture::from(
                web_sys::window()?.fetch_with_str("/resource"),
            )
            .await
            .ok()?;
            let response: web_sys::Response = response.dyn_into().ok()?;
            wasm_bindgen_futures::JsFuture::from(response.text().ok()?)
                .await
                .ok()?
                .as_string()
        }
        .await
        .unwrap_or_else(|| "Resource unavailable".to_owned());
        let _ = send.send(text);
    });
    receive
        .await
        .unwrap_or_else(|_| "Resource cancelled".to_owned())
}
/// Feature-free workspace checks have no asynchronous host to acquire from.
#[cfg(all(not(feature = "ssr"), not(target_arch = "wasm32")))]
async fn load(_: DemoData) -> String {
    HOSTILE.to_owned()
}
