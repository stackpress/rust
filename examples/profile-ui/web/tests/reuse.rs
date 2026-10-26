//! One compiled plugin changes every adapter at the generated operation boundary.
use stackpress_cli::{CliSinks, Terminal};
use stackpress_lib::ParseLimits;
use stackpress_server::http::{HttpAdapter, HttpOutput};

/// Compare native/direct, actual CLI, typed HTTP and SSR, including the seeded public data.
#[tokio::test]
async fn one_operation_one_compiled_customization() {
    let app = stackpress_example_profile_ui_web::application()
        .await
        .unwrap();
    let direct = app
        .call("profile-search", serde_json::json!({"id":"abc"}))
        .await
        .unwrap()
        .response
        .try_to_json()
        .unwrap();
    let prefix = if cfg!(feature = "demo-customization") {
        "Demo: "
    } else {
        ""
    };
    assert_eq!(
        direct["results"][0]["name"],
        format!("{prefix}Ada Lovelace")
    );
    let args = ["emit", "profile-search", "--id", "abc"].map(String::from);
    let terminal = Terminal::parse(&args, ParseLimits::default()).unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    stackpress_cli::run(&app, terminal, &mut CliSinks::new(&mut out, &mut err))
        .await
        .unwrap();
    assert!(err.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&out).unwrap(),
        direct
    );
    let adapter = HttpAdapter::new(Default::default()).unwrap();
    let req = http::Request::builder()
        .method("POST")
        .uri("/ui-api/v1/profile-search")
        .header("content-type", "application/json")
        .body(br#"{"revision":1,"id":"abc"}"#.to_vec())
        .unwrap();
    let HttpOutput::Response(response) = adapter.handle(&app, req).await.unwrap().output else {
        panic!("missing output")
    };
    let typed: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
    assert_eq!(typed["rows"], direct["results"]);
    assert_eq!(typed["total"], 3);
    let req = http::Request::builder()
        .uri("/ui/profiles?id=abc")
        .body(vec![])
        .unwrap();
    let HttpOutput::Response(response) = adapter.handle(&app, req).await.unwrap().output else {
        panic!("missing output")
    };
    let html = String::from_utf8(response.into_body()).unwrap();
    assert!(html.contains(&format!("{prefix}Ada Lovelace")));
    let seed = html
        .split("id=\"stackpress-page\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let seed: serde_json::Value = serde_json::from_str(seed).unwrap();
    assert_eq!(seed["data"]["reply"]["rows"], direct["results"]);
    assert_eq!(
        app.plugins()
            .filter(|p| p.id == "profile-ui-labels")
            .count(),
        usize::from(cfg!(feature = "demo-customization"))
    );
}
