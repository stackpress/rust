//! Real normalized metadata, exact typed URL state, layout bounds and acknowledgement policy.
#![cfg(feature = "codegen")]
use serde_json::{Value, json};
use stackpress_framework_admin::{
    pages::{self, Action, AdminConfig, PageSpec, Reply, WriteState},
    search::{SearchQuery, decode, encode},
};
use stackpress_framework_schema::{
    FieldAccess, Schema, SortDirection, codegen::DefinitionRegistry,
};
use stackpress_idea_parser::{SchemaConfig, parse};

/// Actual revision-3 public query capabilities, with an integer zero and Boolean false.
fn model() -> stackpress_framework_schema::UiModel {
    let config:SchemaConfig=parse("fixture.idea", "model Example { id String @id name String @searchable @sortable @filter.input flag Boolean @filter.checkbox count Integer @filter.slider }", &Default::default()).unwrap();
    let mut registry = DefinitionRegistry::standard();
    for field in ["id", "name", "flag", "count"] {
        registry = registry
            .with_field_policy(
                "Example",
                field,
                FieldAccess {
                    readable: true,
                    writable: true,
                    ..Default::default()
                },
            )
            .unwrap();
    }
    Schema::normalize(&config, &registry, Default::default())
        .unwrap()
        .project_ui()
        .unwrap()
        .models
        .remove(0)
}

/// URL filtering and ordering preserve exact values and reject malformed one-decode inputs.
#[test]
fn exact_state_roundtrip_and_generated_query_shape() {
    let model = model();
    let mut query = SearchQuery::default();
    query.filter("flag", json!(false));
    query.filter("count", json!(0));
    query.filter("name", json!("%2F </script> 🦀"));
    query.cycle_order("name");
    assert_eq!(query.order, Some(("name".into(), SortDirection::Desc)));
    query.cycle_order("name");
    assert_eq!(query.order, Some(("name".into(), SortDirection::Asc)));
    query.cycle_order("name");
    assert!(query.order.is_none());
    let wire = query.operation(&model).unwrap();
    assert_eq!(wire["filters"][0]["predicate"]["value"], 0);
    assert_eq!(wire["filters"][1]["predicate"]["value"], false);
    let encoded = query.query().unwrap();
    assert_eq!(SearchQuery::from_query(&encoded).unwrap(), query);
    assert_eq!(decode(&encode("%2F + 🦀")).unwrap(), "%2F + 🦀");
    query.filter("flag", Value::Null);
    assert!(!query.filters.contains_key("flag"));
    query.order = Some(("name".into(), SortDirection::Desc));
    assert_eq!(
        query.operation(&model).unwrap()["sort"][0]["direction"],
        "desc"
    );
    for invalid in [
        "x=1",
        "state=%",
        "state=%GG",
        "state=%FF",
        "state=+",
        "state=%7B%7D&state=%7B%7D",
        "state=%257B%257D",
        "state=%7B%22unknown%22%3A1%7D",
    ] {
        assert!(SearchQuery::from_query(invalid).is_err(), "{invalid}");
    }
    assert_eq!(SearchQuery::from_query("").unwrap(), SearchQuery::default());
    for (skip, limit) in [(100_001, 20), (0, 0), (0, 101)] {
        let query = SearchQuery {
            skip,
            limit,
            ..Default::default()
        };
        assert!(query.operation(&model).is_err());
    }
    for (field, value) in [
        ("unknown", json!(1)),
        ("flag", json!(0)),
        ("count", json!("0")),
    ] {
        let mut query = SearchQuery::default();
        query.filter(field, value);
        assert!(query.operation(&model).is_err());
    }
    let mut query = SearchQuery {
        q: Some("x".repeat(4097)),
        ..Default::default()
    };
    assert!(query.query().is_err());
    assert!(query.operation(&model).is_err());
    query.q = None;
    query.order = Some(("flag".into(), SortDirection::Asc));
    assert!(query.operation(&model).is_err());
    let mut private = model.clone();
    for field in &mut private.fields {
        field.searchable = false;
    }
    query.order = None;
    query.q = Some("q".into());
    assert!(query.operation(&private).is_err());
}

/// Public page identity/configuration and uncertain writes never rely on truthiness.
#[test]
fn layout_identity_and_honest_acknowledgements() {
    for mount in ["/", "/admin/"] {
        assert!(AdminConfig::new(mount, vec!["profile".into()]).is_ok());
    }
    for mount in ["admin/", "/admin", "//", "/../", "/a?b/", "/\\/"] {
        assert!(pages::mount(mount).is_err());
    }
    for models in [
        vec![],
        vec!["profile".into(), "profile".into()],
        vec!["../x".into()],
    ] {
        assert!(AdminConfig::new("/", models).is_err());
    }
    let spec = PageSpec {
        module: "example".into(),
        model: model(),
        soft_remove: false,
    };
    assert_eq!(
        pages::key(&spec, &json!({"id":"", "name":"ignored"})).unwrap(),
        json!({"id":""})
    );
    for value in [json!({}), json!({"id":null})] {
        assert!(pages::key(&spec, &value).is_err());
    }
    assert_eq!(
        pages::input(
            &spec,
            Action::Copy,
            json!({"id":"old","flag":false,"count":0})
        ),
        json!({"flag":false,"count":0})
    );
    assert_eq!(
        pages::input(&spec, Action::Update, json!({"id":"old","flag":false})),
        json!({"flag":false})
    );
    assert_eq!(
        pages::input(&spec, Action::Create, json!({"id":"new"})),
        json!({"id":"new"})
    );
    let unknown = Reply {
        status: 500,
        envelope: json!({"code":500,"errors":{"outcome":"unknown"}}),
    };
    assert_eq!(WriteState::reply(&unknown), WriteState::Unknown);
    assert!(!WriteState::Unknown.message().contains("Saved"));
    assert_eq!(
        WriteState::reply(&Reply {
            status: 200,
            envelope: json!({"code":200,"results":{"id":"x"}})
        }),
        WriteState::Saved
    );
    for (status, envelope) in [
        (403, json!({"code":403,"error":"operation is forbidden"})),
        (500, json!({"code":500})),
        (200, json!({})),
    ] {
        assert!(matches!(
            WriteState::reply(&Reply { status, envelope }),
            WriteState::Failed(_)
        ));
    }
    for action in [
        Action::Search,
        Action::Detail,
        Action::Create,
        Action::Update,
        Action::Copy,
        Action::Remove,
        Action::Restore,
    ] {
        assert!(!action.name().is_empty());
    }
    assert!(!Action::Search.writes());
    assert!(Action::Copy.writes());
    assert!(WriteState::Idle.message().is_empty());
    assert!(!WriteState::Pending.message().is_empty());
}
