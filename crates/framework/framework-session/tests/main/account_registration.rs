//! Immediate and source Listen registration share one identical account event inventory.
use stackpress_framework_session::account::events::{register, register_listen};
use stackpress_lib::{Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};

/// Mark actual phase insertion without a resource, database or policy substitute.
async fn marker(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Ok(Flow::Continue)
}

/// Pin callback order and pass-through behavior of both public entry points.
#[tokio::test]
async fn immediate_and_listen_entry_points_share_exactly_seven_events() {
    let expected = [
        "auth-search",
        "auth-update",
        "auth-create",
        "auth-remove",
        "profile-detail",
        "profile-update",
        "profile-remove",
    ];
    for (select, queued) in [
        (register as fn(&mut ServerBuilder) -> Result<()>, false),
        (register_listen, true),
    ] {
        let mut app = ServerBuilder::new();
        app.on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.on("earlier-listen", marker)?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
        select(&mut app).unwrap();
        app.on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.on("later-listen", marker)?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
        let host = app.build().await.unwrap();
        let names: Vec<_> = host
            .listeners()
            .map(|listener| listener.pattern.clone())
            .collect();
        // Immediate registration predates every queued phase, including the earlier callback.
        let wanted: Vec<_> = if queued {
            std::iter::once("earlier-listen")
                .chain(expected)
                .chain(["later-listen"])
                .collect()
        } else {
            expected
                .into_iter()
                .chain(["earlier-listen", "later-listen"])
                .collect()
        };
        assert_eq!(names, wanted);
        for event in expected {
            assert_eq!(
                names.iter().filter(|name| name.as_str() == event).count(),
                1
            );
            // Each real handler retains the source no-Engine pass-through without validation IO.
            let result = host.resolve(event, serde_json::json!({})).await.unwrap();
            assert!(result.response.results.is_none());
        }
    }
}
