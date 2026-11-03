//! M1 source emission, complete construct retention and explicit external-family proofs.
#![cfg(feature = "codegen")]
use stackpress_framework_schema::{Role, Schema, SchemaErrorKind, ValueType, codegen::*};
use stackpress_idea_parser::{CompileInputs, SchemaConfig, parse};
use stackpress_idea_transformer::{
    Composer, CompositionLimits, MemoryLoader, OutputPlan, PackageLoader, Registry, Result,
    TransformContext, TransformSpec,
};
use std::collections::{BTreeMap, BTreeSet};

/// Compose actual shipped package bytes and optional app extensions with the real loader.
fn config(input: &str) -> SchemaConfig {
    let mut app = MemoryLoader::new("/source-emission").unwrap();
    app.insert("schema.idea", input);
    let loader = PackageLoader::new(app)
        .with_source(
            "stackpress-session/schema.idea",
            include_str!("../../framework-session/schema.idea"),
        )
        .unwrap()
        .with_source(
            "stackpress-api/schema.idea",
            include_str!("../../framework-api/schema.idea"),
        )
        .unwrap();
    Composer::new(loader, CompositionLimits::default())
        .compose("schema.idea")
        .unwrap()
        .config
}

/// Source schemas own a distinct partition and never call the Typed emitter.
fn emit(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let graph = Schema::normalize_source(context.schema(), NormalizeOptions::default(), &[])
        .map_err(transform_error)?;
    emit_source_contracts(&graph, output)
}

/// Session-only composition keeps API links external instead of synthesizing targets.
fn emit_session(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let graph = Schema::normalize_source(
        context.schema(),
        NormalizeOptions::default(),
        &["Application", "Session"],
    )
    .map_err(transform_error)?;
    emit_source_contracts(&graph, output)
}

/// Plan without a filesystem or an executable host/provider.
fn plan(config: &SchemaConfig) -> OutputPlan {
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("source").owns(&["source"]), emit)
        .unwrap();
    registry.plan(config, Vec::new()).unwrap()
}

#[test]
fn all_51_constructs_and_source_role_arguments_reach_emission() {
    let config = config("use \"stackpress-api/schema.idea\"");
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    let catalog = graph.catalog().unwrap();
    let mut attributes = BTreeSet::new();
    let mut role_count = 0;
    for model in &catalog.models {
        attributes.extend(model.attributes.iter().map(|attr| attr.name.clone()));
        for field in &model.fields {
            attributes.extend(field.attributes.iter().map(|attr| attr.name.clone()));
            let explicit: Vec<_> = field
                .attributes
                .iter()
                .filter(|attr| {
                    ["field.", "filter.", "span.", "view.", "list."]
                        .iter()
                        .any(|prefix| attr.name.starts_with(prefix))
                })
                .collect();
            assert_eq!(
                field.roles.len(),
                explicit.len(),
                "{}.{}",
                model.name,
                field.name
            );
            role_count += explicit.len();
            // Every positional or object prop is represented; raw values also remain exact.
            for (attr, binding) in explicit.into_iter().zip(&field.roles) {
                assert!(!binding.available, "metadata does not choose a renderer");
                if let Some(args) = attr.value.as_array() {
                    if args.len() == 1 && args[0].is_object() {
                        assert_eq!(serde_json::to_value(&binding.props).unwrap(), args[0]);
                    } else {
                        assert_eq!(binding.props.len(), args.len(), "{}", attr.name);
                        assert!(
                            args.iter()
                                .all(|value| binding.props.values().any(|prop| prop == value))
                        );
                    }
                }
            }
        }
    }
    assert_eq!(attributes.len(), 51);
    assert_eq!(role_count, 107);
    assert_eq!(catalog.relations.len(), 8);
    assert!(
        catalog
            .relations
            .iter()
            .all(|edge| !edge.external && edge.inverse.is_some())
    );
    let auth = catalog
        .queries
        .iter()
        .find(|query| query.model == "Auth")
        .unwrap();
    assert_eq!(auth.selectors, ["*", "profile.*"]);
    assert!(
        auth.paths.contains(&"secret".into()),
        "no added secret suppression"
    );
    assert!(auth.paths.contains(&"profile.references".into()));
    assert!(!auth.paths.contains(&"profile.auth".into()));
    let output = plan(&config);
    let files: BTreeMap<_, _> = output.files().map(|(path, _, text)| (path, text)).collect();
    for model in graph.models() {
        let encoded = format!("{:?}", serde_json::to_string(model).unwrap());
        let file = files[format!("source/{}.rs", model.module).as_str()];
        assert!(
            file.contains(&encoded),
            "complete {} metadata reaches output",
            model.name
        );
        assert!(!file.contains("SourceStore"), "M1 emits no stores/actions");
    }
    let auth_source = files["source/auth.rs"];
    assert!(auth_source.contains("kind: Kind::Token"));
    assert!(auth_source.contains("kind: Kind::Secret"));
    assert!(!auth_source.contains("stackpress_framework_session"));
}

#[test]
fn application_extension_emits_without_core_repair_and_metadata_is_inert() {
    let config = config(
        "use \"stackpress-api/schema.idea\" model Profile { nickname String? @default(\"[%inert%] {{value}}\") @field.input({type \"text\"}) }",
    );
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    let profile = graph
        .models()
        .iter()
        .find(|model| model.name == "Profile")
        .unwrap();
    assert_eq!(profile.fields.len(), 14);
    assert_eq!(profile.fields.last().unwrap().name, "nickname");
    assert!(
        profile
            .fields
            .last()
            .unwrap()
            .roles
            .iter()
            .any(|role| role.role == Role::Field)
    );
    let output = plan(&config);
    assert!(
        output
            .files()
            .any(|(_, _, text)| text.contains("[%inert%] {{value}}"))
    );
    let wrong = config_for("model Profile { id Boolean }");
    let graph = Schema::normalize_source(&wrong, NormalizeOptions::default(), &[]).unwrap();
    let witness = emit_source_witnesses(&graph.models()[0]).unwrap();
    assert!(witness.contains("type FieldValue = bool"));
    assert!(witness.contains("Declaration = stackpress_framework_schema::source::BooleanType"));
}

#[test]
fn source_assertion_dictionary_defaults_and_alias_keep_their_messages() {
    let config = config_for(
        "model Entry { title String @is.required @is.cgte(5) literal String @is.required(\"field validation failed\") }",
    );
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    let fields = &graph.models()[0].fields;
    assert_eq!(fields[0].rules[0].message, "Value is required.");
    assert_eq!(
        fields[0].rules[1].message,
        "Must be greater than or equal to {{arg}} characters."
    );
    assert_eq!(fields[1].rules[0].message, "field validation failed");
    assert!(!plan(&config).files().next().unwrap().2.is_empty());
}

/// Parse isolated rejection cases without package fixtures.
fn config_for(input: &str) -> SchemaConfig {
    parse("probe.idea", input, &CompileInputs::default()).unwrap()
}

#[test]
fn session_only_retains_external_links_without_hidden_api_output() {
    let config = config("use \"stackpress-session/schema.idea\"");
    let graph = Schema::normalize_source(
        &config,
        NormalizeOptions::default(),
        &["Application", "Session"],
    )
    .unwrap();
    let catalog = graph.catalog().unwrap();
    assert_eq!(
        catalog
            .models
            .iter()
            .map(|model| model.name.as_str())
            .collect::<Vec<_>>(),
        ["Profile", "Auth"]
    );
    let edges: Vec<_> = catalog
        .relations
        .iter()
        .filter(|edge| edge.external)
        .collect();
    assert_eq!(edges.len(), 2);
    assert!(
        edges
            .iter()
            .all(|edge| edge.local.is_none() && edge.foreign.is_none() && edge.inverse.is_none())
    );
    assert_eq!(
        catalog
            .models
            .iter()
            .flat_map(|model| &model.fields)
            .filter(|field| matches!(field.value_type, ValueType::Model(_)))
            .count(),
        4
    );
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("session-only").owns(&["source"]),
            emit_session,
        )
        .unwrap();
    let output = registry.plan(&config, Vec::new()).unwrap();
    let paths: Vec<_> = output.files().map(|(path, _, _)| path).collect();
    assert!(paths.contains(&"source/profile.rs") && paths.contains(&"source/auth.rs"));
    assert!(!paths.contains(&"source/application.rs") && !paths.contains(&"source/session.rs"));
}

#[test]
fn unsupported_emission_fails_before_any_file_is_contributed() {
    let config = config_for("model Good { id String } model Bad { id Integer }");
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    let mut output = OutputPlan::default();
    let error = emit_source_contracts(&graph, &mut output).unwrap_err();
    let cause = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<stackpress_framework_schema::SchemaError>()
        .unwrap();
    assert_eq!(cause.kind(), SchemaErrorKind::Unsupported);
    assert!(cause.to_string().contains("source scalar codec"));
    assert!(output.files().next().is_none());
    let unsupported = config_for("model Entry { id String title String @is.url }");
    let graph = Schema::normalize_source(&unsupported, NormalizeOptions::default(), &[]).unwrap();
    assert!(emit_source_contracts(&graph, &mut output).is_err());
    assert!(output.files().next().is_none());
    let bad = config_for("model Broken @query(\"missing\") { id String }");
    assert_eq!(
        Schema::normalize_source(&bad, NormalizeOptions::default(), &[])
            .unwrap_err()
            .kind(),
        SchemaErrorKind::Invalid
    );
}

#[test]
fn normalized_but_unemittable_constructs_are_explicit_before_output() {
    // Exhaust the source emitter's unsupported scalar/list/provider branches. None occurs
    // in the shipped four-model graph; accepted metadata is not silently executed as Typed.
    for field in [
        "value Integer",
        "value Number",
        "value Float",
        "value Integer[]",
        "value Number[]",
        "value Float[]",
        "value Boolean[]",
        "value Hash[]",
        "value Object[]",
        "value Json[]",
        "value Date[]",
        "value Time[]",
        "value Datetime[]",
        "value String[] @is.required",
        "value String @default(\"nanoid()\")",
        "value String @default(\"nanoid(8)\")",
        "value String @default(\"cuid(8)\")",
    ] {
        let config = config_for(&format!("model Entry {{ {field} }}"));
        let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
        let mut output = OutputPlan::default();
        assert!(
            emit_source_contracts(&graph, &mut output).is_err(),
            "{field}"
        );
        assert!(output.files().next().is_none(), "{field}");
    }
    // Each normalizable unsupported assertion family (including aliases) has a control.
    for field in [
        "value String @is.ne",
        "value String @is.notempty",
        "value String @is.string",
        "value Boolean @is.boolean",
        "value Integer @is.integer",
        "value Number @is.number",
        "value Float @is.float",
        "value Hash @is.object",
        "value String @is.unique",
        "value String @is.eq(\"x\")",
        "value String @is.neq(\"x\")",
        "value String @is.option([\"a\" \"b\"])",
        "value Number @is.gt(0)",
        "value Number @is.ge(0)",
        "value Number @is.gte(0)",
        "value Number @is.lt(1)",
        "value Number @is.le(1)",
        "value Number @is.lte(1)",
        "value String @is.ceq(2)",
        "value String @is.cgt(2)",
        "value String @is.clt(2)",
        "value String @is.cle(2)",
        "value String @is.clte(2)",
        "value String @is.weq(2)",
        "value String @is.wgt(2)",
        "value String @is.wge(2)",
        "value String @is.wgte(2)",
        "value String @is.wlt(2)",
        "value String @is.wle(2)",
        "value String @is.wlte(2)",
        "value String @is.lowercase",
        "value String @is.uppercase",
        "value String @is.slug",
        "value String @is.hex",
        "value String @is.url",
        "value String @is.regex(\"a\")",
        "value String @is.pattern(\"a\")",
        "value String @is.date",
        "value Datetime @is.future",
        "value Datetime @is.past",
        "value Datetime @is.present",
    ] {
        let config = config_for(&format!("model Entry {{ {field} }}"));
        let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
        let mut output = OutputPlan::default();
        assert!(
            emit_source_contracts(&graph, &mut output).is_err(),
            "{field}"
        );
        assert!(output.files().next().is_none(), "{field}");
    }
    for input in [
        "model Catalog { value String }",
        "model Profile { id String @id auth Auth[] } model Auth @query(\"profile\") { id String @id profileId String profile Profile @relation({local \"profileId\" foreign \"id\"}) }",
    ] {
        let graph =
            Schema::normalize_source(&config_for(input), NormalizeOptions::default(), &[]).unwrap();
        let mut output = OutputPlan::default();
        assert!(emit_source_contracts(&graph, &mut output).is_err());
        assert!(output.files().next().is_none());
    }
}
