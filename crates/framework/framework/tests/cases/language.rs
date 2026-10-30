//! Locale prefix raw dispatch preserves buffered values and never duplicates a native reader.
use super::*;
use stackpress_lib::{Body, LoadResult, RequestView};

/// Direct language helpers retain source insertion order, copied maps and scalar interpolation.
#[tokio::test]
async fn language_direct_helpers_and_falsy_configuration() {
    use stackpress_framework::language::{Scalar, Service};
    for value in [Value::Null, json!(false), json!(0), json!("")] {
        let host = application(json!({"language":value}))
            .build()
            .await
            .unwrap();
        assert!(host.host().service::<Service>().is_err());
        assert!(
            !host
                .listeners()
                .any(|listener| listener.pattern == "request")
        );
    }
    let host=application(json!({"language":{"key":"locale","languages":{"en_US":{"label":"English","translations":{"Hello %s":"Hi %s","empty":""}},"fr_FR":{"label":"Français","translations":{}}}}})).build().await.unwrap();
    let service = host.host().service::<Service>().unwrap();
    assert_eq!(service.locales(), ["en_US", "fr_FR"]);
    let mut copied = service.language("en_US").unwrap();
    copied["translations"]["changed"] = true.into();
    assert!(
        service.language("en_US").unwrap()["translations"]
            .get("changed")
            .is_none()
    );
    assert!(service.language("missing").is_none());
    let req = Request::new();
    let mut language = service.load(&req, "en_US");
    assert_eq!(language.label(), "English");
    assert_eq!(
        language.translate("Hello %s", &[Scalar::String("Ada")]),
        "Hi Ada"
    );
    assert_eq!(language.translate("empty", &[]), "empty");
    assert_eq!(
        language.translate(
            "%s|%s|%s|%s",
            &[
                Scalar::Boolean(false),
                Scalar::Null,
                Scalar::Undefined,
                Scalar::Number(1.0)
            ]
        ),
        "false|null|undefined|1"
    );
    assert_eq!(
        language.translate("[%s]", &[Scalar::String("$$ $& $` $'")]),
        "[$ %s [ ]]"
    );
    assert_eq!(
        language.translate(
            "%s %s %s %s %s",
            &[
                Scalar::Number(-0.0),
                Scalar::Number(f64::NAN),
                Scalar::Number(f64::INFINITY),
                Scalar::Number(f64::NEG_INFINITY),
                Scalar::Number(1e21)
            ]
        ),
        "0 NaN Infinity -Infinity 1e+21"
    );
    let mut res = Response::new();
    language.update("unknown", &mut res).unwrap();
    assert_eq!(language.label(), "unknown");
    assert!(language.translations().is_none());
    assert_eq!(
        res.session().get("locale").and_then(|v| v.as_str()),
        Some("unknown")
    );
    let cookie = Request::builder()
        .session(stackpress_lib::Session::from_pairs([("locale", "fr_FR")]).unwrap())
        .build()
        .unwrap();
    assert_eq!(service.load(&cookie, "en_US").locale, "fr_FR");
}

/// Inspect the exact reconstructed child and mutate it to prove parent request isolation.
async fn child(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    assert_eq!(req.url.path(), "/page");
    assert!(req.url.query().is_none());
    assert_eq!(req.query.path_as::<String>("q")?, Some("search".into()));
    assert_eq!(req.post.path_as::<String>("post")?, Some("posted".into()));
    assert_eq!(req.data.path_as::<String>("data")?, Some("explicit".into()));
    assert_eq!(req.headers.get("x-proof"), Some("retained"));
    assert_eq!(req.mimetype(), "application/octet-stream");
    assert_eq!(
        req.session.get("original").and_then(|value| value.as_str()),
        Some("cookie")
    );
    assert!(matches!(req.body(), Body::Bytes(bytes) if bytes == &[0, 1, 255]));
    assert_eq!(inv.stage()?, stackpress_server::LifecycleStage::Request);
    req.data.set("child-only", true)?;
    res.results(&"nested")?;
    Ok(Flow::Abort)
}

/// Source nested Abort does not suppress the original prefixed route's later dispatch.
async fn outer(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.data_mut()?.set("outer-ran", true)?;
    Ok(Flow::Continue)
}

/// An uncloneable native owner is borrowed once by the execution adapter, then drops once.
struct Reader {
    reads: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
}
impl Drop for Reader {
    /// Observe destruction without introducing a detached native resource.
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

/// Run this under the external watchdog as well as nextest; no executor-only timeout.
#[tokio::test]
async fn locale_prefix_nested_dispatch_keeps_owned_reader_and_source_no_abort() {
    let mut app = application(json!({"language":{"languages":{"fr":{},"de":{}}}}));
    app.get("/page", child).unwrap();
    app.get("/fr/page", outer).unwrap();
    let host = app.build().await.unwrap();
    let reads = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let reader = Reader {
        reads: reads.clone(),
        drops: drops.clone(),
    };
    let mut headers = stackpress_lib::Headers::new();
    headers.set("x-proof", "retained").unwrap();
    let mut req = Request::builder()
        .url("/fr/page?q=search")
        .headers(headers)
        .mimetype("application/octet-stream")
        .data(json!({"locale":"de", "data":"explicit"}))
        .post(json!({"post":"posted"}))
        .build()
        .unwrap();
    req.session.set("original", "cookie", None).unwrap();
    req.load_with(move |_: RequestView| async move {
        reader.reads.fetch_add(1, Ordering::SeqCst);
        Ok(LoadResult {
            body: Some(Body::Bytes(vec![0, 1, 255])),
            post: None,
        })
    })
    .await
    .unwrap();
    // Native IO finished at the adapter boundary, with no Clone or serialization of its owner.
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    let mut res = Response::new();
    host.handle_route(&mut req, &mut res).await.unwrap();
    assert_eq!(res.result().unwrap().as_str(), Some("nested"));
    assert_eq!(res.data().path_as::<bool>("outer-ran").unwrap(), Some(true));
    assert_eq!(
        res.session().get("locale").and_then(|value| value.as_str()),
        Some("fr")
    );
    assert!(!req.data.has("child-only"));
    assert_eq!(req.url.path(), "/fr/page");
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

/// Unknown prefixes do not dispatch or rewrite the URL; recognized data still stages cookies.
#[tokio::test]
async fn locale_data_only_unknown_prefix_and_missing_section_modes() {
    for config in [
        json!({}),
        json!({"language":{}}),
        json!({"language":{"languages":{"de":{}}}}),
    ] {
        let host = application(config.clone()).build().await.unwrap();
        let mut req = Request::builder()
            .url("/unknown/page")
            .data(json!({"locale":"de"}))
            .build()
            .unwrap();
        let mut res = Response::new();
        host.emit("request", &mut req, &mut res).await.unwrap();
        assert_eq!(
            res.session().get("locale").is_some(),
            config.pointer("/language/languages/de").is_some()
        );
        assert_eq!(req.url.path(), "/unknown/page");
        assert!(res.result().is_none());
    }
}
