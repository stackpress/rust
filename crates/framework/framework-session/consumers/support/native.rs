//! One direct proof serves both raw-crate and facade imports without a plugin or SQL host.
use crate::identity::{Service, SessionConfig, SessionData, SessionPermission};
use serde_json::json;

/// Keep short seed, optional expiry, all-of can, Guest defaults and per-instance memoization.
pub async fn prove() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config: SessionConfig = serde_json::from_value(json!({
        "key":"identity","seed":"x","access":{"USER":["profile-*"],"GUEST":["public-*"]}
    }))?;
    let service = Service::new(config.clone());
    let claims: SessionData = serde_json::from_value(json!({"id":"person","roles":["USER"]}))?;
    let token = service.create(claims).await?;
    let identity = service.load(token.clone());
    assert!(!identity.guest().await);
    assert!(!identity.data().await.unwrap().contains_key("exp"));
    assert!(identity.can(&[]).await?);
    assert!(
        identity
            .can(&[SessionPermission::Event("profile-search".into())])
            .await?
    );
    assert!(
        !identity
            .can(&[SessionPermission::Event("admin-search".into())])
            .await?
    );
    assert_eq!(identity.authorization().await?["token"], token);

    // Invalid JWT data becomes Guest, whose configured grants still apply as upstream.
    let guest = service.load("invalid");
    assert!(guest.data().await.is_none());
    assert!(guest.guest().await);
    assert_eq!(guest.authorization().await?["id"], 0);
    assert!(
        guest
            .can(&[SessionPermission::Event("public-search".into())])
            .await?
    );

    // Memoization belongs to the loaded Session, never a process-global or request-local slot.
    service.configure(SessionConfig {
        seed: "changed".into(),
        ..config
    });
    assert_eq!(identity.data().await.unwrap()["id"], "person");
    assert!(service.load(token).data().await.is_none());
    Ok(())
}
