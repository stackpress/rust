//! Port-free native HTTP exchange for browser interception, using the same prepared host.
use crate::gateway::Gateway;
use base64::{Engine, engine::general_purpose::STANDARD};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Result};
use std::collections::BTreeMap;

/// One browser request; no storage, role or fault authority fields exist.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Real method selected by the browser.
    pub method: String,
    /// Exact local target including query.
    pub path: String,
    /// Original HTTP metadata, including cookies and Origin.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// Base64 preserves binary fixture/uploads.
    #[serde(default)]
    pub body: String,
}
/// Exact native HTTP response after transport decoding; binary bytes remain base64.
#[derive(Serialize)]
pub struct Output {
    /// Native status.
    pub status: u16,
    /// Multi-valued native metadata, preserving all Set-Cookie fields.
    pub headers: BTreeMap<String, Vec<String>>,
    /// Binary-safe body.
    pub body: String,
    /// True when native transport ended without a complete body; never a success claim.
    pub interrupted: bool,
}
/// Drive real HTTP/1 over duplex, jointly owning/reaping both connection futures.
/// # Errors
/// Returns native transport/body errors or the finite ten-second exchange deadline.
pub async fn exchange(gateway: &Gateway, origin: &str, input: Input) -> Result<Output> {
    let (server, client) = tokio::io::duplex(8192);
    let host = gateway.http.serve_io(
        server,
        "127.0.0.1:1".parse().expect("fixed peer"),
        std::future::pending(),
    );
    let browser = async {
        let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(client))
            .await
            .map_err(Error::from_source)?;
        let request = async {
            let mut request = http::Request::builder()
                .method(input.method.as_str())
                .uri(&input.path)
                .header(
                    "host",
                    origin
                        .strip_prefix("http://")
                        .ok_or_else(|| Error::invalid("pipe origin"))?,
                )
                .header("connection", "close");
            for (key, value) in input.headers {
                if !["host", "connection", "content-length", "transfer-encoding"]
                    .contains(&key.to_ascii_lowercase().as_str())
                {
                    request = request.header(key, value);
                }
            }
            let body = STANDARD.decode(input.body).map_err(Error::from_source)?;
            let response = sender
                .send_request(
                    request
                        .body(Full::new(Bytes::from(body)))
                        .map_err(Error::from_source)?,
                )
                .await
                .map_err(Error::from_source)?;
            let status = response.status().as_u16();
            let mut headers = BTreeMap::<String, Vec<String>>::new();
            for (key, value) in response.headers() {
                headers
                    .entry(key.to_string())
                    .or_default()
                    .push(value.to_str().map_err(Error::from_source)?.into());
            }
            let mut body = response.into_body();
            let mut bytes = Vec::new();
            let mut interrupted = false;
            while let Some(frame) = body.frame().await {
                match frame {
                    Ok(frame) => {
                        if let Ok(data) = frame.into_data() {
                            bytes.extend_from_slice(&data);
                        }
                    }
                    Err(_) => {
                        interrupted = true;
                        break;
                    }
                }
                if bytes.len() > 64 * 1024 * 1024 {
                    return Err(Error::limit("pipe response exceeded asset ceiling"));
                }
            }
            drop(sender);
            Ok(Output {
                status,
                headers,
                body: STANDARD.encode(bytes),
                interrupted,
            })
        };
        let (connection, response) = tokio::join!(connection, request);
        if let Err(error) = connection
            && !response.as_ref().is_ok_and(|reply| reply.interrupted)
        {
            return Err(Error::from_source(error));
        }
        response
    };
    let (_, response) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::join!(host, browser)
    })
    .await
    .map_err(Error::from_source)?;
    response
}
