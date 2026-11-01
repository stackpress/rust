//! Closing evidence for every source Abort position on routes and ordinary events.
use super::*;

/// Every matching handler may stop the chain, including its first and final registrations.
#[tokio::test]
async fn first_middle_last_abort_preserves_route_and_event_ties() {
    for event in ["shared-event", "GET /shared-route"] {
        for stop in 0..3 {
            let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
            let mut app = application(json!({}));
            for index in 0..3 {
                let calls = calls.clone();
                app.on(
                    event,
                    stackpress_lib::handler(
                        move |_: &mut Request, _: &mut Response, _: Invocation<'_>| {
                            let calls = calls.clone();
                            Box::pin(async move {
                                calls.lock().unwrap().push(index);
                                Ok(if index == stop {
                                    Flow::Abort
                                } else {
                                    Flow::Continue
                                })
                            })
                        },
                    ),
                )
                .unwrap();
            }
            let host = app.build().await.unwrap();
            let result = host.resolve(event, json!({})).await.unwrap();
            assert_eq!(result.outcome, stackpress_lib::DispatchOutcome::Aborted);
            assert_eq!(*calls.lock().unwrap(), (0..=stop).collect::<Vec<_>>());
        }
    }
}

/// Independent constituent selection preserves the original duplicate typed-section error.
#[test]
fn aggregate_plus_constituent_preserves_duplicate_error() {
    let mut app = ServerBuilder::new();
    stackpress_framework_session::composite::register(&mut app).unwrap();
    assert!(register(&mut app).is_err());
}

/// Two suspended locale-prefixed dispatches retain independent response cookies and results.
#[tokio::test]
async fn concurrent_locale_dispatches_keep_request_and_response_state() {
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut app = application(json!({"language":{"languages":{"fr":{},"de":{}}}}));
    app.get(
        "/page",
        stackpress_lib::handler(
            move |_: &mut Request, res: &mut Response, _: Invocation<'_>| {
                let barrier = barrier.clone();
                Box::pin(async move {
                    let locale = res
                        .session()
                        .get("locale")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned();
                    barrier.wait().await;
                    assert_eq!(
                        res.session().get("locale").unwrap().as_str(),
                        Some(locale.as_str())
                    );
                    res.results(&locale)?;
                    Ok(Flow::Abort)
                })
            },
        ),
    )
    .unwrap();
    let host = app.build().await.unwrap();
    async fn request(host: &stackpress_server::Server, locale: &str) -> String {
        let mut req = Request::builder()
            .url(format!("/{locale}/page"))
            .build()
            .unwrap();
        let mut res = Response::new();
        host.emit("request", &mut req, &mut res).await.unwrap();
        assert_eq!(req.url.path(), format!("/{locale}/page"));
        res.result().unwrap().as_str().unwrap().to_owned()
    }
    let (french, german) = tokio::join!(request(&host, "fr"), request(&host, "de"));
    assert_eq!((french.as_str(), german.as_str()), ("fr", "de"));
}
