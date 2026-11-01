//! Source page-render skips, native development boundary and ordered finite builds.

use super::*;
use stackpress_framework::view;
use stackpress_framework_view::RenderedPage;

/// The finite native provider returns source props for assertions without opening a browser.
fn render(
    _: String,
    props: Value,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<RenderedPage>>> + Send>> {
    Box::pin(async move {
        Ok(Some(
            RenderedPage::new(serde_json::to_string(&props)?)
                .map_err(stackpress_lib::Error::from_source)?,
        ))
    })
}
/// A direct page loader invokes the same public aggregate view helper.
async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    res.data_mut()?.set("override", "response")?;
    view::render("account", req, res, inv).await
}
/// Record a successful finite build result; long contents are truncated only in the return value.
fn compiler(options: Value) -> view::NativeFuture<view::BuildEngine> {
    Box::pin(async move {
        let loaded = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let entries = loaded.clone();
        let set = Arc::new(move |entry: String| -> view::NativeFuture<()> {
            let entries = entries.clone();
            Box::pin(async move {
                if entry == "bad" {
                    return Err(stackpress_lib::Error::new("source entry load failed"));
                }
                entries.lock().unwrap().push(entry);
                Ok(())
            })
        });
        let size_entries = loaded.clone();
        let stage = |key: &str| -> view::Build {
            let destination = options["engine"][key].clone();
            let entries = loaded.clone();
            Arc::new(move || {
                let destination = destination.clone();
                let entries = entries.clone();
                Box::pin(async move {
                    Ok(
                        json!({"destination":destination,"entries":entries.lock().unwrap().clone(),"contents":"x".repeat(120)}),
                    )
                })
            })
        };
        Ok(view::BuildEngine {
            set,
            size: Arc::new(move || size_entries.lock().unwrap().len()),
            clients: stage("clientPath"),
            assets: stage("assetPath"),
            pages: stage("pagePath"),
        })
    })
}
/// Invoke the direct script so its ordered returned values remain inspectable.
async fn build(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    res.results(&view::build(&inv).await?)?;
    Ok(Flow::Continue)
}

/// Key presence, rather than value truthiness, activates view settings/build.
#[tokio::test]
async fn view_presence_and_development_route_hook() {
    for value in [Value::Null, json!(false), json!({})] {
        let host = application(json!({"view":value})).build().await.unwrap();
        assert_eq!(
            host.host().service::<view::Settings>().unwrap().config,
            value
        );
        assert!(host.listeners().any(|listener| listener.pattern == "build"));
    }
    let host = application(json!({"view":{},"server":{"mode":"development"}}))
        .build()
        .await
        .unwrap();
    let hook = host
        .listeners()
        .find(|listener| listener.pattern == "request")
        .unwrap();
    assert_eq!(hook.label, "request (Route)");
    // Raw/library calls own neither Node/native HTTP resource, so no middleware binding is needed.
    assert!(host.resolve("request", json!({})).await.is_ok());
}

/// The original JSON flag is presence-based, while redirect/string bodies skip rendering too.
#[tokio::test]
async fn source_render_props_status_and_skips() {
    let mut app = application(json!({"view":{"props":{"override":"config","configured":1}}}));
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.register(Arc::new(view::Provider {
                render: Arc::new(render),
            }))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.on("page", page).unwrap();
    app.on("me", public).unwrap();
    let host = app.build().await.unwrap();
    let mut req = Request::builder()
        .url("https://example.test/account?q=1#part")
        .data(json!({"input":"kept"}))
        .build()
        .unwrap();
    let mut res = Response::new();
    res.status_code(404, None).unwrap();
    host.emit("page", &mut req, &mut res).await.unwrap();
    let stackpress_lib::Body::Text(html) = res.body() else {
        panic!("rendered body")
    };
    let props: Value = serde_json::from_str(html).unwrap();
    assert_eq!(props["data"], json!({"override":"response","configured":1}));
    assert_eq!(props["session"], "public");
    assert_eq!(props["request"]["url"]["search"], "?q=1");
    assert_eq!(props["request"]["url"]["hash"], "#part");
    assert_eq!(props["request"]["data"]["input"], "kept");
    assert_eq!(res.status().unwrap().code, 404);
    for mode in ["flag", "redirect", "text"] {
        let mut req = Request::builder()
            .data(if mode == "flag" {
                json!({"json":null})
            } else {
                json!({})
            })
            .build()
            .unwrap();
        let mut res = Response::new();
        if mode == "redirect" {
            res.redirect("/next").unwrap();
        }
        if mode == "text" {
            res.html("existing").unwrap();
        }
        host.emit("page", &mut req, &mut res).await.unwrap();
        if mode == "flag" {
            assert!(matches!(res.body(), stackpress_lib::Body::Empty));
        }
        if mode == "text" {
            assert!(matches!(res.body(),stackpress_lib::Body::Text(text) if text=="existing"));
        }
    }
}

/// Ordered build stages use explicit finite providers; absent packaging never reads npm files.
#[tokio::test]
async fn build_stages_and_empty_catalog() {
    let root = sql::Fixture::new();
    let mut app = application(
        json!({"server":{"cwd":root.0},"view":{"engine":{"clientPath":"clients","assetPath":"assets","pagePath":"pages"}}}),
    );
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.register(Arc::new(view::Catalog {
                entries: vec!["account".into(), "signin".into()],
            }))?;
            start.register(Arc::new(view::BuildProvider {
                create: Arc::new(compiler),
            }))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.on("inspect-build", build).unwrap();
    let host = app.build().await.unwrap();
    let output = host
        .resolve("inspect-build", json!({}))
        .await
        .unwrap()
        .response
        .results
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(
        output
            .as_array()
            .unwrap()
            .iter()
            .map(|result| result["destination"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["clients", "assets", "pages"]
    );
    assert_eq!(output[0]["contents"], format!("{} ...", "x".repeat(100)));
    assert_eq!(output[0]["entries"], json!(["account", "signin"]));
    let mut req = Request::new();
    let mut res = Response::new();
    res.status_code(409, None).unwrap();
    host.emit("build", &mut req, &mut res).await.unwrap();
    assert_eq!(res.status().unwrap().code, 200);
    assert!(!root.0.join("package.json").exists());
    assert!(!root.0.join(".build").exists());
    let mut empty = application(json!({"view":{}}));
    empty.on("inspect-build", build).unwrap();
    empty
        .on_config(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.register(Arc::new(view::BuildProvider {
                    create: Arc::new(compiler),
                }))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    assert_eq!(
        empty
            .build()
            .await
            .unwrap()
            .resolve("inspect-build", json!({}))
            .await
            .unwrap()
            .response
            .results
            .unwrap()
            .to_json()
            .unwrap(),
        json!([])
    );
}

/// Source falsy paths skip individual stages even with entries; empty arrays/objects stay truthy.
#[tokio::test]
async fn build_path_truthiness_selects_only_source_stages() {
    for (paths, expected) in [
        (
            json!({"clientPath":null,"assetPath":false,"pagePath":0}),
            json!([]),
        ),
        (
            json!({"clientPath":[],"assetPath":{},"pagePath":""}),
            json!([[], {}]),
        ),
        (
            json!({"clientPath":"client","assetPath":false,"pagePath":"page"}),
            json!(["client", "page"]),
        ),
    ] {
        let mut app = application(json!({"view":{"engine":paths}}));
        // A real nonempty catalog distinguishes path selection from the earlier empty-engine skip.
        app.on_config(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.register(Arc::new(view::Catalog {
                    entries: vec!["account".into()],
                }))?;
                start.register(Arc::new(view::BuildProvider {
                    create: Arc::new(compiler),
                }))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
        app.on("inspect-build", build).unwrap();
        let host = app.build().await.unwrap();
        let response = host
            .resolve("inspect-build", json!({}))
            .await
            .unwrap()
            .response;
        let results = response.results.unwrap().to_json().unwrap();
        let destinations: Vec<_> = results
            .as_array()
            .unwrap()
            .iter()
            .map(|result| result["destination"].clone())
            .collect();
        assert_eq!(json!(destinations), expected);
    }
}

/// Install the same finite compiler with an optional retained packaging lifecycle handle.
async fn packaging_host(
    packaging: Option<Arc<view::Packaging>>,
    destination: Option<Value>,
) -> stackpress_server::Server {
    let mut config = json!({"server":{"cwd":"/source/app"},"view":{}});
    if let Some(destination) = destination {
        config["server"]["build"] = destination;
    }
    let mut app = application(config);
    app.on_config(task(move |start: &mut StartupContext| {
        let packaging = packaging.clone();
        Box::pin(async move {
            start.register(Arc::new(view::BuildProvider {
                create: Arc::new(compiler),
            }))?;
            if let Some(packaging) = packaging {
                start.register(packaging)?;
            }
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.build().await.unwrap()
}

/// Compare every projected response field, including the deliberate prior-error replacement.
async fn build_response(host: &stackpress_server::Server) -> Value {
    let mut req = Request::new();
    let mut res = Response::new();
    res.status_code(409, None).unwrap();
    host.emit("build", &mut req, &mut res).await.unwrap();
    res.try_to_json().unwrap()
}

/// Slow/failing packaging never affects the parent's source response, and failures stay observable.
#[tokio::test]
async fn packaging_is_non_awaited_and_failure_is_supervised() {
    let baseline = build_response(&packaging_host(None, None).await).await;
    for failing in [false, true] {
        let entered = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        let starts = entered.clone();
        let releases = release.clone();
        let packaging = Arc::new(view::Packaging::new(
            tokio::runtime::Handle::current(),
            Arc::new(move |paths| {
                assert_eq!(paths.cwd, std::path::Path::new("/source/app"));
                assert_eq!(paths.build, std::path::Path::new("/source/app/.build"));
                let entered = starts.clone();
                let release = releases.clone();
                Box::pin(async move {
                    entered.notify_one();
                    release.notified().await;
                    if failing {
                        return Err(stackpress_lib::Error::new("packaging failed"));
                    }
                    Ok(())
                })
            }),
        ));
        let host = packaging_host(Some(packaging.clone()), None).await;
        // This must finish without release; the external watchdog catches an accidental await.
        assert_eq!(build_response(&host).await, baseline);
        entered.notified().await;
        release.notify_one();
        let outcomes = packaging.drain().await;
        assert_eq!(outcomes.len(), 1);
        let result = outcomes.into_iter().next().unwrap().unwrap();
        if failing {
            assert_eq!(result.unwrap_err().to_string(), "packaging failed");
        } else {
            result.unwrap();
        }
        packaging.shutdown().await;
    }
    // Async source path errors also stay off the build response, before calling the provider.
    let packaging = Arc::new(view::Packaging::new(
        tokio::runtime::Handle::current(),
        Arc::new(|_| panic!("invalid packaging path must not reach callback")),
    ));
    let host = packaging_host(Some(packaging.clone()), Some(json!(false))).await;
    assert_eq!(build_response(&host).await, baseline);
    assert!(packaging.drain().await.pop().unwrap().unwrap().is_err());
    packaging.shutdown().await;
}

/// Own a callback resource until the explicit shutdown barrier has aborted and reaped its task.
struct PackageResource(Arc<AtomicUsize>);
impl Drop for PackageResource {
    /// Destruction is observable independently of a cancelled task's completion status.
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Shutdown interrupts a live drain and never leaves the callback's resource alive on return.
#[tokio::test]
async fn packaging_shutdown_reaps_suspended_task_and_live_drain() {
    use std::{future::Future, task::Poll};
    let entered = Arc::new(tokio::sync::Notify::new());
    let dropped = Arc::new(AtomicUsize::new(0));
    let starts = entered.clone();
    let drops = dropped.clone();
    let packaging = Arc::new(view::Packaging::new(
        tokio::runtime::Handle::current(),
        Arc::new(move |paths| {
            assert_eq!(paths.build, std::path::Path::new("/explicit/build"));
            let entered = starts.clone();
            let dropped = drops.clone();
            Box::pin(async move {
                let _resource = PackageResource(dropped);
                entered.notify_one();
                std::future::pending().await
            })
        }),
    ));
    let host = packaging_host(Some(packaging.clone()), Some(json!("/explicit/build"))).await;
    build_response(&host).await;
    entered.notified().await;
    let mut draining = Box::pin(packaging.drain());
    // Poll exactly once to establish a live waiter; scheduler sleeps do not establish ordering.
    std::future::poll_fn(|context| {
        assert!(draining.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
    let (outcomes, ()) = tokio::join!(draining, packaging.shutdown());
    assert_eq!(outcomes.len(), 1);
    assert!(
        outcomes
            .into_iter()
            .next()
            .unwrap()
            .unwrap_err()
            .is_cancelled()
    );
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    // The shutdown state cannot accept another job, even if the host remains retained.
    build_response(&host).await;
    assert!(packaging.drain().await.is_empty());
    packaging.shutdown().await;
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
}

/// A provider panic is a supervised task outcome and cannot become a build error.
#[tokio::test]
async fn packaging_panic_is_observable_without_changing_build_response() {
    let baseline = build_response(&packaging_host(None, None).await).await;
    let packaging = Arc::new(view::Packaging::new(
        tokio::runtime::Handle::current(),
        Arc::new(|_| panic!("finite packaging panic")),
    ));
    let host = packaging_host(Some(packaging.clone()), None).await;
    assert_eq!(build_response(&host).await, baseline);
    assert!(
        packaging
            .drain()
            .await
            .pop()
            .unwrap()
            .unwrap_err()
            .is_panic()
    );
    packaging.shutdown().await;
}

/// Source loads entries before checking output paths; load errors remain observable with no outputs.
#[tokio::test]
async fn build_entry_errors_precede_output_path_skips() {
    let mut app = application(json!({"view":{}}));
    app.on("inspect-build", build).unwrap();
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.register(Arc::new(view::Catalog {
                entries: vec!["bad".into()],
            }))?;
            start.register(Arc::new(view::BuildProvider {
                create: Arc::new(compiler),
            }))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let host = app.build().await.unwrap();
    assert_eq!(
        host.resolve("inspect-build", json!({}))
            .await
            .unwrap_err()
            .message(),
        "source entry load failed"
    );
}

/// Native HTTP middleware runs only with trusted execution-adapter context and may Abort.
#[tokio::test]
async fn native_middleware_marker_and_abort() {
    let mut app = application(json!({"view":{},"server":{"mode":"development"}}));
    app.on_config(task(|start: &mut StartupContext| {
        Box::pin(async move {
            let mut bindings = Bindings::default();
            bindings.bind("view-http", command);
            start.register(Arc::new(bindings))?;
            start.register(Arc::new(AtomicUsize::new(0)))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let host = app.build().await.unwrap();
    let mut req = Request::new();
    let mut res = Response::new();
    host.handle_with_options(
        "unused",
        &mut req,
        &mut res,
        stackpress_server::HandleOptions {
            adapter_context: Some(Arc::new(view::NativeHttp)),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        host.host()
            .service::<AtomicUsize>()
            .unwrap()
            .load(Ordering::SeqCst),
        1
    );
}
