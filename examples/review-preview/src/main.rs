//! Explicit setup, port-free pipe proof, or devmetrics-only bounded loopback serving.
use stackpress_example_review_preview::{gateway::Gateway, options::Options, pipe, setup};
use stackpress_server_http::{AcceptedIo, Acceptor};
use std::{
    io::{BufRead, Read, Write},
    sync::Arc,
};

/// The executable alone owns TCP accepts; tests inject duplex and never construct this value.
struct Loopback(tokio::net::TcpListener);
impl Acceptor for Loopback {
    type Io = tokio::net::TcpStream;
    /// Pass only accepted loopback streams to the existing bounded host supervisor.
    async fn accept(&mut self) -> stackpress_lib::Result<Option<AcceptedIo<Self::Io>>> {
        let (io, peer) = self
            .0
            .accept()
            .await
            .map_err(stackpress_lib::Error::from_source)?;
        Ok(Some(AcceptedIo::plain(io, peer)))
    }
}
/// All executable modes require explicit inputs; only `serve` opens IO.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let action = args
        .first()
        .ok_or("Expected setup, serve, pipe or stream-pipe")?;
    if action == "setup" {
        if args.len() != 3 || args[1] != "--directory" {
            return Err("setup requires --directory DIR".into());
        }
        println!(
            "{}",
            setup::create(std::path::Path::new(&args[2]))
                .await?
                .display()
        );
        return Ok(());
    }
    if !["serve", "pipe", "stream-pipe"].contains(&action.as_str()) {
        return Err("Unknown action".into());
    }
    let options = Options::parse(&args[1..])?;
    let gateway = Arc::new(Gateway::new(&options).await?);
    if action == "stream-pipe" {
        // The parent owns an input/watchdog deadline, as in buffered pipe mode.
        // One bounded request emits actual native frames for a browser ReadableStream.
        // Normal/error completion is reported separately; missing HTTP framing stays failure.
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .take(3 * 1024 * 1024)
            .read_line(&mut line)?;
        if !line.ends_with('\n') {
            return Err("stream-pipe request exceeds bound".into());
        }
        let emit = |value| {
            println!("{value}");
            std::io::stdout().flush().expect("open proof output");
        };
        let output = pipe::exchange_stream(
            &gateway,
            &options.origin(),
            serde_json::from_str(&line)?,
            &emit,
        )
        .await?;
        emit(serde_json::json!({"done":true,"terminal":!output.interrupted}));
    } else if action == "pipe" {
        // Only one framed request is processed at a time; callers own a parent watchdog.
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();
        loop {
            line.clear();
            let count = reader.by_ref().take(3 * 1024 * 1024).read_line(&mut line)?;
            if count == 0 {
                break;
            }
            if !line.ends_with('\n') {
                return Err("pipe request exceeds bound".into());
            }
            let output =
                pipe::exchange(&gateway, &options.origin(), serde_json::from_str(&line)?).await?;
            println!("{}", serde_json::to_string(&output)?);
            std::io::stdout().flush()?;
        }
    } else {
        let listener =
            tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, options.port)).await?;
        println!(
            "Slice 8 review: {}/review (bounded {} seconds)",
            options.origin() + options.base.as_str(),
            options.duration.as_secs()
        );
        // The existing supervisor drains accepted work after either explicit stop or expiry.
        let shutdown = async {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = tokio::time::sleep(options.duration) => {},
            }
        };
        gateway
            .http
            .clone()
            .serve_acceptor_until(Loopback(listener), shutdown)
            .await;
    }
    gateway.engine.close().await?;
    Ok(())
}
