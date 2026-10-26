//! Managed-preview executable only; tests exercise the same runner over duplex connections.

use stackpress_example_profile_ui_gateway::{
    Gateway, GatewayLimits, LaunchOptions, assets::AssetBundle,
};
use std::sync::Arc;

/// Validate startup, bind only the explicit loopback port, then delegate owned task shutdown.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let options = LaunchOptions::parse(
        &std::env::args().skip(1).collect::<Vec<_>>(),
        std::env::var("PORT").ok().as_deref(),
    )?;
    let assets = AssetBundle::load(&options.assets)?;
    let gateway = Arc::new(Gateway::new(options.base, assets, GatewayLimits::default()).await?);
    // This is the only bind in the example. No check/proof invokes this executable.
    let listener =
        tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, options.port)).await?;
    eprintln!("Profile UI preview: http://127.0.0.1:{}", options.port);
    let stop = async {
        tokio::select! {
            signal = shutdown_signal() => signal,
            _ = tokio::time::sleep(options.duration) => Ok(()),
        }
    };
    gateway
        .run(
            || async { listener.accept().await.map(|(io, _)| Some(io)) },
            stop,
        )
        .await?;
    Ok(())
}

/// Managed preview termination drains HTTP work on both Ctrl-C and Unix SIGTERM.
async fn shutdown_signal() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await
    }
}
