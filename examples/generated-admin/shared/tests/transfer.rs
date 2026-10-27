//! Import SSR uses public descriptors and root-owned inert file data without dispatching requests.
#![cfg(feature = "ssr")]
use generated_admin_shared::{transfer, ui};
use leptos::prelude::*;
use serde_json::json;
use stackpress_framework_admin::pages::{Action, Snapshot};

/// Both empty and failed file selections remain inert and escaped before browser hydration.
#[test]
fn import_ssr_preserves_drafts_and_requires_preview_then_confirmation() {
    for file in [
        transfer::FileDraft::default(),
        transfer::FileDraft {
            name: "</script> hostile.csv".into(),
            csv: "id,name\nx,Draft".into(),
            error: "Unable to read UTF-8 CSV file".into(),
            ..Default::default()
        },
    ] {
        let owner = Owner::new();
        let html = owner.with(|| {
            provide_context(transfer::UploadState(RwSignal::new(file.clone())));
            let snapshot = Snapshot {
                transfer_limits: Default::default(),
                spec: ui::pages::spec("profile").unwrap(),
                action: Action::Import,
                allowed: vec![Action::Search, Action::Import, Action::Export],
                mount: "/admin/".into(),
                csrf: "test-csrf".into(),
                key: json!({}),
                data: json!({}),
                query: Default::default(),
                parent: None,
            };
            transfer::form(
                snapshot,
                Callback::new(|_| panic!("SSR must never dispatch")),
            )
            .to_html()
        });
        owner.cleanup();
        assert!(html.contains("Decode Stackpress export guards"));
        assert!(html.contains("Off by default"));
        assert!(html.contains("id=\"import-confirm\" type=\"button\" disabled"));
        assert!(!html.contains("id=\"import-preview-table\""));
        assert!(!html.contains("</script>"));
        if !file.error.is_empty() {
            assert!(html.contains("&lt;/script&gt; hostile.csv"));
            assert!(html.contains(&file.error));
        }
    }
}
