//! Real generated public contracts and every role render on the same shared source.
#![cfg(feature = "ssr")]

use generated_admin_shared::contracts;
use leptos::prelude::*;

#[path = "../../../generated-workspace/support/contracts.rs"]
mod contracts_cases;

/// Reuse the domain owner's real contract assertions against this new generated output root.
#[test]
fn generated_contracts_remain_real() {
    contracts_cases::prove_contracts();
    contracts_cases::prove_public_contracts();
}

/// Fixture-backed SSR contains all models, native associations and no active hostile markup.
#[test]
fn actual_generated_admin_ssr() {
    let owner = Owner::new();
    let html = owner.with(|| view! { <generated_admin_shared::Admin/> }.to_html());
    owner.cleanup();
    assert!(html.contains("Generated workspace admin"));
    assert!(html.contains("Profile"));
    assert!(html.contains("Project"));
    assert!(html.contains("Task"));
    assert!(!html.contains("Unable to render"));
    for role in ["field", "filter", "span"] {
        assert!(html.contains(&format!("data-role=\"{role}\"")));
    }
    assert!(html.contains("&lt;/script&gt;"));
    assert!(html.contains("profile-field-name"));
    assert!(html.contains("task-field-done"));
    assert!(html.contains("\"done\":false"));
}

/// Embedded Address wrappers use the same approved descriptors and generated Public type.
#[test]
fn generated_fieldsets_and_missing_drafts() {
    use generated_admin_shared::ui::address;
    use stackpress_framework_admin::Draft;
    use stackpress_framework_schema::Role;
    let owner = Owner::new();
    owner.with(|| {
        let fields = address::descriptors().unwrap();
        let draft = Draft::new(
            &fields,
            &serde_json::json!({"city":"Paris","country":"FR"}),
            Role::Field,
        )
        .unwrap();
        let output = address::field(&draft, "address").unwrap().to_html();
        assert!(output.contains("address-field-city"));
        let value = contracts::public::PublicAddress {
            city: "Paris".into(),
            country: "FR".into(),
        };
        assert!(
            address::view(&value, "view")
                .unwrap()
                .to_html()
                .contains("Paris")
        );
        let missing = Draft::default();
        assert!(address::field(&missing, "missing").is_err());
    });
    owner.cleanup();
}
