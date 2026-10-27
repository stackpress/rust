//! Generated pure page adapters execute the same typed contracts across all selected models.
#![cfg(feature = "ssr")]
use generated_admin_shared::ui::pages;
use leptos::prelude::*;
use serde_json::json;
use stackpress_framework_admin::{
    Draft,
    pages::{Action, Form},
};
use stackpress_framework_schema::Role;

/// Render each admitted placement and validate strict envelopes, defaults and identity behavior.
#[test]
fn typed_page_adapters_preserve_models_and_drafts() {
    let owner = Owner::new();
    owner.with(|| {
        for (module,row,patch) in [
            ("profile",json!({"id":"p","name":"</script> Ada","active":false,"address":null}),json!({"name":"Updated"})),
            ("project",json!({"id":"j","title":"Project","ownerId":"p","state":"open","tags":[]}),json!({"title":"Updated"})),
            ("task",json!({"id":"t","title":"Task","projectId":"j","done":false,"note":null,"priority":0}),json!({"note":null,"done":false,"priority":0})),
        ] {
            let spec = pages::spec(module).unwrap();
            assert_eq!(spec.module,module);
            assert_eq!(pages::identity(module).unwrap().0,module);
            let input=pages::input(module,Action::Create,row.clone()).unwrap();
            assert_eq!(input["id"],row["id"]);
            assert_eq!(pages::input(module,Action::Update,patch.clone()).unwrap(),patch);
            assert!(pages::input(module,Action::Create,json!({"private":"denied"})).is_err());
            pages::query(module,json!({"filters":[],"sort":[],"take":1,"skip":0,"total":true})).unwrap();
            assert!(pages::query(module,json!({"take":0})).is_err());
            let draft=Draft::new(&spec.model.fields,&row,Role::Field).unwrap();
            for action in [Action::Create,Action::Update,Action::Copy] {
                let rendered=pages::form(module,Form {action,draft:draft.clone(),instance:format!("{module}-form"),locked:Some(if module=="profile" {"name"} else {"title"}.into())}).unwrap().to_html();
                assert!(rendered.contains("disabled"));
                assert_eq!(rendered.contains("id=\"" ) && rendered.contains("-field-id\""), action!=Action::Update);
            }
            let filters=Draft::new(&spec.model.fields,&json!({}),Role::Filter).unwrap();
            assert!(!pages::filters(module,&filters,"filters").unwrap().to_html().is_empty());
            let table=pages::table(module,&json!({"rows":[{"row":row}],"total":1}),"table").unwrap().to_html();
            assert!(table.contains("<table"));
            assert!(!table.contains("</script> Ada"));
            assert!(!pages::detail(module,&row,"detail").unwrap().to_html().is_empty());
            assert!(pages::table(module,&json!({"rows":[{}]}),"bad").is_err());
            assert!(pages::detail(module,&json!({"private":true}),"bad").is_err());
        }
        for module in ["missing","../../private"] {
            assert!(pages::spec(module).is_err());
            assert!(pages::input(module,Action::Create,json!({})).is_err());
            assert!(pages::query(module,json!({})).is_err());
            assert!(pages::form(module,Form {action:Action::Create,draft:Draft::default(),instance:"bad".into(),locked:None}).is_err());
            assert!(pages::filters(module,&Draft::default(),"bad").is_err());
            assert!(pages::table(module,&json!({}),"bad").is_err());
            assert!(pages::detail(module,&json!({}),"bad").is_err());
            assert!(pages::identity(module).is_none());
        }
    });
    owner.cleanup();
}
