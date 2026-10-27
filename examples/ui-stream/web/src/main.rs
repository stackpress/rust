//! Finite chunk emitter for the intercepted browser transport; never listens.
use std::{
    io::{Read, Write},
    sync::atomic::Ordering,
};

/// A separate input thread is joined after the final report; browser finally
/// closes stdin on success, navigation, failure and watchdog termination.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "success".into());
    let base = std::env::args().nth(2).unwrap_or_default();
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let input = std::thread::spawn(move || {
        let mut byte = [0];
        let _ = std::io::stdin().read(&mut byte);
        let _ = cancel.send(());
    });
    let result = stackpress_example_ui_stream_web::exchange(
        &mode,
        &base,
        |chunk| {
            println!("{}", serde_json::json!({ "chunk": chunk }));
            std::io::stdout().flush().unwrap();
        },
        async {
            let _ = cancelled.await;
        },
    )
    .await;
    match result {
        Ok(exchange) => println!(
            "{}",
            serde_json::json!({ "done": true, "head": exchange.head, "hooks": exchange.probe.hooks.load(Ordering::SeqCst), "errors": exchange.probe.errors.load(Ordering::SeqCst), "cleanups": exchange.probe.cleanups.load(Ordering::SeqCst), "released": exchange.probe.released(), "delivery": format!("{:?}", exchange.report.exchanges.last().map(|report| report.delivery)) })
        ),
        Err(error) => println!(
            "{}",
            serde_json::json!({ "done": true, "error": error.to_string() })
        ),
    }
    std::io::stdout().flush()?;
    input
        .join()
        .map_err(|_| std::io::Error::other("browser input worker panicked"))?;
    Ok(())
}
