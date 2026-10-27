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

/// Tables preserve selected adapters, hostile data, explicit identities and empty datasets.
#[test]
fn generated_public_tables_validate_keys_before_rendering() {
    use generated_admin_shared::ui;
    let owner = Owner::new();
    owner.with(|| {
        let value:contracts::public::PublicTask=serde_json::from_value(serde_json::json!({"id":"t","projectId":"p","title":"</script> \"quoted\" 🦀", "done":false})).unwrap();
        let rows=vec![("key </script>".into(),value.clone())];
        let html=ui::task::table(&rows,"table").unwrap().to_html();
        assert!(html.contains("scope=\"col\""));assert!(html.contains("<thead>"));assert!(html.contains("<tbody>"));assert!(html.contains("No"));
        assert!(!html.contains("</script>"));assert!(html.contains("&lt;/script&gt;"));
        assert!(ui::task::table(&[],"empty").unwrap().to_html().contains("<tbody>"));
        for keys in [vec!["","other"],vec!["same","same"]] {
            let rows=keys.into_iter().map(|key|(key.into(),value.clone())).collect::<Vec<_>>();
            assert!(ui::task::table(&rows,"invalid").is_err());
        }
        assert!(ui::task::table(&vec![("x".into(),value.clone());101],"large").is_err());
        assert!(ui::task::table(&[("x".repeat(129),value)],"long").is_err());
        // Each model's real public DTO exercises its own selected adapter columns.
        let profile: contracts::public::PublicProfile = serde_json::from_value(
            serde_json::json!({"id":"p","name":"Ada", "active":false,"address":null}),
        ).unwrap();
        let project: contracts::public::PublicProject = serde_json::from_value(
            serde_json::json!({"id":"j","ownerId":"p","title":"Project", "state":"open","tags":["", "0"]}),
        ).unwrap();
        let proof: contracts::public::PublicRoleProof = serde_json::from_value(
            serde_json::json!({"id":"r","text":null,"flag":false,"zero":0,"rows":["", "0"],"metadata":{"key":"</script>"},"score":0,"slug":"slug"}),
        ).unwrap();
        assert!(ui::profile::table(&[("p".into(),profile)], "profile").unwrap().to_html().contains("Ada"));
        assert!(ui::project::table(&[("j".into(),project)], "project").unwrap().to_html().contains("open"));
        let proof = ui::role_proof::table(&[("r".into(),proof)], "proof").unwrap().to_html();
        assert!(proof.contains("&lt;/script&gt;"));
        assert!(proof.contains("No"));
        assert!(proof.contains("slug"));
        assert!(!proof.contains("ADMIN_PRIVATE_DEFAULT_SENTINEL"));
    });
    owner.cleanup();
}
