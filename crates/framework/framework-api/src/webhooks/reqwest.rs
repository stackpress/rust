//! Native HTTP(S) fetch adapter; redirects are fetch mechanics, not delivery retries.
//!
//! Request preparation and redirect steps are shared with an in-process scripted exchange
//! in tests. Actual HTTP I/O stays in reqwest; no listener, worker or body-drain task is made.
//! Method/body and redirect transforms follow the
//! [Fetch Standard](https://fetch.spec.whatwg.org/#http-redirect-fetch).

use stackpress_lib::{Error, HandlerFuture, Result};

use super::fetch::{Fetch, FetchRequest};

/// Reusable native client with no API timeout, retry, signing or address restriction.
#[derive(Clone)]
pub struct ReqwestFetch {
    /// Reqwest owns pooling/TLS; the source wrapper owns method/body/redirect compatibility.
    client: reqwest::Client,
}

impl ReqwestFetch {
    /// Construct a client without DNS resolution or sending.
    ///
    /// The source Node fetch has no automatic Referer or environment proxy selection.
    /// Redirects are handled explicitly to preserve fetch's method/body/error rules.
    ///
    /// # Errors
    /// Returns native client/TLS setup failures without installing a partial transport.
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .referer(false)
            .no_proxy()
            .build()
            .map_err(Error::from_source)?;
        Ok(Self { client })
    }
}

impl Fetch for ReqwestFetch {
    /// Resolve at final response headers, ignore its body/status meaning, and drop the body.
    ///
    /// Dropping the borrowing future cancels active I/O. A completed external effect cannot
    /// be undone. Reqwest may close rather than pool a connection with an unread body.
    fn send<'a>(&'a self, request: FetchRequest<'a>) -> HandlerFuture<'a, Result<u16>> {
        Box::pin(async move {
            let request = prepare(&self.client, request)?;
            let client = self.client.clone();
            execute(request, move |request| {
                let client = client.clone();
                Box::pin(async move {
                    let response = client.execute(request).await.map_err(Error::from_source)?;
                    let status = response.status().as_u16();
                    let location = if is_redirect(status) {
                        response
                            .headers()
                            .get(reqwest::header::LOCATION)
                            .map(|value| value.to_str().map(str::to_owned))
                            .transpose()
                            .map_err(Error::from_source)?
                    } else {
                        None
                    };
                    // No response.bytes()/error_for_status(): fetch resolves at headers.
                    Ok(Hop { status, location })
                })
            })
            .await
        })
    }
}

/// Response metadata required for normal redirects; no response body enters API code.
struct Hop {
    /// All final HTTP codes are resolved statuses, including 4xx and 5xx.
    status: u16,
    /// A redirect with no Location resolves as-is.
    location: Option<String>,
}

/// Enforce Fetch Request construction rules before the first exchange.
fn prepare(client: &reqwest::Client, input: FetchRequest<'_>) -> Result<reqwest::Request> {
    let mut method =
        reqwest::Method::from_bytes(input.method.as_bytes()).map_err(Error::from_source)?;
    let upper = input.method.to_ascii_uppercase();
    if ["CONNECT", "TRACE", "TRACK"].contains(&upper.as_str()) {
        return Err(Error::new("fetch forbids this request method"));
    }
    // Fetch uppercases only its six normalized methods; extension spelling stays unchanged.
    if ["DELETE", "GET", "HEAD", "OPTIONS", "POST", "PUT"].contains(&upper.as_str()) {
        method = reqwest::Method::from_bytes(upper.as_bytes()).map_err(Error::from_source)?;
    }
    if method == reqwest::Method::GET || method == reqwest::Method::HEAD {
        return Err(Error::new("fetch GET/HEAD cannot have a body"));
    }
    let url = reqwest::Url::parse(input.uri).map_err(Error::from_source)?;
    check_url(&url)?;
    client
        .request(method, url)
        .header(reqwest::header::CONTENT_TYPE, input.content_type)
        .body(input.body.to_vec())
        .build()
        .map_err(Error::from_source)
}

/// HTTP redirect statuses defined by fetch, not every 3xx.
fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

/// Reject representations the source Request/fetch or native HTTP transport cannot use.
fn check_url(url: &reqwest::Url) -> Result<()> {
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Error::new("fetch URL cannot include credentials"));
    }
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::new("native fetch requires an HTTP(S) URL"));
    }
    Ok(())
}

/// Run one fetch including redirects; rejection exits immediately without replay.
async fn execute<F>(mut request: reqwest::Request, mut exchange: F) -> Result<u16>
where
    F: FnMut(reqwest::Request) -> HandlerFuture<'static, Result<Hop>>,
{
    let mut redirects = 0;
    loop {
        // Every API body is buffered bytes and therefore replayable for 307/308.
        let mut next = request
            .try_clone()
            .ok_or_else(|| Error::new("fetch redirect body is not replayable"))?;
        let response = exchange(request).await?;
        if !is_redirect(response.status) || response.location.is_none() {
            return Ok(response.status);
        }
        if redirects == 20 {
            return Err(Error::new("fetch redirect count exceeded"));
        }
        let location = response.location.as_deref().unwrap_or_default();
        let mut url = next.url().join(location).map_err(Error::from_source)?;
        if url.fragment().is_none() {
            // An absent fragment inherits the previous URL's fragment, per fetch.
            url.set_fragment(next.url().fragment());
        }
        check_url(&url)?;
        let method = next.method();
        if (matches!(response.status, 301 | 302) && method == reqwest::Method::POST)
            || (response.status == 303
                && method != reqwest::Method::GET
                && method != reqwest::Method::HEAD)
        {
            *next.method_mut() = reqwest::Method::GET;
            *next.body_mut() = None;
            // Fetch removes request-body headers when rewriting to GET.
            for name in [
                "content-encoding",
                "content-language",
                "content-location",
                "content-type",
            ] {
                next.headers_mut().remove(name);
            }
        }
        // This adapter supplies only Content-Type, never ambient auth/cookie credentials.
        *next.url_mut() = url;
        redirects += 1;
        request = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Build through production preparation while retaining all bytes for observation.
    fn request(method: &str) -> Result<reqwest::Request> {
        prepare(
            &ReqwestFetch::new()?.client,
            FetchRequest {
                uri: "https://source.invalid/path#fragment",
                method,
                content_type: "application/json",
                body: b"{\"secret\":\"kept\"}",
            },
        )
    }

    #[test]
    fn constructor_methods_body_url_and_normalization() -> Result<()> {
        for method in [
            "GET",
            "get",
            "HEAD",
            "CONNECT",
            "trace",
            "TRACK",
            "bad method",
            "",
        ] {
            assert!(request(method).is_err(), "{method}");
        }
        for (method, expected) in [
            ("post", "POST"),
            ("put", "PUT"),
            ("patch", "patch"),
            ("PURGE", "PURGE"),
        ] {
            let req = request(method)?;
            assert_eq!(req.method().as_str(), expected);
            assert_eq!(
                req.body().unwrap().as_bytes(),
                Some(b"{\"secret\":\"kept\"}".as_slice())
            );
            assert_eq!(req.headers().len(), 1);
        }
        for uri in ["relative", "http://u:p@private/path", "file:///tmp/test"] {
            assert!(
                prepare(
                    &ReqwestFetch::new()?.client,
                    FetchRequest {
                        uri,
                        method: "POST",
                        content_type: "application/json",
                        body: b"",
                    }
                )
                .is_err()
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn scripted_redirect_methods_body_headers_and_final_status() -> Result<()> {
        for status in [301, 302, 303, 307, 308] {
            for method in ["POST", "PUT", "patch"] {
                let seen = Arc::new(Mutex::new(Vec::new()));
                let observed = seen.clone();
                let code = execute(request(method)?, move |request| {
                    let seen = observed.clone();
                    Box::pin(async move {
                        let mut seen = seen.lock().unwrap();
                        seen.push(request);
                        Ok(if seen.len() == 1 {
                            Hop {
                                status,
                                location: Some("http://127.0.0.1/private".into()),
                            }
                        } else {
                            Hop {
                                status: 503,
                                location: None,
                            }
                        })
                    })
                })
                .await?;
                assert_eq!(code, 503);
                let seen = seen.lock().unwrap();
                let next = &seen[1];
                assert_eq!(next.url().as_str(), "http://127.0.0.1/private#fragment");
                let rewritten = (matches!(status, 301 | 302) && method == "POST") || status == 303;
                assert_eq!(
                    next.method().as_str(),
                    if rewritten { "GET" } else { method }
                );
                assert_eq!(next.body().is_none(), rewritten);
                assert_eq!(next.headers().get("content-type").is_none(), rewritten);
                assert!(!next.headers().contains_key("referer"));
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn redirect_count_missing_location_errors_and_no_retries() -> Result<()> {
        for redirects in [20, 21] {
            let mut calls = 0;
            let outcome = execute(request("POST")?, move |_| {
                calls += 1;
                Box::pin(async move {
                    Ok(Hop {
                        status: if calls <= redirects { 307 } else { 200 },
                        location: (calls <= redirects).then(|| "/again".into()),
                    })
                })
            })
            .await;
            assert_eq!(outcome.is_err(), redirects == 21);
        }
        for status in [200, 301, 304, 500] {
            assert_eq!(
                execute(request("POST")?, move |_| Box::pin(async move {
                    Ok(Hop {
                        status,
                        location: None,
                    })
                }))
                .await?,
                status
            );
        }
        for location in ["file:///tmp/no", "http://u:p@other.invalid", "http://["] {
            assert!(
                execute(request("POST")?, move |_| Box::pin(async move {
                    Ok(Hop {
                        status: 302,
                        location: Some(location.into()),
                    })
                }))
                .await
                .is_err()
            );
        }
        let calls = Arc::new(Mutex::new(0));
        let observed = calls.clone();
        let error = execute(request("POST")?, move |_| {
            *observed.lock().unwrap() += 1;
            Box::pin(async { Err(Error::new("original exchange rejection")) })
        })
        .await
        .unwrap_err();
        assert!(error.to_string().contains("original exchange rejection"));
        assert_eq!(*calls.lock().unwrap(), 1);
        Ok(())
    }
}
