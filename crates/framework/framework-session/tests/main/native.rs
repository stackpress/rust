//! JWT, source grant semantics and configuration/cache ownership.
use serde_json::{Value, json};
use stackpress_framework_session::{
    Clock, Service, SessionConfig, SessionPermission, SessionPermissionList, permissions,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};

/// Mutable test time with an observable read count, independent of scheduler delays.
pub struct TestClock {
    pub now: AtomicU64,
    pub reads: AtomicUsize,
}
impl TestClock {
    pub fn new(now: u64) -> Self {
        Self {
            now: AtomicU64::new(now),
            reads: AtomicUsize::new(0),
        }
    }
}
impl Clock for TestClock {
    fn now(&self) -> f64 {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.now.load(Ordering::SeqCst) as f64
    }
}
pub fn service(access: Option<SessionPermissionList>) -> (Service, Arc<TestClock>) {
    let clock = Arc::new(TestClock::new(1_700_000_000));
    (
        Service::with_clock(
            SessionConfig {
                access,
                ..Default::default()
            },
            clock.clone(),
        ),
        clock,
    )
}
#[tokio::test]
async fn original_jose_tokens_and_temporal_boundaries() {
    let vectors: Value = serde_json::from_str(include_str!("../jwt-vectors.json")).unwrap();
    let (service, _) = service(None);
    for case in vectors["vectors"].as_array().unwrap() {
        let session = service.load(case["token"].as_str().unwrap());
        assert_eq!(!session.guest().await, case["valid"], "{}", case["name"]);
        if case["valid"] == true {
            assert_eq!(
                session.data().await.unwrap(),
                case["claims"].as_object().unwrap()
            );
        }
    }
}
#[tokio::test]
async fn source_defaults_expiry_seed_and_guest() {
    let (service, clock) = service(None);
    assert_eq!(service.config().key, "session");
    assert_eq!(service.config().seed, "abc123");
    let token = service
        .create(json!({"custom":{"x":true}}).as_object().unwrap().clone())
        .await
        .unwrap();
    let identity = service.load(&token);
    assert!(!identity.guest().await);
    assert!(identity.data().await.unwrap().get("exp").is_none());
    // A valid arbitrary object without id is not a guest; authorization supplies default id.
    let auth = identity.authorization().await.unwrap();
    assert_eq!(auth["id"], 0);
    assert_eq!(auth["token"], token);
    assert_eq!(identity.token(), token);
    service.set_expires(1_700_000_001.0);
    let exp = service
        .create(json!({}).as_object().unwrap().clone())
        .await
        .unwrap();
    clock.now.store(1_700_000_001, Ordering::SeqCst);
    assert!(service.load(exp).guest().await);
    service.set_expires(0.0);
    let past = service
        .create(json!({"exp":1}).as_object().unwrap().clone())
        .await
        .unwrap();
    assert!(service.load(past).guest().await);
    service.set_expires(f64::NAN);
    assert!(service.create(Default::default()).await.is_err());
    for invalid in ["", "x.y.z", "eyJhbGciOiJub25lIn0.e30."] {
        assert!(service.load(invalid).guest().await);
    }
    // Changing verification seed affects new loads, not the verified instance's cache.
    service.configure(SessionConfig {
        seed: "wrong".into(),
        ..Default::default()
    });
    assert!(service.load(&token).guest().await);
    assert!(!identity.guest().await);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn memoization_is_per_instance_including_null_and_expiry() {
    let (service, clock) = service(None);
    let token = service
        .create(json!({"exp":1_700_000_001}).as_object().unwrap().clone())
        .await
        .unwrap();
    clock.reads.store(0, Ordering::SeqCst);
    let identity = Arc::new(service.load(&token));
    let mut jobs = Vec::new();
    for _ in 0..16 {
        let identity = identity.clone();
        jobs.push(tokio::spawn(async move {
            identity.data().await.unwrap()["exp"].clone()
        }));
    }
    for job in jobs {
        assert_eq!(job.await.unwrap(), 1_700_000_001);
    }
    assert_eq!(clock.reads.load(Ordering::SeqCst), 1);
    clock.now.store(1_700_000_002, Ordering::SeqCst);
    assert!(identity.data().await.is_some());
    assert!(service.load(&token).data().await.is_none());
    let invalid = service.load("bad");
    assert!(invalid.data().await.is_none());
    let before = clock.reads.load(Ordering::SeqCst);
    assert!(invalid.data().await.is_none());
    assert_eq!(clock.reads.load(Ordering::SeqCst), before);
}
#[tokio::test]
async fn all_of_empty_guest_empty_roles_and_route_identity() {
    let route = SessionPermission::route("ALL", "/profile/:id");
    let another = SessionPermission::route("ALL", "/profile/:id");
    let access = SessionPermissionList::from_iter([
        (
            "GUEST".into(),
            vec![SessionPermission::Event("read-*".into())],
        ),
        (
            "USER".into(),
            vec![
                SessionPermission::Event("read-*".into()),
                route.clone(),
                another,
            ],
        ),
        (
            "ADMIN".into(),
            vec![SessionPermission::Event("read-*".into()), route],
        ),
    ]);
    let (service, _) = service(Some(access));
    let guest = service.load("");
    assert!(guest.can(&[]).await.unwrap());
    assert!(
        guest
            .can(&[SessionPermission::Event("read-one".into())])
            .await
            .unwrap()
    );
    assert!(
        !guest
            .can(&[
                SessionPermission::Event("read-one".into()),
                SessionPermission::Event("write-one".into())
            ])
            .await
            .unwrap()
    );
    let token = service
        .create(
            json!({"roles":["USER","ADMIN","USER"]})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
        .unwrap();
    let identity = service.load(token);
    assert_eq!(identity.permits().await.unwrap().len(), 3);
    assert!(
        identity
            .can(&[SessionPermission::route("POST", "/profile/42")])
            .await
            .unwrap()
    );
    for roles in [json!([]), json!(["UNKNOWN"])] {
        let token = service
            .create(json!({"roles":roles}).as_object().unwrap().clone())
            .await
            .unwrap();
        assert!(service.load(token).permits().await.unwrap().is_empty());
    }
    let token = service
        .create(json!({"roles":false}).as_object().unwrap().clone())
        .await
        .unwrap();
    assert_eq!(service.load(token).permits().await.unwrap().len(), 1);
    let token = service
        .create(json!({"roles":"USER"}).as_object().unwrap().clone())
        .await
        .unwrap();
    assert!(service.load(token).permits().await.is_err());
    // can(empty) returns before even evaluating invalid source roles.
    let token = service
        .create(json!({"roles":"USER"}).as_object().unwrap().clone())
        .await
        .unwrap();
    assert!(service.load(token).can(&[]).await.unwrap());
}
#[test]
fn matching_source_language_and_mechanical_syntax_errors() {
    for (input, pattern, expected) in [
        ("READ-one", "read-*", true),
        ("read-a-b", "read-*", false),
        ("read-a-b", "read-**", true),
        ("aXb", "a.b", true),
        ("prefix", "/fix/", true),
        ("prefix", "/fix/y", false),
        ("fixmore", "/fix/y", true),
        ("UP", "/up/i", true),
        ("UP", "/up/", false),
    ] {
        assert_eq!(permissions::match_event(input, pattern).unwrap(), expected);
    }
    assert!(permissions::match_event("anything", "/[/").is_err());
    assert!(permissions::match_event("anything", "/x/ii").is_err());
    assert!(permissions::match_event("a", "/(?<=x)a/").is_err());
    let route = |method: &str, path: &str| stackpress_framework_session::SessionRoute {
        method: method.into(),
        route: path.into(),
    };
    assert!(permissions::match_route(&route("GET", "/a/42"), &route("ALL", "/a/:id")).unwrap());
    assert!(!permissions::match_route(&route("get", "/a/42"), &route("GET", "/a/:id")).unwrap());
    assert!(!permissions::match_route(&route("GET", "/a/42/"), &route("ALL", "/a/:id")).unwrap());
    assert!(permissions::match_route(&route("GET", "/a/42/more"), &route("ALL", "/a/**")).unwrap());
}

#[tokio::test]
async fn original_zero_length_seed_provider_boundary() {
    let source: Value = serde_json::from_str(include_str!("../empty-seed.json")).unwrap();
    let service = Service::new(SessionConfig {
        seed: String::new(),
        ..Default::default()
    });
    assert_eq!(
        service.create(Default::default()).await.is_err(),
        source["signError"]
    );
    assert_eq!(
        !service
            .load(source["token"].as_str().unwrap())
            .guest()
            .await,
        source["verified"]
    );
}
