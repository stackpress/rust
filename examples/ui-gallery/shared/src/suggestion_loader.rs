//! Explicit example-only yielding suggestion acquisition; no hidden endpoint or initial fetch.
use stackpress_framework_ui::components::{ControlError, SelectOption, SelectOptions};

/// The same fixed owned option values are delivered after an explicit user query.
pub async fn load(query: String) -> Result<SelectOptions, ControlError> {
    #[cfg(target_arch = "wasm32")]
    wait(if query == "slow" { 250 } else { 25 }).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::yield_now().await;
    if query == "bad" {
        return Err(ControlError);
    }
    let values = if query == "slow" {
        vec![("old", "Slow old result")]
    } else {
        vec![("ada", "Ada Lovelace"), ("grace", "Grace Hopper")]
    };
    SelectOptions::new(
        values
            .into_iter()
            .map(|(value, label)| SelectOption {
                value: value.into(),
                label: label.into(),
                disabled: false,
            })
            .collect(),
    )
}
/// Dropping the loader clears its finite native timer and releases its callback.
#[cfg(target_arch = "wasm32")]
async fn wait(milliseconds: i32) {
    use wasm_bindgen::{JsCast, closure::Closure};
    struct Timer {
        i: i32,
        _callback: Closure<dyn FnMut()>,
    }
    impl Drop for Timer {
        fn drop(&mut self) {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(self.i);
            }
        }
    }
    let (sender, receiver) = futures::channel::oneshot::channel();
    let mut sender = Some(sender);
    let callback = Closure::wrap(Box::new(move || {
        if let Some(sender) = sender.take() {
            let _ = sender.send(());
        }
    }) as Box<dyn FnMut()>);
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(i) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        milliseconds,
    ) else {
        return;
    };
    let _timer = Timer {
        i,
        _callback: callback,
    };
    let _ = receiver.await;
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    /// Fixed example acquisition yields, returns checked options and exposes safe failure.
    #[test]
    fn explicit_acquisition_results() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(async {
            assert_eq!(load("Grace".into()).await.unwrap().entries().len(), 2);
            assert_eq!(load("slow".into()).await.unwrap().entries()[0].value, "old");
            assert!(load("bad".into()).await.is_err());
        });
    }
}
