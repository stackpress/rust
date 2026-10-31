# Choose a consumer recipe

Use the existing standalone consumer whose graph matches the application you are building. Each has its own workspace, checked-in lock and separate target. Copy the application setup you need; do not copy native providers into a browser entry.

| Recipe | Starting point | What to supply or observe |
| --- | --- | --- |
| Full native | [full](../../consumers/full/Cargo.toml) and [shared native implementation](../../consumers/support/native.rs) | Static client, file SQLite, generated Note operations/admin, real JSON email, CSRF, public API and finite command providers. Application policy is explicit. |
| Headless / absent config | [headless](../../consumers/headless/src/main.rs) | Stock registrar; absent client/Engine/bindings; no startup IO or request guard. |
| API disabled | [api-disabled](../../consumers/api-disabled/Cargo.toml) | Select remaining leaves individually; API package must be absent from its normal graph. Omit its feature, then rebuild. |
| Direct constituents | [direct](../../consumers/direct/Cargo.toml) | Call the ordinary functions in source order; full/direct inventories match. Source bridges still import this aggregate crate. |
| Default facade disabled | [disabled](../../consumers/disabled/Cargo.toml) | Default stackpress with no aggregate feature; only stackpress-lib among first-party dependencies. |
| Aggregate facade | [facade](../../consumers/facade/Cargo.toml) | Import through stackpress::framework; full/facade inventories match. |
| Website / browser | [website](../../consumers/website/Cargo.toml) | Native SSR plus separate hydrate/CSR artifacts and public seeds. |
| Build host | [build-host](../../consumers/build-host/src/main.rs) | Generate/check Note with existing emitters; keep codegen out of runtime/browser graphs. |

## Use the facade

The facade consumer selects `framework-aggregate-full`. A named wrapper can still be passed to `ServerBuilder::plugin`; selecting the facade alone does not execute registration.

<!-- example: facade -->
```rust
use stackpress::{Flow, Invocation, Request, Response, Result, framework};
use stackpress_server::ServerBuilder;

async fn ready(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"facade ready")?;
    Ok(Flow::Continue)
}

pub async fn facade_host() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = ServerBuilder::new();
    app.plugin("framework", framework::register)?;
    app.on("ready", ready)?;
    let server = app.build().await?;
    assert_eq!(
        server
            .resolve("ready", stackpress::Nest::new())
            .await?
            .response
            .results
            .unwrap()
            .as_str(),
        Some("facade ready")
    );
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    Ok(())
}
```

The event result proves the import and ordinary registrar work together. The independent facade exports `framework_api`, `framework_csrf` and `framework_email` have their own opt-in feature families; they do not call the aggregate. See [feature lookup](../api/features.md).

## Reproduce a proof

Record `consumers/graphs.py --check` and `consumers/run.py` with the repository benchmark recorder and an absolute effort CARGO_TARGET_DIR. The runner compiles before its external runtime deadline, then checks exact full/direct/facade inventories and generated freshness. Locks must remain byte-identical for normal --locked --offline runs. Documentation examples use disposable copies of these graphs rather than modifying them.

Browser replay uses the approved cached Playwright path, fresh headless Chromium contexts and in-memory request interception, without a listener. [Verification](../verification.md) describes the full session matrix and the API replay gap; these recipes do not imply live provider or installed-platform acceptance.
