//! Native decoded-head policy and one-attempt bounded body loading; Hyper alone parses HTTP.

use crate::external;
use http::header::{CONTENT_LENGTH, TRANSFER_ENCODING};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use stackpress_lib::{Error, Result};
use tokio::time::Instant;

/// Exact decoded-head charging, independent from a particular version's framing overhead.
pub(crate) fn head_bytes(request: &http::Request<Incoming>) -> usize {
    request.headers().iter().fold(
        request
            .method()
            .as_str()
            .len()
            .saturating_add(request.uri().to_string().len()),
        |sum, (name, value)| {
            sum.saturating_add(name.as_str().len())
                .saturating_add(value.as_bytes().len())
        },
    )
}
/// Hyper owns parsing; reject the surviving fields before any route or body loader runs.
pub(crate) fn check_head(
    request: &http::Request<Incoming>,
    limits: &crate::NetworkLimits,
    total: usize,
) -> Option<(u16, &'static str)> {
    let header_bytes = total
        .saturating_sub(request.method().as_str().len())
        .saturating_sub(request.uri().to_string().len());
    if request.headers().len() > limits.max_headers
        || header_bytes > limits.max_header_bytes
        || request.uri().to_string().len() > limits.max_target_bytes
    {
        return Some((431, "HTTP head limit exceeded"));
    }
    if total > limits.max_request_bytes {
        return Some((431, "HTTP head aggregate exceeded"));
    }
    let headers = request.headers();
    // G31: Hyper merges equal CLs and may drop CL after TE; reject only conflicts still visible.
    // The connection driver closes after every TE response, covering the otherwise invisible case.
    if headers.contains_key(TRANSFER_ENCODING) && headers.contains_key(CONTENT_LENGTH) {
        return Some((400, "HTTP visible conflicting framing"));
    }
    if headers.contains_key(TRANSFER_ENCODING)
        && (headers.get_all(TRANSFER_ENCODING).iter().count() != 1
            || !headers[TRANSFER_ENCODING]
                .as_bytes()
                .eq_ignore_ascii_case(b"chunked"))
    {
        return Some((400, "HTTP transfer coding unsupported"));
    }
    if headers.contains_key("trailer")
        || headers.contains_key("upgrade")
        || request.method() == http::Method::CONNECT
    {
        return Some((400, "HTTP trailers/upgrades/tunnels unsupported"));
    }
    None
}
/// Count decoded bytes incrementally, with separate absolute and decoded-progress deadlines.
pub(crate) async fn load(
    body: &mut Incoming,
    limits: &crate::NetworkLimits,
    head: usize,
    known: Option<u64>,
) -> Result<Vec<u8>> {
    if known.is_some_and(|n| {
        n > limits.max_body_bytes as u64 || n > limits.max_request_bytes.saturating_sub(head) as u64
    }) {
        return Err(input_error(413, "HTTP declared body limit exceeded"));
    }
    let total = Instant::now() + limits.body_timeout;
    let mut bytes = Vec::new();
    loop {
        let deadline = total.min(Instant::now() + limits.body_idle_timeout);
        let frame = tokio::time::timeout_at(deadline, body.frame())
            .await
            .map_err(|cause| external("HTTP body deadline exceeded", cause).with_code(408))?;
        let Some(frame) = frame else {
            return Ok(bytes);
        };
        let frame = frame
            .map_err(|cause| external("HTTP request body read failed", cause).with_code(400))?;
        let data = frame
            .into_data()
            .map_err(|_| input_error(400, "HTTP request trailers unsupported"))?;
        if data.len() > limits.max_body_bytes.saturating_sub(bytes.len())
            || data.len()
                > limits
                    .max_request_bytes
                    .saturating_sub(head)
                    .saturating_sub(bytes.len())
        {
            return Err(input_error(413, "HTTP decoded body limit exceeded"));
        }
        bytes.extend_from_slice(&data);
    }
}

/// Input policy chooses an HTTP status without exposing developer or native cause messages.
fn input_error(status: u16, message: &str) -> Error {
    Error::invalid(message).with_code(status)
}
