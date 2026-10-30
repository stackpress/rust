//! Real auth-signup submits its original extension hook; the application delegates explicitly.
use super::*;
use std::sync::atomic::Ordering;

/// Application hook owns a snapshot while suspended, then forwards a fresh email request.
pub(super) async fn delegate(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let state = inv.service::<Fixture>()?;
    let mut snapshot = Response::new();
    snapshot.results_value(stackpress_lib::Value::Object(req.data.entries().clone()))?;
    let input = auth::events::json_result(&snapshot)?;
    state.hook_input.lock().unwrap().push(input.clone());
    // Explicit barriers prove nonawaiting and ownership without depending on scheduler delays.
    state.hook_entered.add_permits(1);
    state.hook_release.acquire().await.unwrap().forget();
    let mut mail=Request::builder().data(json!({"to":input["email"],"text":"signup hook","extension":{"source":"email-email-send"}})).build()?;
    inv.emit(EMAIL_SEND, &mut mail, res).await?;
    Ok(Flow::Continue)
}
/// Unbound jobs remain queued; bound tasks preserve payload and errors outside parent response.
#[tokio::test]
async fn real_signup_owned_hook_delegates_or_skips_and_retains_failures() {
    for (configured, fail, selected) in [
        (true, false, true),
        (false, false, true),
        (true, true, true),
        (true, false, false),
    ] {
        let (server, state) = host(
            json!({"signupHook":selected,"throws":fail}),
            configured.then(|| json!({"jsonTransport":true})),
        )
        .await;
        let mut req=Request::builder().data(json!({"name":"Signup","email":"signup@example.test","phone":"+1 (555) 123-4567","secret":"correct"})).build().unwrap();
        let mut res = Response::new();
        server
            .emit("auth-signup", &mut req, &mut res)
            .await
            .unwrap();
        assert_eq!(res.status().unwrap().code, 200);
        assert!(
            auth::events::json_result(&res).unwrap()["auth"]["email"]
                .get("secret")
                .is_none()
        );
        let before = format!("{:?}", res.to_envelope());
        if selected {
            state.hook_entered.acquire().await.unwrap().forget();
            let payload = state.hook_input.lock().unwrap()[0].clone();
            assert_eq!(payload["email"], "signup@example.test");
            assert!(payload["auth"]["email"].get("secret").is_some());
            // Mutation after parent return cannot rewrite the already owned hook payload.
            req.data.set("email", "changed@example.test").unwrap();
            state.hook_release.add_permits(1);
        }
        let outcomes = server.host().service::<Hooks>().unwrap().drain().await;
        assert_eq!(outcomes.len(), 2); // email-email-send and the intentionally missing phone hook.
        assert_eq!(
            outcomes
                .iter()
                .filter(|result| result.as_ref().unwrap().is_err())
                .count(),
            usize::from(selected && configured && fail)
        );
        assert_eq!(format!("{:?}", res.to_envelope()), before);
        assert_eq!(
            state.factory.created.load(Ordering::SeqCst),
            usize::from(selected && configured)
        );
        if selected && configured && !fail {
            assert_eq!(
                state.factory.sent.lock().unwrap()[0]["to"],
                "signup@example.test"
            );
        }
        server.host().service::<Hooks>().unwrap().shutdown().await;
        state.engine.close().await.unwrap();
    }
}
