//! Opt-in generic native-output publication after the ordinary HTTP lifecycle.
//!
//! Protocol hosts own validation and the native action. This module owns the same final-response
//! budgets, cookies and one-attempt OutputGuard as ordinary output. It contains no protocol IO.

use super::{
    common,
    stream::{HttpBody, StreamingHttpAdapter},
};
use crate::{HandleOptions, HandleReport, OutputGuard, Server};
use stackpress_lib::{CookieOptions, DeliveryState, Error, Response, Result};
use std::future::Future;

/// New sibling output type; existing exhaustive streaming enums remain unchanged.
pub enum NativeHttpOutput<N> {
    /// Ordinary preflighted HTTP response, with an unpolled body.
    Http(http::Response<HttpBody>),
    /// Host-owned protocol response/action selected by trusted preflight.
    Native(N),
    /// Framework stop intentionally suppresses output.
    Suppressed,
    /// Foreign/failed finalization cannot grant another write.
    Unavailable {
        /// Observed foundation delivery state.
        state: DeliveryState,
    },
}
/// Lifecycle and publication diagnostics for a host-specific native result.
pub struct NativeHttpReply<N> {
    /// One selected output opportunity.
    pub output: NativeHttpOutput<N>,
    /// Ordinary lifecycle report, retaining output failures.
    pub report: HandleReport,
}
/// Validate final response data, headers and cookie revisions before a native action is taken.
/// The protocol host must validate/regenerate its own status, framing and protected headers.
pub fn prepare_headers(response: &Response, cookie: &CookieOptions) -> Result<http::HeaderMap> {
    let limits = response.limits();
    response.body().validate(&limits)?;
    response.data().value().validate(&limits)?;
    response.errors().value().validate(&limits)?;
    let mut headers = common::response_headers(response, &limits, cookie)?;
    super::stream::strip_hop_headers(&mut headers)?;
    Ok(headers)
}
impl StreamingHttpAdapter {
    /// Run the normal lifecycle once, then preflight final output under one publication guard.
    ///
    /// `select` must validate its trusted protocol metadata before consuming a private descriptor.
    /// Returning None uses ordinary streaming conversion, which continues to reject bare 101.
    /// Dropping/rejecting a native value must release all of its resources without invoking user IO.
    pub async fn handle_native<F, Fut, N, P>(
        &self,
        app: &Server,
        parts: http::request::Parts,
        options: HandleOptions,
        load: F,
        select: P,
    ) -> Result<NativeHttpReply<N>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<Vec<u8>>> + Send,
        P: FnOnce(&Response, &CookieOptions) -> Result<Option<N>> + Send,
        N: Send,
    {
        self.handle_native_with(app, parts, options, load, (select, Ok))
            .await
    }
    /// Preflight both protocol selection and ordinary body admission before one output guard.
    /// The pair contains the native selector and the ordinary-body preflight respectively.
    pub async fn handle_native_with<F, Fut, N, P, Q>(
        &self,
        app: &Server,
        parts: http::request::Parts,
        options: HandleOptions,
        load: F,
        preflight: (P, Q),
    ) -> Result<NativeHttpReply<N>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<Vec<u8>>> + Send,
        P: FnOnce(&Response, &CookieOptions) -> Result<Option<N>> + Send,
        N: Send,
        Q: FnOnce(http::Response<HttpBody>) -> Result<http::Response<HttpBody>> + Send,
    {
        let (select, body_preflight) = preflight;
        let (mut reply, head) = self.handle_reply(app, parts, options, load).await?;
        let mut guard = OutputGuard::default();
        // Finalization is checked before protocol preflight; foreign writes never consume an intent.
        match reply.response.delivery_state() {
            DeliveryState::Stopped => {
                let _ = guard
                    .dispatch(&mut reply.response, |_| async { Ok(()) })
                    .await;
                reply.report.record_output(&reply.response);
                return Ok(NativeHttpReply {
                    output: NativeHttpOutput::Suppressed,
                    report: reply.report,
                });
            }
            DeliveryState::Open => {}
            state => {
                reply.report.record_output_failure(
                    Error::new("HTTP response finalized outside adapter authority"),
                    &reply.response,
                );
                return Ok(NativeHttpReply {
                    output: NativeHttpOutput::Unavailable { state },
                    report: reply.report,
                });
            }
        }
        // Any preflight error can still choose the single sanitized empty 500 response.
        let prepared =
            select(&reply.response, self.cookie_options()).and_then(|selected| match selected {
                Some(native) => Ok(NativeHttpOutput::Native(native)),
                None => super::stream::prepare(&reply.response, head, self.cookie_options())
                    .and_then(body_preflight)
                    .map(NativeHttpOutput::Http),
            });
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                reply.report.record_output_failure(error, &reply.response);
                reply.response = Response::new();
                let _ = reply.response.status_code(500, None);
                let mut response = http::Response::new(HttpBody::buffered(Vec::new()));
                *response.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
                response.headers_mut().insert(
                    http::header::CONTENT_LENGTH,
                    http::HeaderValue::from_static("0"),
                );
                NativeHttpOutput::Http(response)
            }
        };
        let mut output = None;
        let result = guard
            .dispatch(&mut reply.response, |_| async {
                output = Some(prepared);
                Ok(())
            })
            .await;
        if let Err(error) = result {
            reply.report.record_output_failure(error, &reply.response);
        } else {
            reply.report.record_output(&reply.response);
        }
        Ok(NativeHttpReply {
            output: output.unwrap_or(NativeHttpOutput::Unavailable {
                state: guard.state(),
            }),
            report: reply.report,
        })
    }
}
